use arche_types::*;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct Cache {
    pub clients: HashMap<Uuid, Client>,
    pub blueprints: HashMap<Uuid, Blueprint>,
    pub affixes: HashMap<Uuid, Affix>,
    pub global_meta_attributes: HashMap<Uuid, GlobalMetaAttribute>,
    pub blueprint_affixes: HashMap<Uuid, Vec<BlueprintAffix>>,
    pub by_client: HashMap<Uuid, ClientCache>,
}

#[derive(Debug)]
pub struct ClientCache {
    pub blueprints: Vec<Arc<Blueprint>>,
    pub affixes: Vec<Arc<Affix>>,
    pub global_meta_attributes: Vec<Arc<GlobalMetaAttribute>>,
}

impl Cache {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let clients = Self::load_clients(pool).await?;
        let blueprints = Self::load_blueprints(pool).await?;
        let affixes = Self::load_affixes(pool).await?;
        let global_meta_attributes = Self::load_global_meta_attributes(pool).await?;
        let blueprint_affixes = Self::load_blueprint_affixes(pool).await?;

        let by_client = Self::build_client_caches(
            &clients,
            &blueprints,
            &affixes,
            &global_meta_attributes,
        );

        Ok(Cache {
            clients,
            blueprints,
            affixes,
            global_meta_attributes,
            blueprint_affixes,
            by_client,
        })
    }

    pub fn get_client_data(&self, client_id: Uuid) -> Option<&ClientCache> {
        self.by_client.get(&client_id)
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
            "SELECT id, client_id, name, type, description, attribute, created_at, updated_at \
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
            "SELECT id, client_id, name, description, value_type, payload, created_at, updated_at \
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
            "SELECT id, blueprint_id, affix_id, weight, location, sort_order \
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

    fn build_client_caches(
        clients: &HashMap<Uuid, Client>,
        blueprints: &HashMap<Uuid, Blueprint>,
        affixes: &HashMap<Uuid, Affix>,
        global_meta_attributes: &HashMap<Uuid, GlobalMetaAttribute>,
    ) -> HashMap<Uuid, ClientCache> {
        let mut by_client: HashMap<Uuid, ClientCache> = clients
            .keys()
            .map(|&id| {
                (
                    id,
                    ClientCache {
                        blueprints: Vec::new(),
                        affixes: Vec::new(),
                        global_meta_attributes: Vec::new(),
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

        by_client
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::collections::HashMap;

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
        let cache = Cache {
            clients: HashMap::new(),
            blueprints: HashMap::new(),
            affixes: HashMap::new(),
            global_meta_attributes: HashMap::new(),
            blueprint_affixes: HashMap::new(),
            by_client: HashMap::new(),
        };
        assert!(cache.get_client_data(Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_get_client_data_returns_correct_client_cache() {
        let client_id = Uuid::new_v4();
        let bp_id = Uuid::new_v4();
        let affix_id = Uuid::new_v4();
        let gma_id = Uuid::new_v4();

        let client = make_client(client_id, "test-game");
        let bp = make_blueprint(bp_id, client_id, "Longsword");
        let affix = make_affix(affix_id, client_id, "Fire", AffixLocation::Prefix);
        let gma = make_gma(gma_id, client_id, "damage");

        let mut clients = HashMap::new();
        clients.insert(client_id, client);

        let mut blueprints = HashMap::new();
        blueprints.insert(bp_id, bp);

        let mut affixes = HashMap::new();
        affixes.insert(affix_id, affix);

        let mut gmas = HashMap::new();
        gmas.insert(gma_id, gma);

        let by_client = Cache::build_client_caches(&clients, &blueprints, &affixes, &gmas);

        let cache = Cache {
            clients,
            blueprints,
            affixes,
            global_meta_attributes: gmas,
            blueprint_affixes: HashMap::new(),
            by_client,
        };

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

        let gmas = HashMap::new();
        let by_client = Cache::build_client_caches(&clients, &blueprints, &affixes, &gmas);

        let cc1 = by_client.get(&c1).unwrap();
        let cc2 = by_client.get(&c2).unwrap();

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

        let by_client = Cache::build_client_caches(
            &clients,
            &blueprints,
            &HashMap::new(),
            &HashMap::new(),
        );

        let cc = by_client.get(&client_id).unwrap();
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

        let by_client = Cache::build_client_caches(
            &clients,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );

        let cc = by_client.get(&client_id).unwrap();
        assert!(cc.blueprints.is_empty());
        assert!(cc.affixes.is_empty());
        assert!(cc.global_meta_attributes.is_empty());
    }
}
