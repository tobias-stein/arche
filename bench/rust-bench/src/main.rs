use clap::{Parser, Subcommand};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run fixed-concurrency benchmark
    Bench {
        #[arg(long, env = "ARCHE_API_KEY")]
        api_key: String,

        #[arg(long, default_value = "http://localhost:8080")]
        target: String,

        #[arg(long, default_value_t = 50)]
        concurrency: u32,

        #[arg(long, default_value_t = 30)]
        duration: u64,
    },

    /// Seed synthetic benchmark data
    Seed {
        #[arg(long, env = "ARCHE_API_KEY")]
        api_key: String,

        #[arg(long, default_value = "http://localhost:8080")]
        target: String,

        #[arg(long, default_value_t = 10)]
        blueprints: u32,

        #[arg(long, default_value_t = 4)]
        attributes: u32,

        #[arg(long, default_value_t = 0)]
        affixes: u32,
    },

    /// Run multi-scenario sweep
    Sweep {
        #[arg(long, env = "ARCHE_API_KEY")]
        api_key: String,

        #[arg(long, default_value = "http://localhost:8080")]
        target: String,

        #[arg(long, default_value = "bench/scenarios.yaml")]
        scenarios: String,

        #[arg(long, default_value_t = 10)]
        duration: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenariosConfig {
    dimensions: HashMap<String, Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Scenario {
    blueprint_count: u32,
    affix_count: u32,
    attribute_count: u32,
    concurrency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SweepResult {
    blueprint_count: u32,
    affix_count: u32,
    attribute_count: u32,
    concurrency: u32,
    duration_s: f64,
    total_requests: u64,
    throughput: f64,
    p50: Option<f64>,
    p95: Option<f64>,
    p99: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateClientResponse {
    id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateApiKeyResponse {
    id: Uuid,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateGlobalMetaResponse {
    id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateBlueprintResponse {
    id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateAffixResponse {
    id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteResponse {
    deleted: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Command::Bench {
            api_key,
            target,
            concurrency,
            duration,
        } => run_bench(&api_key, &target, concurrency, duration).await,
        Command::Seed {
            api_key,
            target,
            blueprints,
            attributes,
            affixes,
        } => run_seed(&api_key, &target, blueprints, attributes, affixes).await,
        Command::Sweep {
            api_key,
            target,
            scenarios,
            duration,
        } => run_sweep(&api_key, &target, &scenarios, duration).await,
    }
}

async fn run_bench(api_key: &str, target: &str, concurrency: u32, duration: u64) {
    let client = build_client(concurrency);
    let target = target.trim_end_matches('/').to_string();
    let url = format!("{target}/api/generate");

    let (total, elapsed, rate, latencies) =
        bench_loop(&client, &url, api_key, concurrency, duration).await;

    let p50 = percentile(&latencies, 50.0);
    let p95 = percentile(&latencies, 95.0);
    let p99 = percentile(&latencies, 99.0);

    println!();
    println!(
        "{} req in {:.1}s = {:.0} gen/s @ conc={}  ({} samples)",
        total,
        elapsed,
        rate,
        concurrency,
        latencies.len()
    );
    println!(
        "latency: p50={:.1}ms  p95={:.1}ms  p99={:.1}ms",
        p50.unwrap_or(0.0),
        p95.unwrap_or(0.0),
        p99.unwrap_or(0.0)
    );
}

async fn run_seed(
    api_key: &str,
    target: &str,
    blueprint_count: u32,
    attribute_count: u32,
    affix_count: u32,
) {
    let client = build_client(10);
    let target = target.trim_end_matches('/').to_string();

    let scenario = Scenario {
        blueprint_count,
        affix_count,
        attribute_count,
        concurrency: 1,
    };

    let client_id = create_client(&client, &target, api_key).await;
    let (_, scoped_key) = create_api_key(&client, &target, api_key, client_id).await;

    let global_meta_ids = create_global_meta_attributes(
        &client,
        &target,
        api_key,
        client_id,
        &scenario,
    )
    .await;

    let mut affix_ids = Vec::new();
    if affix_count > 0 {
        let mut rng = make_rng(&scenario, 0, 0);
        for i in 0..affix_count {
            let affix_name = format!("BenchAffix_{i:04}");
            let id = create_affix(
                &client, &target, api_key, client_id, &affix_name, &mut rng,
            )
            .await;
            affix_ids.push(id);
        }
    }

    for i in 0..blueprint_count {
        let name = format!("Blueprint-{i:04}");
            let body = generate_blueprint_body(
                &name,
                attribute_count,
                &scenario,
                i,
                &global_meta_ids,
                &affix_ids,
            );
            let _ = create_blueprint_raw(&client, &target, api_key, client_id, &body).await;
        if i % 20 == 0 {
            println!("  {}/{} blueprints created...", i + 1, blueprint_count);
        }
    }

    println!("Client ID: {client_id}");
    println!("API Key: {scoped_key}");
}

async fn run_sweep(
    api_key: &str,
    target: &str,
    config_path: &str,
    duration: u64,
) {
    let yaml_content = match fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {config_path}: {e}");
            std::process::exit(1);
        }
    };

    let config: ScenariosConfig = match serde_yaml::from_str(&yaml_content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing {config_path}: {e}");
            std::process::exit(1);
        }
    };

    let scenarios = cross_product(&config.dimensions);
    if scenarios.is_empty() {
        eprintln!("No scenarios generated from dimensions");
        std::process::exit(1);
    }

    let client = build_client(64);
    let target = target.trim_end_matches('/').to_string();
    let mut results = Vec::new();
    for (i, scenario) in scenarios.iter().enumerate() {
        let bp = scenario.blueprint_count;
        let af = scenario.affix_count;
        let at = scenario.attribute_count;
        let conc = scenario.concurrency;
        let client_name = format!("bench_bp{bp}_aff{af}_attr{at}_conc{conc}");

        println!(
            "\n--- [{}/{}] bp={}  aff={}  attr={}  conc={} ---",
            i + 1,
            scenarios.len(),
            bp,
            af,
            at,
            conc,
        );

        print!("  Seeding client and {bp} blueprint(s)...");

        let scenario_client_id = create_client_with_name(
            &client,
            &target,
            api_key,
            &client_name,
        )
        .await;
        let (scenario_key_id, scoped_key) =
            create_api_key(&client, &target, api_key, scenario_client_id).await;

        let global_meta_ids = create_global_meta_attributes(
            &client,
            &target,
            api_key,
            scenario_client_id,
            scenario,
        )
        .await;

        let mut affix_ids = Vec::new();
        if af > 0 {
            let mut rng = make_rng(scenario, 0, 0);
            for i in 0..af {
                let affix_name = format!("BenchAffix_{i:04}");
                let id = create_affix(
                    &client, &target, api_key, scenario_client_id, &affix_name, &mut rng,
                )
                .await;
                affix_ids.push(id);
            }
        }

        for j in 0..bp {
            let name = format!("Blueprint-{j:04}");
            let body = generate_blueprint_body(
                &name,
                at,
                scenario,
                j,
                &global_meta_ids,
                &affix_ids,
            );
            let _ = create_blueprint_raw(
                &client,
                &target,
                api_key,
                scenario_client_id,
                &body,
            )
            .await;
        }
        println!(" done");

        println!(
            "  Benchmarking (concurrency={conc}, duration={duration}s)..."
        );

        let url = format!("{target}/api/generate");
        let (total, elapsed, rate, latencies) =
            bench_loop(&client, &url, &scoped_key, conc, duration).await;

        let p50 = if latencies.len() >= 50 {
            Some(round_1dp(percentile(&latencies, 50.0).unwrap_or(0.0)))
        } else {
            None
        };
        let p95 = if latencies.len() >= 50 {
            Some(round_1dp(percentile(&latencies, 95.0).unwrap_or(0.0)))
        } else {
            None
        };
        let p99 = if latencies.len() >= 50 {
            Some(round_1dp(percentile(&latencies, 99.0).unwrap_or(0.0)))
        } else {
            None
        };

        let result = SweepResult {
            blueprint_count: bp,
            affix_count: af,
            attribute_count: at,
            concurrency: conc,
            duration_s: elapsed,
            total_requests: total,
            throughput: rate,
            p50,
            p95,
            p99,
        };

        println!(
            "  {} req in {:.1}s = {:.0} gen/s @ conc={}  (p50={}ms  p95={}ms  p99={}ms)",
            total,
            elapsed,
            rate,
            conc,
            p50.map_or("—".into(), |v| format!("{v:.1}")),
            p95.map_or("—".into(), |v| format!("{v:.1}")),
            p99.map_or("—".into(), |v| format!("{v:.1}")),
        );

        // Clean up immediately to avoid DB/cache bloat across scenarios
        let _ = delete_api_key(&client, &target, api_key, scenario_client_id, scenario_key_id).await;
        let _ = delete_client(&client, &target, api_key, scenario_client_id).await;

        results.push(result);
    }

    let csv_path = "bench/results.csv";
    write_csv(&results, csv_path).await;
    println!("\nResults written to {csv_path}");

    print_summary_table(&results);

    let html_path = "bench/report.html";
    generate_html_report(&results, html_path).await;
    println!("Report written to {html_path}");
}

fn build_client(max_concurrency: u32) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(max_concurrency as usize)
        .build()
        .unwrap()
}

async fn bench_loop(
    client: &Client,
    url: &str,
    api_key: &str,
    concurrency: u32,
    duration: u64,
) -> (u64, f64, f64, Vec<f64>) {
    let done = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));
    let latencies = Arc::new(Mutex::new(Vec::new()));

    let start = Instant::now();

    let mut handles = Vec::new();
    for _ in 0..concurrency {
        let client = client.clone();
        let url = url.to_string();
        let body = serde_json::json!({});
        let key = api_key.to_string();
        let done = done.clone();
        let errors = errors.clone();
        let latencies = latencies.clone();

        handles.push(tokio::spawn(async move {
            loop {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed >= duration as f64 {
                    break;
                }

                let req_start = Instant::now();
                match client
                    .post(&url)
                    .header("X-API-Key", &key)
                    .json(&body)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        let latency = req_start.elapsed().as_secs_f64() * 1000.0;
                        done.fetch_add(1, Ordering::Relaxed);
                        latencies.lock().unwrap().push(latency);
                    }
                    _ => {
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }));
    }

    let report_interval = Duration::from_secs(2);
    let mut last_count = 0u64;
    let mut last_time = Instant::now();

    loop {
        tokio::time::sleep(report_interval).await;
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed >= duration as f64 {
            break;
        }

        let current = done.load(Ordering::Relaxed);
        let dt = last_time.elapsed().as_secs_f64();
        let rate = (current - last_count) as f64 / dt;
        println!(
            "  {:6} req  {:5.1} gen/s  ({} err)",
            current,
            rate,
            errors.load(Ordering::Relaxed)
        );
        last_count = current;
        last_time = Instant::now();
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total = done.load(Ordering::Relaxed);
    let errs = errors.load(Ordering::Relaxed);
    let rate = total as f64 / elapsed;
    let latencies = latencies.lock().unwrap().clone();

    println!(
        "  {total} req in {elapsed:.1}s = {rate:.0} gen/s @ conc={concurrency}  ({errs} errors)"
    );

    (total, elapsed, rate, latencies)
}

fn percentile(sorted_samples: &[f64], p: f64) -> Option<f64> {
    if sorted_samples.is_empty() {
        return None;
    }
    let mut sorted = sorted_samples.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let rank = ((p / 100.0) * n as f64 + 0.5) as usize;
    let rank = rank.max(1).min(n);
    Some(sorted[rank - 1])
}

fn round_1dp(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn cross_product(dimensions: &HashMap<String, Vec<serde_json::Value>>) -> Vec<Scenario> {
    let key_map: HashMap<&str, &str> = [
        ("blueprints", "blueprint_count"),
        ("affixes", "affix_count"),
        ("attributes", "attribute_count"),
        ("concurrency", "concurrency"),
    ]
    .iter()
    .cloned()
    .collect();

    let keys: Vec<&String> = dimensions.keys().collect();
    if keys.is_empty() {
        return vec![];
    }

    let value_lists: Vec<&Vec<serde_json::Value>> =
        keys.iter().map(|k| &dimensions[*k]).collect();

    let mut results = Vec::new();
    let _indices: Vec<usize> = vec![0; keys.len()];
    let lengths: Vec<usize> = value_lists.iter().map(|v| v.len()).collect();

    fn recurse(
        depth: usize,
        keys: &[&String],
        key_map: &HashMap<&str, &str>,
        value_lists: &[&Vec<serde_json::Value>],
        lengths: &[usize],
        current: &mut Vec<usize>,
        results: &mut Vec<Scenario>,
    ) {
        if depth == keys.len() {
            let mut scenario = Scenario {
                blueprint_count: 0,
                affix_count: 0,
                attribute_count: 0,
                concurrency: 1,
            };
            for (i, key) in keys.iter().enumerate() {
                let dim_name = key.as_str();
                let field = key_map.get(dim_name).copied().unwrap_or(dim_name);
                let val = &value_lists[i][current[i]];
                let num = val.as_i64().unwrap_or(0) as u32;
                match field {
                    "blueprint_count" => scenario.blueprint_count = num,
                    "affix_count" => scenario.affix_count = num,
                    "attribute_count" => scenario.attribute_count = num,
                    "concurrency" => scenario.concurrency = num,
                    _ => {}
                }
            }
            results.push(scenario);
            return;
        }

        for i in 0..lengths[depth] {
            current[depth] = i;
            recurse(depth + 1, keys, key_map, value_lists, lengths, current, results);
        }
    }

    let mut current = vec![0usize; keys.len()];
    recurse(
        0,
        &keys,
        &key_map,
        &value_lists,
        &lengths,
        &mut current,
        &mut results,
    );

    results
}

async fn create_client(client: &Client, target: &str, api_key: &str) -> Uuid {
    create_client_with_name(client, target, api_key, "bench_scratch").await
}

async fn create_client_with_name(
    client: &Client,
    target: &str,
    api_key: &str,
    name: &str,
) -> Uuid {
    let body = serde_json::json!({ "name": name });
    let resp = client
        .post(format!("{target}/api/clients"))
        .header("X-API-Key", api_key)
        .json(&body)
        .send()
        .await
        .unwrap();
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        eprintln!("Error creating client ({status}): {text}");
        std::process::exit(1);
    }
    let cr: CreateClientResponse = resp.json().await.unwrap();
    cr.id
}

async fn create_api_key(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
) -> (Uuid, String) {
    let body = serde_json::json!({
        "name": "benchmark-key",
        "permissions": ["generate"],
    });
    let resp = client
        .post(format!("{target}/api/clients/{client_id}/keys"))
        .header("X-API-Key", api_key)
        .json(&body)
        .send()
        .await
        .unwrap();
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        eprintln!("Error creating API key ({status}): {text}");
        std::process::exit(1);
    }
    let akr: CreateApiKeyResponse = resp.json().await.unwrap();
    (akr.id, akr.key)
}

async fn create_global_meta_attribute(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
    body: &serde_json::Value,
) -> Uuid {
    let resp = client
        .post(format!(
            "{target}/api/global-meta-attributes?clientId={client_id}"
        ))
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .unwrap();
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        eprintln!("Error creating global meta attribute ({status}): {text}");
        std::process::exit(1);
    }
    let gmr: CreateGlobalMetaResponse = resp.json().await.unwrap();
    gmr.id
}

async fn create_blueprint_raw(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
    body: &serde_json::Value,
) -> Uuid {
    let resp = client
        .post(format!("{target}/api/blueprints?clientId={client_id}"))
        .header("X-API-Key", api_key)
        .json(body)
        .send()
        .await
        .unwrap();
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        eprintln!("Error creating blueprint ({status}): {text}");
        std::process::exit(1);
    }
    let br: CreateBlueprintResponse = resp.json().await.unwrap();
    br.id
}

async fn create_affix(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
    name: &str,
    rng: &mut ChaCha8Rng,
) -> Uuid {
    let description = format!("Affix attribute for {name}");
    let attr_body = make_attribute(rng, &description);

    let mut attribute = serde_json::Map::new();
    attribute.insert("name".to_string(), serde_json::Value::String(format!("{name}_attr")));
    attribute.insert("description".to_string(), serde_json::Value::String(description));
    if let Some(obj) = attr_body.as_object() {
        for (k, v) in obj {
            attribute.insert(k.clone(), v.clone());
        }
    }

    let affix_body = serde_json::json!({
        "name": name,
        "type": "prefix",
        "attribute": attribute,
    });

    let resp = client
        .post(format!("{target}/api/affixes?clientId={client_id}"))
        .header("X-API-Key", api_key)
        .json(&affix_body)
        .send()
        .await
        .unwrap();
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        eprintln!("Error creating affix ({status}): {text}");
        std::process::exit(1);
    }
    let ar: CreateAffixResponse = resp.json().await.unwrap();
    ar.id
}

