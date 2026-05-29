use arche_types::attribute::{
    AffixAttribute, AttributePayload, BlueprintAttribute, BlueprintRefAttribute,
};
use arche_types::*;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::AuthenticatedKey;

#[derive(Debug)]
pub struct Cache {
    pub clients: HashMap<Uuid, Client>,
    pub blueprints: HashMap<Uuid, Blueprint>,
    pub affixes: HashMap<Uuid, Affix>,
    pub global_meta_attributes: HashMap<Uuid, GlobalMetaAttribute>,
    pub blueprint_affixes: HashMap<Uuid, Vec<BlueprintAffix>>,
    pub by_client: HashMap<Uuid, Arc<ClientCache>>,
}

#[derive(Debug)]
pub struct ClientCache {
    pub blueprints: Vec<Arc<Blueprint>>,
    pub affixes: Vec<Arc<Affix>>,
    pub global_meta_attributes: Vec<Arc<GlobalMetaAttribute>>,
    pub blueprint_resolved_attributes: HashMap<Uuid, HashMap<String, AttributePayload>>,
    pub affix_resolved_attributes: HashMap<Uuid, Option<(String, AttributePayload)>>,
}

impl Cache {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let clients = Self::load_clients(pool).await?;
        let blueprints = Self::load_blueprints(pool).await?;
        let affixes = Self::load_affixes(pool).await?;
        let global_meta_attributes = Self::load_global_meta_attributes(pool).await?;
        let blueprint_affixes = Self::load_blueprint_affixes(pool).await?;

