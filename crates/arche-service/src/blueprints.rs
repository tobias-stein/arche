use arche_types::common::{BlueprintListQuery, PaginatedResponse};
use arche_types::Blueprint;
use axum::extract::{Query, State};
use axum::Json;
use sqlx::postgres::PgRow;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::auth::permission::CurrentUser;
use crate::error::ProblemResponse;
use crate::pagination::{
    paginate_cursor, paginate_offset, HasId, PaginationMode, PaginationParams,
};

impl HasId for Blueprint {
    fn id(&self) -> Uuid {
        self.id
    }
}

const BLUEPRINT_COLUMNS: &str = "id, client_id, name, archetype, weight, description, \
    attributes, attribute_order, min_prefixes, max_prefixes, \
    min_suffixes, max_suffixes, created_at, updated_at";

pub async fn list_blueprints(
    State(state): State<crate::AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<BlueprintListQuery>,
) -> Result<Json<PaginatedResponse<Blueprint>>, ProblemResponse> {
    let client_id_filter = resolve_client_id(&user, query.client_id)?;

    let params = PaginationParams {
        cursor: query.cursor.clone(),
        limit: query.limit,
        page: query.page,
        per_page: query.per_page,
    };

    let mode = params.mode().map_err(|e| {
        ProblemResponse::validation_error(e.to_string(), vec![])
    })?;

    match mode {
        PaginationMode::Cursor { after, limit } => {
            let fetch_limit = limit + 1;

            let mut builder = QueryBuilder::new(
                format!("SELECT {BLUEPRINT_COLUMNS} FROM blueprints"),
            );
            apply_filters(
                &mut builder,
                client_id_filter,
                query.archetype.as_deref(),
                query.search.as_deref(),
                after,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(fetch_limit);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "blueprints: list cursor query failed");
                ProblemResponse::unprocessable_entity("Failed to list blueprints")
            })?;

            let blueprints: Vec<Blueprint> = rows.iter().map(row_to_blueprint).collect();
            let response = paginate_cursor(blueprints, limit);
            Ok(Json(response))
        }
        PaginationMode::Offset {
            per_page,
            offset,
            ..
        } => {
            let mut count_builder =
                QueryBuilder::new("SELECT COUNT(*) FROM blueprints");
            apply_filters(
                &mut count_builder,
                client_id_filter,
                query.archetype.as_deref(),
                query.search.as_deref(),
                None,
            );

            let total: i64 = count_builder
                .build()
                .fetch_one(&*state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "blueprints: list count query failed");
                    ProblemResponse::unprocessable_entity("Failed to count blueprints")
                })?
                .get(0);

            let mut builder = QueryBuilder::new(
                format!("SELECT {BLUEPRINT_COLUMNS} FROM blueprints"),
            );
            apply_filters(
                &mut builder,
                client_id_filter,
                query.archetype.as_deref(),
                query.search.as_deref(),
                None,
            );
            builder.push(" ORDER BY id ASC LIMIT ");
            builder.push_bind(per_page);
            builder.push(" OFFSET ");
            builder.push_bind(offset);

            let rows = builder.build().fetch_all(&*state.pool).await.map_err(|e| {
                tracing::error!(error = %e, "blueprints: list offset query failed");
                ProblemResponse::unprocessable_entity("Failed to list blueprints")
            })?;

            let blueprints: Vec<Blueprint> = rows.iter().map(row_to_blueprint).collect();
            let response = paginate_offset(blueprints, total);
            Ok(Json(response))
        }
    }
}

fn apply_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    client_id: Option<Uuid>,
    archetype: Option<&'a str>,
    search: Option<&'a str>,
    cursor: Option<Uuid>,
) {
    let mut first = true;

    if let Some(cid) = client_id {
        builder.push(" WHERE client_id = ");
        builder.push_bind(cid);
        first = false;
    }

    if let Some(at) = archetype {
        if first {
            builder.push(" WHERE ");
            first = false;
        } else {
            builder.push(" AND ");
        }
        builder.push("archetype = ");
        builder.push_bind(at);
    }

    if let Some(s) = search {
        if first {
            builder.push(" WHERE ");
            first = false;
        } else {
            builder.push(" AND ");
        }
        builder.push("name ILIKE ");
        builder.push_bind(format!("%{s}%"));
    }

    if let Some(c) = cursor {
        if first {
            builder.push(" WHERE ");
        } else {
            builder.push(" AND ");
        }
        builder.push("id > ");
        builder.push_bind(c);
    }
}

fn resolve_client_id(
    user: &crate::auth::AuthenticatedKey,
    query_client_id: Option<Uuid>,
) -> Result<Option<Uuid>, ProblemResponse> {
    if user.is_super {
        Ok(query_client_id)
    } else {
        user.client_id.map(Some).ok_or_else(|| {
            ProblemResponse::forbidden(
                "Access denied: key is not associated with any client",
            )
        })
    }
}