async fn delete_api_key(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
    key_id: Uuid,
) -> bool {
    let resp = client
        .delete(format!(
            "{target}/api/clients/{client_id}/keys/{key_id}"
        ))
        .header("X-API-Key", api_key)
        .send()
        .await;
    match resp {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}

async fn delete_client(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
) -> bool {
    let resp = client
        .delete(format!("{target}/api/clients/{client_id}"))
        .header("X-API-Key", api_key)
        .send()
        .await;
    match resp {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}

async fn create_global_meta_attributes(
    client: &Client,
    target: &str,
    api_key: &str,
    client_id: Uuid,
    scenario: &Scenario,
) -> Vec<Uuid> {
    let num_global = std::cmp::max(1, scenario.attribute_count / 5);
    let mut rng = make_rng(scenario, 0, 0);
    let mut ids = Vec::new();
    for i in 0..num_global {
        let body = make_attribute(&mut rng, &format!("Global meta attribute global_meta_{i}"));
        let mut gma_body = match body {
            serde_json::Value::Object(m) => m,
            _ => continue,
        };
        gma_body.insert(
            "name".to_string(),
            serde_json::Value::String(format!("bench_global_meta_{i}")),
        );
        gma_body.entry("description".to_string()).or_insert_with(|| {
            serde_json::Value::String(format!("Global meta attribute global_meta_{i}"))
        });
        let id = create_global_meta_attribute(
            client,
            target,
            api_key,
            client_id,
            &serde_json::Value::Object(gma_body),
        )
        .await;
        ids.push(id);
    }
    ids
}

fn generate_blueprint_body(
    name: &str,
    attribute_count: u32,
    scenario: &Scenario,
    blueprint_idx: u32,
    global_meta_ids: &[Uuid],
    affix_ids: &[Uuid],
) -> serde_json::Value {
    let mut rng = make_rng(scenario, blueprint_idx, 0);
    let num_global = global_meta_ids.len();

    let mut attributes = serde_json::Map::new();
    let mut attribute_order = Vec::new();

    for i in 0..attribute_count {
        let attr_name = format!("attr_{i}");
        let description = format!("Attribute {attr_name}");

        let attr_body = if num_global > 0 && rng.gen::<f64>() < 0.2 {
            let ref_idx = rng.gen_range(0..num_global);
            serde_json::json!({ "$ref_id": global_meta_ids[ref_idx] })
        } else {
            make_attribute(&mut rng, &description)
        };

        attributes.insert(attr_name.clone(), attr_body);
        attribute_order.push(attr_name);
    }

    let prefixes: Vec<serde_json::Value> = affix_ids
        .iter()
        .map(|id| serde_json::json!({ "affixId": id, "weight": 1.0 }))
        .collect();

    let affix_count = affix_ids.len() as u32;
    let min_prefixes = if affix_count > 0 { 1 } else { 0 };

    serde_json::json!({
        "name": name,
        "archetype": "item",
        "weight": 1.0,
        "attributes": attributes,
        "attributeOrder": attribute_order,
        "affixes": {
            "minPrefixes": min_prefixes,
            "maxPrefixes": affix_count,
            "minSuffixes": 0,
            "maxSuffixes": 0,
            "prefixes": prefixes,
            "suffixes": [],
        }
    })
}

fn make_rng(scenario: &Scenario, blueprint_idx: u32, attr_idx: u32) -> ChaCha8Rng {
    let seed_str = format!(
        "arche-bench-{}-{}-{}-{}-{}",
        scenario.blueprint_count,
        scenario.affix_count,
        scenario.attribute_count,
        blueprint_idx,
        attr_idx,
    );
    let hash: u64 = farmhash_64(seed_str.as_bytes());
    ChaCha8Rng::seed_from_u64(hash)
}

fn farmhash_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xc6a4a7935bd1e995;
    let len = bytes.len();
    let mut i = 0;
    while i + 8 <= len {
        let mut k = u64::from_le_bytes([
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7],
        ]);
        k = k.wrapping_mul(0xc6a4a7935bd1e995);
        k ^= k >> 47;
        k = k.wrapping_mul(0xc6a4a7935bd1e995);
        h ^= k;
        h = h.wrapping_mul(0xc6a4a7935bd1e995);
        i += 8;
    }
    if i < len {
        let remainder = &bytes[i..];
        for &b in remainder.iter().rev() {
            h = (h << 8) | (b as u64);
        }
        h ^= h >> 47;
        h = h.wrapping_mul(0xc6a4a7935bd1e995);
    }
    h ^= h >> 47;
    h
}