        Ok(Cache::new(
            clients,
            blueprints,
            affixes,
            global_meta_attributes,
            blueprint_affixes,
        ))
    }

    pub fn get_client_data(&self, client_id: Uuid) -> Option<&Arc<ClientCache>> {
        self.by_client.get(&client_id)
    }

    pub fn new(
        clients: HashMap<Uuid, Client>,
        blueprints: HashMap<Uuid, Blueprint>,
        affixes: HashMap<Uuid, Affix>,
        global_meta_attributes: HashMap<Uuid, GlobalMetaAttribute>,
        blueprint_affixes: HashMap<Uuid, Vec<BlueprintAffix>>,
    ) -> Self {
        let by_client = Self::build_client_caches(
            &clients,
            &blueprints,
            &affixes,
            &global_meta_attributes,
        );
        Cache {
            clients,
            blueprints,
            affixes,
            global_meta_attributes,
            blueprint_affixes,
            by_client,
        }
    }

    async fn load_clients(pool: &PgPool) -> Result<HashMap<Uuid, Client>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, name, created_at, updated_at FROM clients")
            .fetch_all(pool)
            .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let name: String = row.get("name");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            map.insert(
                id,
                Client {
                    id,
                    name,
                    created_at,
                    updated_at,
                },
            );
        }
        Ok(map)
    }

    async fn load_blueprints(pool: &PgPool) -> Result<HashMap<Uuid, Blueprint>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, client_id, name, archetype, weight, description, \
             attributes, attribute_order, min_prefixes, max_prefixes, \
             min_suffixes, max_suffixes, created_at, updated_at \
             FROM blueprints",
        )
        .fetch_all(pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let client_id: Uuid = row.get("client_id");
            let name: String = row.get("name");
            let archetype: String = row.get("archetype");
            let weight: f64 = row.get("weight");
            let description: Option<String> = row.get("description");
            let attributes: serde_json::Value = row.get("attributes");
            let attribute_order: Vec<String> = row.get("attribute_order");
            let min_prefixes: i32 = row.get("min_prefixes");
            let max_prefixes: i32 = row.get("max_prefixes");
            let min_suffixes: i32 = row.get("min_suffixes");
            let max_suffixes: i32 = row.get("max_suffixes");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            map.insert(
                id,
                Blueprint {
                    id,
                    client_id,
                    name,
                    archetype,
                    weight,
                    description,
                    attributes,
                    attribute_order,
                    min_prefixes,
                    max_prefixes,
                    min_suffixes,
                    max_suffixes,
                    created_at,
                    updated_at,
                },
            );
        }
        Ok(map)
    }

    async fn load_affixes(pool: &PgPool) -> Result<HashMap<Uuid, Affix>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, client_id, name, type::TEXT AS type, description, attribute, created_at, updated_at \
             FROM affixes",
        )
        .fetch_all(pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let client_id: Uuid = row.get("client_id");
            let name: String = row.get("name");
            let type_str: String = row.get("type");
            let location = parse_affix_location(&type_str);
            let description: Option<String> = row.get("description");
            let attribute: serde_json::Value = row.get("attribute");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            map.insert(
                id,
                Affix {
                    id,
                    client_id,
                    name,
                    location,
                    description,
                    attribute,
                    created_at,
                    updated_at,
                },
            );
        }
        Ok(map)
    }

    async fn load_global_meta_attributes(
        pool: &PgPool,
    ) -> Result<HashMap<Uuid, GlobalMetaAttribute>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, client_id, name, description, value_type::TEXT AS value_type, payload, created_at, updated_at \
             FROM global_meta_attributes",
        )
        .fetch_all(pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let client_id: Uuid = row.get("client_id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let value_type_str: String = row.get("value_type");
            let value_type = parse_value_type(&value_type_str);
            let payload: serde_json::Value = row.get("payload");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            map.insert(
                id,
                GlobalMetaAttribute {
                    id,
                    client_id,
                    name,
                    description,
                    value_type,
                    payload,
                    created_at,
                    updated_at,
                },
            );
        }
        Ok(map)
    }

    async fn load_blueprint_affixes(
        pool: &PgPool,
    ) -> Result<HashMap<Uuid, Vec<BlueprintAffix>>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, blueprint_id, affix_id, weight, location::TEXT AS location, sort_order \
             FROM blueprint_affixes",
        )
        .fetch_all(pool)
        .await?;
        let mut map: HashMap<Uuid, Vec<BlueprintAffix>> = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let blueprint_id: Uuid = row.get("blueprint_id");
            let affix_id: Uuid = row.get("affix_id");
            let weight: f64 = row.get("weight");
            let location_str: String = row.get("location");
            let location = parse_affix_location(&location_str);
            let sort_order: i32 = row.get("sort_order");
            map.entry(blueprint_id).or_default().push(BlueprintAffix {
                id,
                blueprint_id,
                affix_id,
                weight,
                location,
                sort_order,
            });
        }
        Ok(map)
    }

    pub(crate) fn resolve_blueprint_attributes(
        blueprints: &[Arc<Blueprint>],
        gmas: &[Arc<GlobalMetaAttribute>],
    ) -> HashMap<Uuid, HashMap<String, AttributePayload>> {
        let mut result = HashMap::new();
        for bp in blueprints {
            let mut resolved = HashMap::new();
            let attrs = match bp.attributes.as_object() {
                Some(obj) => obj,
                None => {
                    result.insert(bp.id, resolved);
                    continue;
                }
            };
            for (key, attr_value) in attrs {
                let bp_attr: BlueprintAttribute = match serde_json::from_value(attr_value.clone()) {
                    Ok(a) => a,
                    Err(_) => continue,
                };
                match bp_attr {
                    BlueprintAttribute::Ref(BlueprintRefAttribute { ref_id }) => {
                        if let Some(gma) = gmas.iter().find(|g| g.id == ref_id) {
                            if let Ok(payload) = serde_json::from_value(gma.payload.clone()) {
                                resolved.insert(key.clone(), payload);
                            }
                        }
                    }
                    BlueprintAttribute::Inline(inline_def) => {
                        resolved.insert(key.clone(), inline_def.payload);
                    }
                }
            }
            result.insert(bp.id, resolved);
        }
        result
    }

    pub(crate) fn resolve_affix_attributes(
        affixes: &[Arc<Affix>],
        gmas: &[Arc<GlobalMetaAttribute>],
    ) -> HashMap<Uuid, Option<(String, AttributePayload)>> {
        let mut result = HashMap::new();
        for affix in affixes {
            let resolved = Self::resolve_single_affix_attribute(affix, gmas);
            result.insert(affix.id, resolved);
        }
        result
    }

    pub(crate) fn resolve_single_affix_attribute(
        affix: &Affix,
        gmas: &[Arc<GlobalMetaAttribute>],
    ) -> Option<(String, AttributePayload)> {
        let affix_attr: AffixAttribute = serde_json::from_value(affix.attribute.clone()).ok()?;
        match affix_attr {
            AffixAttribute::Inline(inline_def) => Some((inline_def.name, inline_def.payload)),
            AffixAttribute::Ref { ref_id } => {
                let gma = gmas.iter().find(|g| g.id == ref_id)?;
                let payload: AttributePayload = serde_json::from_value(gma.payload.clone()).ok()?;
                Some((gma.name.clone(), payload))
            }
        }
    }

    fn build_client_caches(
        clients: &HashMap<Uuid, Client>,
        blueprints: &HashMap<Uuid, Blueprint>,
        affixes: &HashMap<Uuid, Affix>,
        global_meta_attributes: &HashMap<Uuid, GlobalMetaAttribute>,
    ) -> HashMap<Uuid, Arc<ClientCache>> {
        let mut by_client: HashMap<Uuid, ClientCache> = clients
            .keys()
            .map(|&id| {
                (
                    id,
                    ClientCache {
                        blueprints: Vec::new(),
                        affixes: Vec::new(),
                        global_meta_attributes: Vec::new(),
                        blueprint_resolved_attributes: HashMap::new(),
                        affix_resolved_attributes: HashMap::new(),
                    },
                )
            })
            .collect();

        for bp in blueprints.values() {
            if let Some(cc) = by_client.get_mut(&bp.client_id) {
                cc.blueprints.push(Arc::new(bp.clone()));
            }
        }

        for affix in affixes.values() {
            if let Some(cc) = by_client.get_mut(&affix.client_id) {
                cc.affixes.push(Arc::new(affix.clone()));
            }
        }

        for gma in global_meta_attributes.values() {
            if let Some(cc) = by_client.get_mut(&gma.client_id) {
                cc.global_meta_attributes.push(Arc::new(gma.clone()));
            }
        }

        for (_, cc) in by_client.iter_mut() {
            cc.blueprint_resolved_attributes =
                Self::resolve_blueprint_attributes(&cc.blueprints, &cc.global_meta_attributes);
            cc.affix_resolved_attributes =
                Self::resolve_affix_attributes(&cc.affixes, &cc.global_meta_attributes);
        }

        by_client.into_iter().map(|(k, v)| (k, Arc::new(v))).collect()
    }
}