fn row_to_blueprint(row: &PgRow) -> Blueprint {
    Blueprint {
        id: row.get("id"),
        client_id: row.get("client_id"),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthenticatedKey;
    use arche_types::Permission;
    use chrono::Utc;

    fn make_key(
        client_id: Option<Uuid>,
        is_super: bool,
    ) -> AuthenticatedKey {
        AuthenticatedKey {
            id: Uuid::nil(),
            name: "test".into(),
            client_id,
            permissions: vec![Permission::Read],
            is_super,
        }
    }

    #[test]
    fn test_blueprint_has_id() {
        let id = Uuid::new_v4();
        let bp = Blueprint {
            id,
            client_id: Uuid::new_v4(),
            name: "test".into(),
            archetype: "sword".into(),
            weight: 1.0,
            description: None,
            attributes: serde_json::json!({}),
            attribute_order: vec![],
            min_prefixes: 0,
            max_prefixes: 0,
            min_suffixes: 0,
            max_suffixes: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(bp.id(), id);
    }

    #[test]
    fn test_resolve_client_id_super_admin_no_filter() {
        let key = make_key(None, true);
        let result = resolve_client_id(&key, None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_client_id_super_admin_with_filter() {
        let key = make_key(None, true);
        let cid = Uuid::new_v4();
        let result = resolve_client_id(&key, Some(cid)).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key() {
        let cid = Uuid::new_v4();
        let key = make_key(Some(cid), false);
        let result = resolve_client_id(&key, None).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_resolve_client_id_regular_key_no_client() {
        let key = make_key(None, false);
        let result = resolve_client_id(&key, None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, 403);
    }

    #[test]
    fn test_resolve_client_id_regular_key_ignores_query_param() {
        let cid = Uuid::new_v4();
        let other = Uuid::new_v4();
        let key = make_key(Some(cid), false);
        let result = resolve_client_id(&key, Some(other)).unwrap();
        assert_eq!(result, Some(cid));
    }

    #[test]
    fn test_blueprint_list_query_json_deserialization() {
        let cid = Uuid::new_v4();
        let json = format!(
            r#"{{"cursor":"abc","limit":5,"page":2,"perPage":20,"clientId":"{}","archetype":"sword","search":"fire"}}"#,
            cid
        );
        let query: BlueprintListQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query.cursor, Some("abc".into()));
        assert_eq!(query.limit, Some(5));
        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(20));
        assert_eq!(query.client_id, Some(cid));
        assert_eq!(query.archetype, Some("sword".into()));
        assert_eq!(query.search, Some("fire".into()));
    }

    #[test]
    fn test_blueprint_list_query_empty_json() {
        let query: BlueprintListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.cursor, None);
        assert_eq!(query.limit, None);
        assert_eq!(query.page, None);
        assert_eq!(query.per_page, None);
        assert_eq!(query.client_id, None);
        assert_eq!(query.archetype, None);
        assert_eq!(query.search, None);
    }

    #[test]
    fn test_pagination_params_from_query() {
        let q = BlueprintListQuery {
            cursor: None,
            limit: Some(10),
            page: Some(2),
            per_page: Some(25),
            client_id: None,
            archetype: None,
            search: None,
        };
        let params = PaginationParams {
            cursor: q.cursor,
            limit: q.limit,
            page: q.page,
            per_page: q.per_page,
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, PaginationMode::Offset { .. }));
        if let PaginationMode::Offset {
            page,
            per_page,
            offset,
        } = mode
        {
            assert_eq!(page, 2);
            assert_eq!(per_page, 25);
            assert_eq!(offset, 25);
        }
    }

    #[test]
    fn test_pagination_params_cursor_wins() {
        let q = BlueprintListQuery {
            cursor: Some(Uuid::new_v4().to_string()),
            limit: Some(5),
            page: Some(3),
            per_page: Some(10),
            client_id: None,
            archetype: None,
            search: None,
        };
        let params = PaginationParams {
            cursor: q.cursor,
            limit: q.limit,
            page: q.page,
            per_page: q.per_page,
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, PaginationMode::Cursor { .. }));
    }

    #[test]
    fn test_pagination_params_offset_mode() {
        let q = BlueprintListQuery {
            cursor: None,
            limit: None,
            page: Some(2),
            per_page: Some(15),
            client_id: None,
            archetype: None,
            search: None,
        };
        let params = PaginationParams {
            cursor: q.cursor,
            limit: q.limit,
            page: q.page,
            per_page: q.per_page,
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, PaginationMode::Offset { .. }));
        if let PaginationMode::Offset {
            page,
            per_page,
            offset,
        } = mode
        {
            assert_eq!(page, 2);
            assert_eq!(per_page, 15);
            assert_eq!(offset, 15);
        }
    }
}