const VALUE_TYPES: &[&str] = &["single", "enum", "range", "string", "boolean"];
const DISTRIBUTIONS: &[&str] = &["uniform", "normal", "exponential"];

fn make_attribute(rng: &mut ChaCha8Rng, description: &str) -> serde_json::Value {
    let value_type = VALUE_TYPES[rng.gen_range(0..VALUE_TYPES.len())];
    match value_type {
        "range" => make_range_attribute(rng, description),
        "single" => make_single_attribute(rng, description),
        "enum" => make_enum_attribute(rng, description),
        "string" => make_string_attribute(rng, description),
        "boolean" => make_boolean_attribute(rng, description),
        _ => unreachable!(),
    }
}

fn make_range_attribute(rng: &mut ChaCha8Rng, description: &str) -> serde_json::Value {
    let dist_type = DISTRIBUTIONS[rng.gen_range(0..DISTRIBUTIONS.len())];
    let min_val = round_2dp(rng.gen_range(0.0..50.0));
    let max_val = round_2dp(rng.gen_range(51.0..200.0));
    let mut attr = serde_json::json!({
        "description": description,
        "valueType": "range",
        "min": min_val,
        "max": max_val,
    });
    let dist = match dist_type {
        "uniform" => serde_json::json!({ "type": "uniform" }),
        "normal" => serde_json::json!({
            "type": "normal",
            "stdDev": round_2dp(rng.gen_range(1.0..50.0)),
        }),
        _ => serde_json::json!({
            "type": "exponential",
            "rate": round_4dp(rng.gen_range(0.01..1.0)),
        }),
    };
    attr.as_object_mut()
        .unwrap()
        .insert("distribution".to_string(), dist);
    attr
}