fn parse_affix_location(s: &str) -> AffixLocation {
    match s {
        "prefix" => AffixLocation::Prefix,
        "suffix" => AffixLocation::Suffix,
        other => panic!("unexpected affix_location: {}", other),
    }
}

fn parse_value_type(s: &str) -> ValueType {
    match s {
        "single" => ValueType::Single,
        "enum" => ValueType::Enum,
        "range" => ValueType::Range,
        "string" => ValueType::String,
        "boolean" => ValueType::Boolean,
        other => panic!("unexpected value_type: {}", other),
    }
}

pub struct FetchedClientData {
    pub client: Option<Client>,
    pub blueprints: HashMap<Uuid, Blueprint>,
    pub affixes: HashMap<Uuid, Affix>,
    pub global_meta_attributes: HashMap<Uuid, GlobalMetaAttribute>,
    pub blueprint_affixes: HashMap<Uuid, Vec<BlueprintAffix>>,
}

#[derive(Debug, Clone)]
struct CachedKeyEntry {
    key: AuthenticatedKey,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ApiKeyCache {
    entries: RwLock<HashMap<String, CachedKeyEntry>>,
}

impl ApiKeyCache {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, raw_key: &str) -> Option<AuthenticatedKey> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(raw_key) {
            if let Some(expires_at) = entry.expires_at {
                if Utc::now() > expires_at {
                    drop(entries);
                    self.entries.write().await.remove(raw_key);
                    return None;
                }
            }
            Some(entry.key.clone())
        } else {
            None
        }
    }

    pub async fn insert(
        &self,
        raw_key: String,
        key: AuthenticatedKey,
        expires_at: Option<DateTime<Utc>>,
    ) {
        self.entries
            .write()
            .await
            .insert(raw_key, CachedKeyEntry { key, expires_at });
    }

    async fn load_all_key_ids(pool: &PgPool) -> Result<HashMap<Uuid, Option<DateTime<Utc>>>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, expires_at FROM api_keys")
            .fetch_all(pool)
            .await?;
        Ok(rows
            .iter()
            .map(|r| {
                let id: Uuid = r.get("id");
                let expires_at: Option<DateTime<Utc>> = r.get("expires_at");
                (id, expires_at)
            })
            .collect())
    }

    pub async fn sync_from_db(&self, pool: &PgPool) {
        let valid_ids = match Self::load_all_key_ids(pool).await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!(error = %e, "api_key_cache poll: failed to load key ids");
                return;
            }
        };

        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| match valid_ids.get(&entry.key.id) {
            None => false,
            Some(Some(exp)) => Utc::now() <= *exp,
            Some(None) => true,
        });
    }
}

pub fn start_api_key_cache_poller(
    pool: PgPool,
    api_key_cache: Arc<ApiKeyCache>,
    interval_ms: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval = tokio::time::Duration::from_millis(interval_ms);
        let mut tick = tokio::time::interval(interval);
        tick.tick().await;

        loop {
            tick.tick().await;
            api_key_cache.sync_from_db(&pool).await;
        }
    })
}

pub fn start_cache_poller(
    pool: PgPool,
    cache: Arc<RwLock<Cache>>,
    interval_ms: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval = tokio::time::Duration::from_millis(interval_ms);
        let mut tick = tokio::time::interval(interval);
        let mut last_seen: HashMap<Uuid, DateTime<Utc>> = HashMap::new();

        tick.tick().await;

        loop {
            tick.tick().await;

            let db_client_ids = match sqlx::query("SELECT id FROM clients")
                .fetch_all(&pool)
                .await
            {
                Ok(rows) => rows.iter().map(|r| r.get::<Uuid, _>("id")).collect::<Vec<_>>(),
                Err(e) => {
                    tracing::warn!(error = %e, "cache poll: failed to list clients");
                    continue;
                }
            };

            let cache_client_ids: Vec<Uuid> = {
                let cache = cache.read().await;
                cache.clients.keys().copied().collect()
            };

            let mut all_client_ids = cache_client_ids;
            for id in &db_client_ids {
                if !all_client_ids.contains(id) {
                    all_client_ids.push(*id);
                }
            }

            for client_id in &all_client_ids {
                if !db_client_ids.contains(client_id) {
                    let mut cache = cache.write().await;
                    cache.apply_client_data(
                        *client_id,
                        FetchedClientData {
                            client: None,
                            blueprints: HashMap::new(),
                            affixes: HashMap::new(),
                            global_meta_attributes: HashMap::new(),
                            blueprint_affixes: HashMap::new(),
                        },
                    );
                    last_seen.remove(client_id);
                    tracing::info!(client_id = %client_id, "cache poll: removed deleted client");
                    continue;
                }

                let max_ts = match Cache::query_max_updated_at_for_client(&pool, *client_id)
                    .await
                {
                    Ok(ts) => ts,
                    Err(e) => {
                        tracing::warn!(error = %e, client_id = %client_id, "cache poll: query failed");
                        continue;
                    }
                };

                if has_cache_changed(max_ts, last_seen.get(client_id).copied()) {
                    tracing::info!(
                        client_id = %client_id,
                        last_seen = ?last_seen.get(client_id),
                        new_max = ?max_ts,
                        "cache poll: change detected, reloading client"
                    );
                    match Cache::fetch_client_data(&pool, *client_id).await {
                        Ok(data) => {
                            let mut cache = cache.write().await;
                            cache.apply_client_data(*client_id, data);
                        }
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                client_id = %client_id,
                                "cache poll: failed to reload client data"
                            );
                            continue;
                        }
                    }
                    if let Some(ts) = max_ts {
                        last_seen.insert(*client_id, ts);
                    }
                }
            }
        }
    })
}

fn has_cache_changed(current_max: Option<DateTime<Utc>>, last_seen: Option<DateTime<Utc>>) -> bool {
    current_max != last_seen
}

impl Cache {
    pub async fn fetch_client_data(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<FetchedClientData, sqlx::Error> {
        let client_row = sqlx::query("SELECT id, name, created_at, updated_at FROM clients WHERE id = $1")
            .bind(client_id)
            .fetch_optional(pool)
            .await?;

        let blueprints = Self::load_blueprints_for_client(pool, client_id).await?;
        let affixes = Self::load_affixes_for_client(pool, client_id).await?;
        let global_meta_attributes =
            Self::load_global_meta_attributes_for_client(pool, client_id).await?;
        let blueprint_affixes =
            Self::load_blueprint_affixes_for_client(pool, client_id).await?;

        let client = client_row.map(|row| Client {
            id: row.get("id"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });

        Ok(FetchedClientData {
            client,
            blueprints,
            affixes,
            global_meta_attributes,
            blueprint_affixes,
        })
    }

    pub fn apply_client_data(&mut self, client_id: Uuid, data: FetchedClientData) {
        let old_bp_ids: Vec<Uuid> = self
            .blueprints
            .iter()
            .filter(|(_, bp)| bp.client_id == client_id)
            .map(|(id, _)| *id)
            .collect();
        for id in &old_bp_ids {
            self.blueprint_affixes.remove(id);
            self.blueprints.remove(id);
        }
        self.affixes.retain(|_, a| a.client_id != client_id);
        self.global_meta_attributes
            .retain(|_, gma| gma.client_id != client_id);

        match data.client {
            Some(client) => {
                let blueprints: Vec<Arc<Blueprint>> =
                    data.blueprints.values().map(|bp| Arc::new(bp.clone())).collect();
                let affixes: Vec<Arc<Affix>> =
                    data.affixes.values().map(|a| Arc::new(a.clone())).collect();
                let gmas: Vec<Arc<GlobalMetaAttribute>> = data
                    .global_meta_attributes
                    .values()
                    .map(|gma| Arc::new(gma.clone()))
                    .collect();
                let blueprint_resolved_attributes =
                    Self::resolve_blueprint_attributes(&blueprints, &gmas);
                let affix_resolved_attributes =
                    Self::resolve_affix_attributes(&affixes, &gmas);
                let cc = ClientCache {
                    blueprints,
                    affixes,
                    global_meta_attributes: gmas,
                    blueprint_resolved_attributes,
                    affix_resolved_attributes,
                };
                self.blueprints.extend(data.blueprints);
                self.affixes.extend(data.affixes);
                self.global_meta_attributes.extend(data.global_meta_attributes);
                self.blueprint_affixes.extend(data.blueprint_affixes);
                self.clients.insert(client_id, client);
                self.by_client.insert(client_id, Arc::new(cc));
            }
            None => {
                self.clients.remove(&client_id);
                self.by_client.remove(&client_id);
            }
        }
    }

    async fn load_blueprints_for_client(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<HashMap<Uuid, Blueprint>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, client_id, name, archetype, weight, description, \
             attributes, attribute_order, min_prefixes, max_prefixes, \
             min_suffixes, max_suffixes, created_at, updated_at \
             FROM blueprints WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            map.insert(
                id,
                Blueprint {
                    id,
                    client_id,
                    name: row.get("name"),
                    archetype: row.get("archetype"),
                    weight: row.get("weight"),
                    description: row.get("description"),
                    attributes: row.get("attributes"),
                    attribute_order: row.get("attribute_order"),
                    min_prefixes: row.get("min_prefixes"),
                    max_prefixes: row.get("max_prefixes"),
                    min_suffixes: row.get("min_suffixes"),
                    max_suffixes: row.get("max_suffixes"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            );
        }
        Ok(map)
    }

    async fn load_affixes_for_client(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<HashMap<Uuid, Affix>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, client_id, name, type::TEXT AS type, description, attribute, created_at, updated_at \
             FROM affixes WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let type_str: String = row.get("type");
            map.insert(
                id,
                Affix {
                    id,
                    client_id,
                    name: row.get("name"),
                    location: parse_affix_location(&type_str),
                    description: row.get("description"),
                    attribute: row.get("attribute"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            );
        }
        Ok(map)
    }

    async fn load_global_meta_attributes_for_client(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<HashMap<Uuid, GlobalMetaAttribute>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, client_id, name, description, value_type::TEXT AS value_type, payload, created_at, updated_at \
             FROM global_meta_attributes WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;
        let mut map = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let value_type_str: String = row.get("value_type");
            map.insert(
                id,
                GlobalMetaAttribute {
                    id,
                    client_id,
                    name: row.get("name"),
                    description: row.get("description"),
                    value_type: parse_value_type(&value_type_str),
                    payload: row.get("payload"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                },
            );
        }
        Ok(map)
    }

    async fn load_blueprint_affixes_for_client(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<HashMap<Uuid, Vec<BlueprintAffix>>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT ba.id, ba.blueprint_id, ba.affix_id, ba.weight, ba.location::TEXT AS location, ba.sort_order \
             FROM blueprint_affixes ba \
             JOIN blueprints b ON ba.blueprint_id = b.id \
             WHERE b.client_id = $1",
        )
        .bind(client_id)
        .fetch_all(pool)
        .await?;
        let mut map: HashMap<Uuid, Vec<BlueprintAffix>> = HashMap::new();
        for row in rows {
            let blueprint_id: Uuid = row.get("blueprint_id");
            let location_str: String = row.get("location");
            map.entry(blueprint_id).or_default().push(BlueprintAffix {
                id: row.get("id"),
                blueprint_id,
                affix_id: row.get("affix_id"),
                weight: row.get("weight"),
                location: parse_affix_location(&location_str),
                sort_order: row.get("sort_order"),
            });
        }
        Ok(map)
    }

    pub async fn query_max_updated_at_for_client(
        pool: &PgPool,
        client_id: Uuid,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT GREATEST( \
                COALESCE((SELECT max(updated_at) FROM blueprints WHERE client_id = $1), '1970-01-01'::timestamptz), \
                COALESCE((SELECT max(updated_at) FROM affixes WHERE client_id = $1), '1970-01-01'::timestamptz), \
                COALESCE((SELECT max(updated_at) FROM global_meta_attributes WHERE client_id = $1), '1970-01-01'::timestamptz), \
                COALESCE((SELECT updated_at FROM clients WHERE id = $1), '1970-01-01'::timestamptz) \
            )",
        )
        .bind(client_id)
        .fetch_optional(pool)
        .await?;
        Ok(row.and_then(|r| r.get(0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_client(id: Uuid, name: &str) -> Client {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        Client {
            id,
            name: name.into(),
            created_at: ts,
            updated_at: ts,
        }
    }

    fn make_blueprint(id: Uuid, client_id: Uuid, name: &str) -> Blueprint {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        Blueprint {
            id,
            client_id,
            name: name.into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({}),
            attribute_order: vec![],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: ts,
            updated_at: ts,
        }
    }

    fn make_affix(id: Uuid, client_id: Uuid, name: &str, location: AffixLocation) -> Affix {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        Affix {
            id,
            client_id,
            name: name.into(),
            location,
            description: None,
            attribute: serde_json::json!({}),
            created_at: ts,
            updated_at: ts,
        }
    }

    fn make_gma(id: Uuid, client_id: Uuid, name: &str) -> GlobalMetaAttribute {
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        GlobalMetaAttribute {
            id,
            client_id,
            name: name.into(),
            description: None,
            value_type: ValueType::Range,
            payload: serde_json::json!({"min": 1.0, "max": 10.0}),
            created_at: ts,
            updated_at: ts,
        }
    }

    #[test]
    fn test_empty_cache() {
        let cache = Cache::new(
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        assert!(cache.get_client_data(Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_get_client_data_returns_correct_client_cache() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();
        let gma_id = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "test-game"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp_id, make_blueprint(bp_id, client_id, "Longsword"));

        let mut affixes = HashMap::new();
        affixes.insert(affix_id, make_affix(affix_id, client_id, "Fire", AffixLocation::Prefix));

        let mut gmas = HashMap::new();
        gmas.insert(gma_id, make_gma(gma_id, client_id, "damage"));

        let cache = Cache::new(clients, blueprints, affixes, gmas, HashMap::new());

        let cc = cache.get_client_data(client_id).expect("client cache should exist");
        assert_eq!(cc.blueprints.len(), 1);
        assert_eq!(cc.blueprints[0].id, bp_id);
        assert_eq!(cc.affixes.len(), 1);
        assert_eq!(cc.affixes[0].id, affix_id);
        assert_eq!(cc.global_meta_attributes.len(), 1);
        assert_eq!(cc.global_meta_attributes[0].id, gma_id);
    }

    #[test]
    fn test_multiple_clients_isolated() {
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();

        let bp1 = Uuid::new_v4();
        let bp2 = Uuid::new_v4();
        let aff1 = Uuid::new_v4();
        let aff2 = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(c1, make_client(c1, "game-1"));
        clients.insert(c2, make_client(c2, "game-2"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp1, make_blueprint(bp1, c1, "Sword"));
        blueprints.insert(bp2, make_blueprint(bp2, c2, "Axe"));

        let mut affixes = HashMap::new();
        affixes.insert(aff1, make_affix(aff1, c1, "Fire", AffixLocation::Prefix));
        affixes.insert(aff2, make_affix(aff2, c2, "Ice", AffixLocation::Suffix));

        let cache = Cache::new(clients, blueprints, affixes, HashMap::new(), HashMap::new());

        let cc1 = cache.get_client_data(c1).unwrap();
        let cc2 = cache.get_client_data(c2).unwrap();

        assert_eq!(cc1.blueprints.len(), 1);
        assert_eq!(cc1.blueprints[0].id, bp1);
        assert_eq!(cc1.affixes.len(), 1);
        assert_eq!(cc1.affixes[0].id, aff1);
        assert_eq!(cc1.global_meta_attributes.len(), 0);

        assert_eq!(cc2.blueprints.len(), 1);
        assert_eq!(cc2.blueprints[0].id, bp2);
        assert_eq!(cc2.affixes.len(), 1);
        assert_eq!(cc2.affixes[0].id, aff2);
        assert_eq!(cc2.global_meta_attributes.len(), 0);
    }

    #[test]
    fn test_client_cache_arc_is_independent() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "test"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp_id, make_blueprint(bp_id, client_id, "Longsword"));

        let cache = Cache::new(clients, blueprints, HashMap::new(), HashMap::new(), HashMap::new());

        let cc = cache.get_client_data(client_id).unwrap();
        assert_eq!(Arc::strong_count(&cc.blueprints[0]), 1);
    }

    #[test]
    fn test_cache_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Cache>();
        assert_send_sync::<ClientCache>();
    }

    #[test]
    fn test_parse_affix_location() {
        assert_eq!(parse_affix_location("prefix"), AffixLocation::Prefix);
        assert_eq!(parse_affix_location("suffix"), AffixLocation::Suffix);
    }

    #[test]
    #[should_panic(expected = "unexpected affix_location")]
    fn test_parse_affix_location_invalid() {
        parse_affix_location("invalid");
    }

    #[test]
    fn test_parse_value_type() {
        assert_eq!(parse_value_type("single"), ValueType::Single);
        assert_eq!(parse_value_type("enum"), ValueType::Enum);
        assert_eq!(parse_value_type("range"), ValueType::Range);
        assert_eq!(parse_value_type("string"), ValueType::String);
        assert_eq!(parse_value_type("boolean"), ValueType::Boolean);
    }

    #[test]
    #[should_panic(expected = "unexpected value_type")]
    fn test_parse_value_type_invalid() {
        parse_value_type("invalid");
    }

    #[test]
    fn test_client_with_no_resources_has_empty_vectors() {
        let client_id = Uuid::new_v4();
        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "empty-game"));

        let cache = Cache::new(clients, HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new());

        let cc = cache.get_client_data(client_id).unwrap();
        assert!(cc.blueprints.is_empty());
        assert!(cc.affixes.is_empty());
        assert!(cc.global_meta_attributes.is_empty());
    }

    fn make_newer_blueprint(id: Uuid, client_id: Uuid, name: &str) -> Blueprint {
        let ts = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        Blueprint {
            id,
            client_id,
            name: name.into(),
            archetype: "axe".into(),
            weight: 2.0,
            description: Some("newer".into()),
            attributes: serde_json::json!({"sharpness": 5}),
            attribute_order: vec!["sharpness".into()],
            min_prefixes: 1,
            max_prefixes: 2,
            min_suffixes: 0,
            max_suffixes: 1,
            created_at: ts,
            updated_at: ts,
        }
    }

    #[test]
    fn test_apply_client_data_replaces_blueprints() {
        let client_id = Uuid::new_v4();
        let old_bp_id = Uuid::new_v4();
        let new_bp1_id = Uuid::new_v4();
        let new_bp2_id = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "game"));

        let mut blueprints = HashMap::new();
        blueprints.insert(old_bp_id, make_blueprint(old_bp_id, client_id, "OldSword"));

        let mut cache = Cache::new(clients, blueprints, HashMap::new(), HashMap::new(), HashMap::new());
        assert_eq!(cache.by_client[&client_id].blueprints.len(), 1);

        let mut new_blueprints = HashMap::new();
        new_blueprints.insert(new_bp1_id, make_newer_blueprint(new_bp1_id, client_id, "NewAxe"));
        new_blueprints.insert(new_bp2_id, make_newer_blueprint(new_bp2_id, client_id, "NewMace"));

        cache.apply_client_data(
            client_id,
            FetchedClientData {
                client: Some(make_client(client_id, "game")),
                blueprints: new_blueprints,
                affixes: HashMap::new(),
                global_meta_attributes: HashMap::new(),
                blueprint_affixes: HashMap::new(),
            },
        );

        let cc = cache.get_client_data(client_id).unwrap();
        assert_eq!(cc.blueprints.len(), 2);
        let names: Vec<&str> = cc.blueprints.iter().map(|bp| bp.name.as_str()).collect();
        assert!(names.contains(&"NewAxe"));
        assert!(names.contains(&"NewMace"));
        assert!(!names.contains(&"OldSword"));
        assert!(!cache.blueprints.contains_key(&old_bp_id));
    }

    #[test]
    fn test_apply_client_data_clears_all_resources() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();
        let gma_id = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "game"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp_id, make_blueprint(bp_id, client_id, "Sword"));

        let mut affixes = HashMap::new();
        affixes.insert(affix_id, make_affix(affix_id, client_id, "Fire", AffixLocation::Prefix));

        let mut gmas = HashMap::new();
        gmas.insert(gma_id, make_gma(gma_id, client_id, "damage"));

        let mut cache = Cache::new(clients, blueprints, affixes, gmas, HashMap::new());

        assert_eq!(cache.by_client[&client_id].blueprints.len(), 1);
        assert_eq!(cache.by_client[&client_id].affixes.len(), 1);
        assert_eq!(cache.by_client[&client_id].global_meta_attributes.len(), 1);

        cache.apply_client_data(
            client_id,
            FetchedClientData {
                client: Some(make_client(client_id, "game")),
                blueprints: HashMap::new(),
                affixes: HashMap::new(),
                global_meta_attributes: HashMap::new(),
                blueprint_affixes: HashMap::new(),
            },
        );

        let cc = cache.get_client_data(client_id).unwrap();
        assert!(cc.blueprints.is_empty());
        assert!(cc.affixes.is_empty());
        assert!(cc.global_meta_attributes.is_empty());
    }