fn make_single_attribute(rng: &mut ChaCha8Rng, description: &str) -> serde_json::Value {
    serde_json::json!({
        "description": description,
        "valueType": "single",
        "value": round_2dp(rng.gen_range(0.0..100.0)),
    })
}

fn make_enum_attribute(rng: &mut ChaCha8Rng, description: &str) -> serde_json::Value {
    let n_options = rng.gen_range(3..=6);
    let options: Vec<String> = (0..n_options)
        .map(|_| format!("option_{:03}", rng.gen_range(0..1000)))
        .collect();
    serde_json::json!({
        "description": description,
        "valueType": "enum",
        "values": options,
    })
}

fn make_string_attribute(rng: &mut ChaCha8Rng, description: &str) -> serde_json::Value {
    serde_json::json!({
        "description": description,
        "valueType": "string",
        "minLength": rng.gen_range(1..10),
        "maxLength": rng.gen_range(10..100),
    })
}

fn make_boolean_attribute(rng: &mut ChaCha8Rng, description: &str) -> serde_json::Value {
    serde_json::json!({
        "description": description,
        "valueType": "boolean",
        "value": rng.gen_bool(0.5),
    })
}

fn round_2dp(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn round_4dp(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

async fn write_csv(results: &[SweepResult], path: &str) {
    let mut wtr = csv::Writer::from_path(path).unwrap();
    wtr.write_record([
        "blueprint_count",
        "affix_count",
        "attribute_count",
        "concurrency",
        "duration_s",
        "total_requests",
        "throughput",
        "p50_ms",
        "p95_ms",
        "p99_ms",
    ])
    .unwrap();
    for r in results {
        wtr.write_record(&[
            r.blueprint_count.to_string(),
            r.affix_count.to_string(),
            r.attribute_count.to_string(),
            r.concurrency.to_string(),
            format!("{:.2}", r.duration_s),
            r.total_requests.to_string(),
            format!("{:.2}", r.throughput),
            r.p50.map_or(String::new(), |v| format!("{v:.2}")),
            r.p95.map_or(String::new(), |v| format!("{v:.2}")),
            r.p99.map_or(String::new(), |v| format!("{v:.2}")),
        ])
        .unwrap();
    }
    wtr.flush().unwrap();
}

fn print_summary_table(results: &[SweepResult]) {
    if results.is_empty() {
        return;
    }

    println!();
    println!(
        "{:<50}  {:>10}  {:>10}  {:>8}  {:>8}  {:>8}",
        "Scenario", "Peak gen/s", "Concurrency", "p50(ms)", "p95(ms)", "p99(ms)"
    );

    for r in results {
        let label = format!(
            "bp={}  aff={}  attr={}  conc={}",
            r.blueprint_count, r.affix_count, r.attribute_count, r.concurrency
        );
        let p50_s = r.p50.map_or("—".into(), |v| format!("{v:.1}"));
        let p95_s = r.p95.map_or("—".into(), |v| format!("{v:.1}"));
        let p99_s = r.p99.map_or("—".into(), |v| format!("{v:.1}"));
        println!(
            "{:<50}  {:>10.0}  {:>10}  {:>8}  {:>8}  {:>8}",
            label, r.throughput, r.concurrency, p50_s, p95_s, p99_s
        );
    }

    if let Some(best) = results
        .iter()
        .max_by(|a, b| a.throughput.partial_cmp(&b.throughput).unwrap())
    {
        println!();
        println!(
            "Peak gen/s: {:.0}  (bp={} aff={} attr={} conc={})",
            best.throughput,
            best.blueprint_count,
            best.affix_count,
            best.attribute_count,
            best.concurrency
        );
        println!();
    }
}

async fn generate_html_report(results: &[SweepResult], path: &str) {
    let results_json = serde_json::to_string_pretty(results).unwrap();
    let mut bp_levels: Vec<u32> = results
        .iter()
        .map(|r| r.blueprint_count)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    bp_levels.sort();

    let mut subchart_tabs = String::new();
    let mut subchart_canvases = String::new();
    for (idx, bp) in bp_levels.iter().enumerate() {
        let active = if idx == 0 { "active" } else { "" };
        let show = if idx == 0 {
            "show active"
        } else {
            ""
        };
        subchart_tabs.push_str(&format!(
            r#"<li class="nav-item"><button class="nav-link {}" id="bp-tab-{}" data-bs-toggle="tab" data-bs-target="{}" type="button" role="tab" onclick="switchBlueprint()">{} blueprints</button></li>"#,
            active, bp, format!("#bp-chart-{}", bp), bp
        ));
        subchart_canvases.push_str(&format!(
            r#"<div class="tab-pane fade {show}" id="bp-chart-{bp}" role="tabpanel"><canvas id="throughputChart-{bp}"></canvas></div>"#
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Benchmark Report</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
<link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
<style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f8f9fa; }}
    h1 {{ color: #333; }}
    .chart-container {{ background: #fff; border-radius: 8px; padding: 20px; margin-bottom: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
    .filters {{ background: #fff; border-radius: 8px; padding: 16px 20px; margin-bottom: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
    .filters h3 {{ margin-top: 0; font-size: 16px; }}
    .filter-group {{ display: inline-block; margin-right: 24px; vertical-align: top; }}
    .filter-group label {{ display: block; font-size: 13px; margin: 2px 0; }}
    .filter-group label input {{ margin-right: 4px; }}
    canvas {{ max-height: 400px; }}
</style>
</head>
<body>
<div class="container">
    <h1>Benchmark Report</h1>
    <div class="filters" id="filters">
        <h3>Scenario Filter</h3>
        <div id="filter-controls"></div>
    </div>
    <div class="chart-container">
        <h2>Throughput vs Concurrency</h2>
        <ul class="nav nav-tabs" role="tablist">
            {subchart_tabs}
        </ul>
        <div class="tab-content">
            {subchart_canvases}
        </div>
    </div>
    <div class="chart-container">
        <h2>Peak Throughput by Scenario</h2>
        <canvas id="peakChart"></canvas>
    </div>
    <div class="chart-container">
        <h2>Latency Percentiles</h2>
        <canvas id="latencyChart"></canvas>
    </div>
</div>
<script>
const BENCH_DATA = {results_json};

const COLORS = [
    'rgba(54, 162, 235, 0.8)', 'rgba(255, 99, 132, 0.8)',
    'rgba(75, 192, 192, 0.8)', 'rgba(255, 159, 64, 0.8)',
    'rgba(153, 102, 255, 0.8)', 'rgba(255, 205, 86, 0.8)',
    'rgba(201, 203, 207, 0.8)', 'rgba(34, 139, 34, 0.8)',
];

function buildFilterControls() {{
    var bpSet = new Set(BENCH_DATA.map(function(r) {{ return r.blueprint_count; }}));
    var affSet = new Set(BENCH_DATA.map(function(r) {{ return r.affix_count; }}));
    var attrSet = new Set(BENCH_DATA.map(function(r) {{ return r.attribute_count; }}));

    var html = '';
    if (bpSet.size > 0) {{
        html += '<div class="filter-group"><strong>Blueprints</strong>';
        bpSet.forEach(function(v) {{
            html += '<label><input type="checkbox" class="filter-bp" value="' + v + '" checked onchange="applyFilters()"> ' + v + '</label>';
        }});
        html += '</div>';
    }}
    if (affSet.size > 0) {{
        html += '<div class="filter-group"><strong>Affixes</strong>';
        affSet.forEach(function(v) {{
            html += '<label><input type="checkbox" class="filter-aff" value="' + v + '" checked onchange="applyFilters()"> ' + v + '</label>';
        }});
        html += '</div>';
    }}
    if (attrSet.size > 0) {{
        html += '<div class="filter-group"><strong>Attributes</strong>';
        attrSet.forEach(function(v) {{
            html += '<label><input type="checkbox" class="filter-attr" value="' + v + '" checked onchange="applyFilters()"> ' + v + '</label>';
        }});
        html += '</div>';
    }}
    document.getElementById('filter-controls').innerHTML = html;
}}

function getFilteredData() {{
    var checkedBP = Array.from(document.querySelectorAll('.filter-bp:checked')).map(function(cb) {{ return Number(cb.value); }});
    var checkedAff = Array.from(document.querySelectorAll('.filter-aff:checked')).map(function(cb) {{ return Number(cb.value); }});
    var checkedAttr = Array.from(document.querySelectorAll('.filter-attr:checked')).map(function(cb) {{ return Number(cb.value); }});
    return BENCH_DATA.filter(function(r) {{
        return checkedBP.indexOf(r.blueprint_count) !== -1 &&
               checkedAff.indexOf(r.affix_count) !== -1 &&
               checkedAttr.indexOf(r.attribute_count) !== -1;
    }});
}}

var peakChart, latencyChart;

function applyFilters() {{
    var filtered = getFilteredData();
    updatePeakChart(filtered);
    updateLatencyChart(filtered);
    updateThroughputCharts(filtered);
}}

function getAttrColor(attrCount) {{
    if (attrCount <= 3) return 'rgba(54, 162, 235, 0.8)';
    if (attrCount <= 10) return 'rgba(255, 159, 64, 0.8)';
    return 'rgba(255, 99, 132, 0.8)';
}}

function updatePeakChart(filtered) {{
    var sorted = filtered.slice().sort(function(a, b) {{ return parseFloat(b.throughput) - parseFloat(a.throughput); }});
    var labels = sorted.map(function(r) {{ return 'bp' + r.blueprint_count + ' aff' + r.affix_count + ' attr' + r.attribute_count; }});
    var data = sorted.map(function(r) {{ return parseFloat(r.throughput); }});
    var bgColors = sorted.map(function(r) {{ return getAttrColor(r.attribute_count); }});

    if (peakChart) peakChart.destroy();
    peakChart = new Chart(document.getElementById('peakChart'), {{
        type: 'bar',
        data: {{
            labels: labels,
            datasets: [{{
                label: 'gen/s',
                data: data,
                backgroundColor: bgColors,
            }}],
        }},
        options: {{
            responsive: true,
            plugins: {{
                title: {{ display: true, text: 'Peak Throughput by Scenario' }},
                legend: {{ display: false }},
            }},
            scales: {{
                x: {{ title: {{ display: true, text: 'Scenario' }} }},
                y: {{ title: {{ display: true, text: 'gen/s' }}, beginAtZero: true }},
            }},
        }},
    }});
}}

function updateLatencyChart(filtered) {{
    var withLatency = filtered.filter(function(r) {{ return r.p50 !== null; }});
    var labels = withLatency.map(function(r) {{ return 'bp' + r.blueprint_count + ' aff' + r.affix_count + ' attr' + r.attribute_count; }});
    var p50 = withLatency.map(function(r) {{ return r.p50; }});
    var p95 = withLatency.map(function(r) {{ return r.p95; }});
    var p99 = withLatency.map(function(r) {{ return r.p99; }});

    if (latencyChart) latencyChart.destroy();
    latencyChart = new Chart(document.getElementById('latencyChart'), {{
        type: 'bar',
        data: {{
            labels: labels,
            datasets: [
                {{ label: 'p50 (ms)', data: p50, backgroundColor: 'rgba(54, 162, 235, 0.7)' }},
                {{ label: 'p95 (ms)', data: p95, backgroundColor: 'rgba(255, 159, 64, 0.7)' }},
                {{ label: 'p99 (ms)', data: p99, backgroundColor: 'rgba(255, 99, 132, 0.7)' }},
            ],
        }},
        options: {{
            responsive: true,
            plugins: {{
                title: {{ display: true, text: 'Latency Percentiles' }},
                legend: {{ position: 'bottom' }},
            }},
            scales: {{
                x: {{ title: {{ display: true, text: 'Scenario' }} }},
                y: {{ title: {{ display: true, text: 'ms' }}, beginAtZero: true }},
            }},
        }},
    }});
}}

var throughputCharts = {{}};

function updateThroughputCharts(filtered) {{
    var bpLevels = [...new Set(BENCH_DATA.map(function(r) {{ return r.blueprint_count; }}))].sort(function(a, b) {{ return a - b; }});
    bpLevels.forEach(function(bp) {{
        var bpData = filtered.filter(function(r) {{ return r.blueprint_count === bp; }});
        var seriesMap = {{}};
        bpData.forEach(function(r) {{
            var key = 'bp' + r.blueprint_count + ' aff' + r.affix_count + ' attr' + r.attribute_count;
            if (!seriesMap[key]) seriesMap[key] = [];
            seriesMap[key].push({{ x: r.concurrency, y: parseFloat(r.throughput) }});
        }});

        var datasets = [];
        var colorIdx = 0;
        for (var key in seriesMap) {{
            datasets.push({{
                label: key,
                data: seriesMap[key],
                borderColor: COLORS[colorIdx % COLORS.length],
                backgroundColor: COLORS[colorIdx % COLORS.length].replace('0.8', '0.2'),
                fill: false,
                tension: 0.1,
                pointRadius: 5,
            }});
            colorIdx++;
        }}

        if (throughputCharts[bp]) throughputCharts[bp].destroy();

        var canvas = document.getElementById('throughputChart-' + bp);
        if (canvas) {{
            throughputCharts[bp] = new Chart(canvas, {{
                type: 'line',
                data: {{ datasets: datasets }},
                options: {{
                    responsive: true,
                    plugins: {{
                        title: {{ display: true, text: 'Throughput vs Concurrency (bp=' + bp + ')' }},
                        legend: {{ position: 'bottom' }},
                    }},
                    scales: {{
                        x: {{ title: {{ display: true, text: 'Concurrency' }}, type: 'linear' }},
                        y: {{ title: {{ display: true, text: 'gen/s' }}, beginAtZero: true }},
                    }},
                }},
            }});
        }}
    }});
}}

function switchBlueprint() {{
    applyFilters();
}}

buildFilterControls();
var initialFiltered = getFilteredData();
updatePeakChart(initialFiltered);
updateLatencyChart(initialFiltered);
updateThroughputCharts(initialFiltered);
</script>
</body>
</html>"#,
    );

    fs::write(path, html).unwrap();
}