    #[test]
    fn test_apply_client_data_preserves_other_clients() {
        let client1 = Uuid::new_v4();
        let client2 = Uuid::new_v4();

        let bp1 = Uuid::new_v4();
        let bp2 = Uuid::new_v4();
        let new_bp2 = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(client1, make_client(client1, "game1"));
        clients.insert(client2, make_client(client2, "game2"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp1, make_blueprint(bp1, client1, "Sword1"));
        blueprints.insert(bp2, make_blueprint(bp2, client2, "Sword2"));

        let mut cache = Cache::new(clients, blueprints, HashMap::new(), HashMap::new(), HashMap::new());

        let mut new_blueprints_c2 = HashMap::new();
        new_blueprints_c2.insert(new_bp2, make_newer_blueprint(new_bp2, client2, "Axe2"));

        cache.apply_client_data(
            client2,
            FetchedClientData {
                client: Some(make_client(client2, "game2")),
                blueprints: new_blueprints_c2,
                affixes: HashMap::new(),
                global_meta_attributes: HashMap::new(),
                blueprint_affixes: HashMap::new(),
            },
        );

        let cc1 = cache.get_client_data(client1).unwrap();
        assert_eq!(cc1.blueprints.len(), 1);
        assert_eq!(cc1.blueprints[0].id, bp1);
        assert_eq!(cc1.blueprints[0].name, "Sword1");

        let cc2 = cache.get_client_data(client2).unwrap();
        assert_eq!(cc2.blueprints.len(), 1);
        assert_eq!(cc2.blueprints[0].id, new_bp2);
        assert_eq!(cc2.blueprints[0].name, "Axe2");
    }

    #[test]
    fn test_apply_client_data_removes_deleted_client() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "game"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp_id, make_blueprint(bp_id, client_id, "Sword"));

        let mut cache = Cache::new(clients, blueprints, HashMap::new(), HashMap::new(), HashMap::new());
        assert!(cache.get_client_data(client_id).is_some());

        cache.apply_client_data(
            client_id,
            FetchedClientData {
                client: None,
                blueprints: HashMap::new(),
                affixes: HashMap::new(),
                global_meta_attributes: HashMap::new(),
                blueprint_affixes: HashMap::new(),
            },
        );

        assert!(cache.get_client_data(client_id).is_none());
        assert!(!cache.clients.contains_key(&client_id));
    }

    #[test]
    fn test_apply_client_data_updates_blueprint_affixes() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();
        let ba_id = Uuid::new_v4();

        let mut clients = HashMap::new();
        clients.insert(client_id, make_client(client_id, "game"));

        let mut blueprints = HashMap::new();
        blueprints.insert(bp_id, make_blueprint(bp_id, client_id, "Sword"));

        let mut affixes = HashMap::new();
        affixes.insert(affix_id, make_affix(affix_id, client_id, "Fire", AffixLocation::Prefix));

        let mut bp_affixes = HashMap::new();
        bp_affixes.insert(
            bp_id,
            vec![BlueprintAffix {
                id: ba_id,
                blueprint_id: bp_id,
                affix_id,
                weight: 1.0,
                location: AffixLocation::Prefix,
                sort_order: 0,
            }],
        );

        let mut cache = Cache::new(clients, blueprints, affixes, HashMap::new(), bp_affixes);
        assert_eq!(cache.blueprint_affixes.get(&bp_id).unwrap().len(), 1);

        let mut new_bp_affixes = HashMap::new();
        new_bp_affixes.insert(bp_id, vec![]);

        cache.apply_client_data(
            client_id,
            FetchedClientData {
                client: Some(make_client(client_id, "game")),
                blueprints: {
                    let mut m = HashMap::new();
                    m.insert(bp_id, make_blueprint(bp_id, client_id, "Sword"));
                    m
                },
                affixes: {
                    let mut m = HashMap::new();
                    m.insert(affix_id, make_affix(affix_id, client_id, "Fire", AffixLocation::Prefix));
                    m
                },
                global_meta_attributes: HashMap::new(),
                blueprint_affixes: new_bp_affixes,
            },
        );

        assert_eq!(cache.blueprint_affixes.get(&bp_id).unwrap().len(), 0);
    }

    #[test]
    fn test_has_cache_changed_no_change() {
        let ts = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert!(!has_cache_changed(Some(ts), Some(ts)));
    }

    #[test]
    fn test_has_cache_changed_newer_timestamp() {
        let old = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let new = old + chrono::Duration::seconds(10);
        assert!(has_cache_changed(Some(new), Some(old)));
    }

    #[test]
    fn test_has_cache_changed_older_timestamp() {
        let old = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let new = old + chrono::Duration::seconds(10);
        assert!(has_cache_changed(Some(old), Some(new)));
    }

    #[test]
    fn test_has_cache_changed_first_poll() {
        let ts = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert!(has_cache_changed(Some(ts), None));
    }

    #[test]
    fn test_has_cache_changed_cleared_timestamp() {
        let ts = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        assert!(has_cache_changed(None, Some(ts)));
    }

    #[test]
    fn test_has_cache_changed_both_none() {
        assert!(!has_cache_changed(None, None));
    }

    mod db_tests {
        use super::*;
        use serde_json::json;
        use sqlx::PgPool;

        async fn ensure_db() -> Option<PgPool> {
            let url = std::env::var("DATABASE_URL").ok()?;
            let pool = PgPool::connect(&url).await.ok()?;
            sqlx::migrate!("../../arche-service/migrations")
                .run(&pool)
                .await
                .ok()?;
            Some(pool)
        }

        async fn create_test_client(pool: &PgPool) -> Uuid {
            let row = sqlx::query("INSERT INTO clients (name) VALUES ('cache-test-client') RETURNING id")
                .fetch_one(pool)
                .await
                .expect("Failed to create test client");
            row.get("id")
        }

        #[tokio::test]
        async fn test_load_affixes_with_custom_enum_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO affixes (client_id, name, type, attribute) \
                 VALUES ($1, 'test-prefix', 'prefix'::affix_location, $2)",
            )
            .bind(client_id)
            .bind(&json!({"value_type": "single", "value": 1.0}))
            .execute(&pool)
            .await
            .expect("Failed to insert test affix");

            sqlx::query(
                "INSERT INTO affixes (client_id, name, type, attribute) \
                 VALUES ($1, 'test-suffix', 'suffix'::affix_location, $2)",
            )
            .bind(client_id)
            .bind(&json!({"value_type": "single", "value": 2.0}))
            .execute(&pool)
            .await
            .expect("Failed to insert test affix");

            let affixes = Cache::load_affixes(&pool).await.expect("load_affixes should not panic");
            let client_affixes: Vec<_> = affixes.values().filter(|a| a.client_id == client_id).collect();
            assert_eq!(client_affixes.len(), 2);
            let names: Vec<&str> = client_affixes.iter().map(|a| a.name.as_str()).collect();
            assert!(names.contains(&"test-prefix"));
            assert!(names.contains(&"test-suffix"));
            let locations: Vec<&AffixLocation> = client_affixes.iter().map(|a| &a.location).collect();
            assert!(locations.contains(&&AffixLocation::Prefix));
            assert!(locations.contains(&&AffixLocation::Suffix));
        }

        #[tokio::test]
        async fn test_load_global_meta_attributes_with_custom_enum_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, description, value_type, payload) \
                 VALUES ($1, 'gma-range', 'a range attr', 'range'::value_type, $2)",
            )
            .bind(client_id)
            .bind(&json!({"min": 1.0, "max": 10.0}))
            .execute(&pool)
            .await
            .expect("Failed to insert test GMA");

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, description, value_type, payload) \
                 VALUES ($1, 'gma-enum', 'an enum attr', 'enum'::value_type, $2)",
            )
            .bind(client_id)
            .bind(&json!({"values": ["a", "b", "c"]}))
            .execute(&pool)
            .await
            .expect("Failed to insert test GMA");

            let gmas = Cache::load_global_meta_attributes(&pool)
                .await
                .expect("load_global_meta_attributes should not panic");
            let client_gmas: Vec<_> = gmas.values().filter(|g| g.client_id == client_id).collect();
            assert_eq!(client_gmas.len(), 2);
            let names: Vec<&str> = client_gmas.iter().map(|g| g.name.as_str()).collect();
            assert!(names.contains(&"gma-range"));
            assert!(names.contains(&"gma-enum"));
            let types: Vec<&ValueType> = client_gmas.iter().map(|g| &g.value_type).collect();
            assert!(types.contains(&&ValueType::Range));
            assert!(types.contains(&&ValueType::Enum));
        }

        #[tokio::test]
        async fn test_load_affixes_for_client_with_custom_enum_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO affixes (client_id, name, type, attribute) \
                 VALUES ($1, 'client-prefix', 'prefix'::affix_location, $2)",
            )
            .bind(client_id)
            .bind(&json!({"value_type": "single", "value": 1.0}))
            .execute(&pool)
            .await
            .expect("Failed to insert test affix");

            let affixes = Cache::load_affixes_for_client(&pool, client_id)
                .await
                .expect("load_affixes_for_client should not panic");
            assert_eq!(affixes.len(), 1);
            let affix = affixes.values().next().unwrap();
            assert_eq!(affix.name, "client-prefix");
            assert_eq!(affix.location, AffixLocation::Prefix);
        }

        #[tokio::test]
        async fn test_load_global_meta_attributes_for_client_with_custom_enum_type() {
            let pool = match ensure_db().await {
                Some(p) => p,
                None => return,
            };
            let client_id = create_test_client(&pool).await;

            sqlx::query(
                "INSERT INTO global_meta_attributes (client_id, name, description, value_type, payload) \
                 VALUES ($1, 'client-gma', 'desc', 'boolean'::value_type, $2)",
            )
            .bind(client_id)
            .bind(&json!({"value": true}))
            .execute(&pool)
            .await
            .expect("Failed to insert test GMA");

            let gmas = Cache::load_global_meta_attributes_for_client(&pool, client_id)
                .await
                .expect("load_global_meta_attributes_for_client should not panic");
            assert_eq!(gmas.len(), 1);
            let gma = gmas.values().next().unwrap();
            assert_eq!(gma.name, "client-gma");
            assert_eq!(gma.value_type, ValueType::Boolean);
        }
    }
}
