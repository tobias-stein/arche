use arche_types::common::PaginatedResponse;
use serde::Deserialize;
use uuid::Uuid;

const DEFAULT_LIMIT: i32 = 50;
const MAX_LIMIT: i32 = 200;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub limit: Option<i32>,
    #[serde(default)]
    pub page: Option<i32>,
    #[serde(default)]
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PaginationError {
    #[error("invalid cursor: {0} is not a valid UUID")]
    InvalidCursor(String),
    #[error("invalid page: must be >= 1")]
    InvalidPage,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaginationMode {
    Cursor {
        after: Option<Uuid>,
        limit: i32,
    },
    Offset {
        page: i32,
        per_page: i32,
        offset: i32,
    },
}

impl PaginationParams {
    pub fn mode(&self) -> Result<PaginationMode, PaginationError> {
        if self.cursor.is_some() || self.page.is_none() {
            let limit = self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
            let after = match &self.cursor {
                None => None,
                Some(s) if s.is_empty() => None,
                Some(s) => {
                    Some(Uuid::parse_str(s).map_err(|_| PaginationError::InvalidCursor(s.clone()))?)
                }
            };
            Ok(PaginationMode::Cursor { after, limit })
        } else {
            let per_page = self
                .per_page
                .or(self.limit)
                .unwrap_or(DEFAULT_LIMIT)
                .clamp(1, MAX_LIMIT);
            let page = self.page.unwrap_or(1);
            if page < 1 {
                return Err(PaginationError::InvalidPage);
            }
            let offset = (page - 1) * per_page;
            Ok(PaginationMode::Offset {
                page,
                per_page,
                offset,
            })
        }
    }
}

impl PaginationMode {
    pub fn limit(&self) -> i32 {
        match self {
            PaginationMode::Cursor { limit, .. } => *limit,
            PaginationMode::Offset { per_page, .. } => *per_page,
        }
    }

    pub fn fetch_limit(&self) -> i32 {
        match self {
            PaginationMode::Cursor { limit, .. } => limit + 1,
            PaginationMode::Offset { per_page, .. } => *per_page,
        }
    }
}

pub trait HasId {
    fn id(&self) -> Uuid;
}

pub fn paginate_cursor<T: HasId>(items: Vec<T>, limit: i32) -> PaginatedResponse<T> {
    let limit = limit.clamp(1, MAX_LIMIT) as usize;
    let has_more = items.len() > limit;
    let mut data = items;
    if has_more {
        data.truncate(limit);
    }
    let next_cursor = if has_more {
        data.last().map(|item| item.id().to_string())
    } else {
        None
    };
    PaginatedResponse {
        data,
        next_cursor,
        total: None,
    }
}

pub fn paginate_offset<T>(items: Vec<T>, total: i64) -> PaginatedResponse<T> {
    PaginatedResponse {
        data: items,
        next_cursor: None,
        total: Some(total),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_mode_when_cursor_present() {
        let uuid = Uuid::new_v4();
        let params = PaginationParams {
            cursor: Some(uuid.to_string()),
            limit: Some(10),
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Cursor {
                after: Some(uuid),
                limit: 10
            }
        );
    }

    #[test]
    fn test_cursor_mode_when_neither_cursor_nor_page() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Cursor {
                after: None,
                limit: DEFAULT_LIMIT
            }
        );
    }

    #[test]
    fn test_cursor_wins_over_page() {
        let uuid = Uuid::new_v4();
        let params = PaginationParams {
            cursor: Some(uuid.to_string()),
            limit: Some(10),
            page: Some(3),
            per_page: Some(20),
        };
        let mode = params.mode().unwrap();
        assert!(matches!(mode, PaginationMode::Cursor { .. }));
        if let PaginationMode::Cursor { after, limit } = mode {
            assert_eq!(after, Some(uuid));
            assert_eq!(limit, 10);
        }
    }

    #[test]
    fn test_offset_mode_when_page_present() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(2),
            per_page: Some(25),
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Offset {
                page: 2,
                per_page: 25,
                offset: 25
            }
        );
    }

    #[test]
    fn test_offset_mode_default_per_page() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(1),
            per_page: None,
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Offset {
                page: 1,
                per_page: DEFAULT_LIMIT,
                offset: 0
            }
        );
    }

    #[test]
    fn test_offset_mode_limit_as_fallback_per_page() {
        let params = PaginationParams {
            cursor: None,
            limit: Some(15),
            page: Some(3),
            per_page: None,
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Offset {
                page: 3,
                per_page: 15,
                offset: 30
            }
        );
    }

    #[test]
    fn test_offset_mode_per_page_overrides_limit() {
        let params = PaginationParams {
            cursor: None,
            limit: Some(15),
            page: Some(2),
            per_page: Some(30),
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Offset {
                page: 2,
                per_page: 30,
                offset: 30
            }
        );
    }

    #[test]
    fn test_default_limit_cursor_mode() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Cursor { limit, .. } = mode {
            assert_eq!(limit, 50);
        }
    }

    #[test]
    fn test_max_limit_clamped() {
        let params = PaginationParams {
            cursor: None,
            limit: Some(500),
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Cursor { limit, .. } = mode {
            assert_eq!(limit, 200);
        }
    }

    #[test]
    fn test_zero_limit_clamped() {
        let params = PaginationParams {
            cursor: None,
            limit: Some(0),
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Cursor { limit, .. } = mode {
            assert_eq!(limit, 1);
        }
    }

    #[test]
    fn test_negative_limit_clamped() {
        let params = PaginationParams {
            cursor: None,
            limit: Some(-10),
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Cursor { limit, .. } = mode {
            assert_eq!(limit, 1);
        }
    }

    #[test]
    fn test_per_page_clamped_max() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(1),
            per_page: Some(300),
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Offset { per_page, .. } = mode {
            assert_eq!(per_page, 200);
        }
    }

    #[test]
    fn test_per_page_clamped_min() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(1),
            per_page: Some(0),
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Offset { per_page, .. } = mode {
            assert_eq!(per_page, 1);
        }
    }

    #[test]
    fn test_invalid_cursor_uuid() {
        let params = PaginationParams {
            cursor: Some("not-a-uuid".to_string()),
            limit: None,
            page: None,
            per_page: None,
        };
        let result = params.mode();
        assert!(matches!(result, Err(PaginationError::InvalidCursor(_))));
    }

    #[test]
    fn test_empty_cursor_treated_as_none() {
        let params = PaginationParams {
            cursor: Some("".to_string()),
            limit: Some(10),
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        assert_eq!(
            mode,
            PaginationMode::Cursor {
                after: None,
                limit: 10
            }
        );
    }

    #[test]
    fn test_negative_page_invalid() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(0),
            per_page: None,
        };
        let result = params.mode();
        assert!(matches!(result, Err(PaginationError::InvalidPage)));

        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(-1),
            per_page: None,
        };
        let result = params.mode();
        assert!(matches!(result, Err(PaginationError::InvalidPage)));
    }

    #[test]
    fn test_offset_calculation() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(3),
            per_page: Some(25),
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Offset {
            page, per_page, offset,
        } = mode
        {
            assert_eq!(page, 3);
            assert_eq!(per_page, 25);
            assert_eq!(offset, 50);
        }
    }

    #[test]
    fn test_fetch_limit_cursor_mode() {
        let mode = PaginationMode::Cursor {
            after: None,
            limit: 50,
        };
        assert_eq!(mode.fetch_limit(), 51);
    }

    #[test]
    fn test_fetch_limit_offset_mode() {
        let mode = PaginationMode::Offset {
            page: 1,
            per_page: 25,
            offset: 0,
        };
        assert_eq!(mode.fetch_limit(), 25);
    }

    #[test]
    fn test_paginate_cursor_with_next_page() {
        #[derive(Debug, Clone, PartialEq)]
        struct Item {
            id: Uuid,
            name: String,
        }
        impl HasId for Item {
            fn id(&self) -> Uuid {
                self.id
            }
        }

        let ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let items: Vec<Item> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| Item {
                id: *id,
                name: format!("item-{}", i),
            })
            .collect();

        let result = paginate_cursor(items, 3);
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.next_cursor, Some(ids[2].to_string()));
        assert_eq!(result.total, None);

        assert_eq!(result.data[0].id, ids[0]);
        assert_eq!(result.data[1].id, ids[1]);
        assert_eq!(result.data[2].id, ids[2]);
    }

    #[test]
    fn test_paginate_cursor_last_page() {
        #[derive(Debug, Clone, PartialEq)]
        struct Item {
            id: Uuid,
        }
        impl HasId for Item {
            fn id(&self) -> Uuid {
                self.id
            }
        }

        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        let items: Vec<Item> = ids.iter().map(|id| Item { id: *id }).collect();

        let result = paginate_cursor(items, 50);
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.total, None);
    }

    #[test]
    fn test_paginate_cursor_empty_result() {
        #[derive(Debug, Clone)]
        struct Wrapper(Uuid);
        impl HasId for Wrapper {
            fn id(&self) -> Uuid {
                self.0
            }
        }

        let items: Vec<Wrapper> = vec![];
        let result = paginate_cursor(items, 50);
        assert_eq!(result.data.len(), 0);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.total, None);
    }

    #[test]
    fn test_paginate_cursor_exact_limit_items() {
        #[derive(Debug, Clone, PartialEq)]
        struct Item {
            id: Uuid,
        }
        impl HasId for Item {
            fn id(&self) -> Uuid {
                self.id
            }
        }

        let ids: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
        let items: Vec<Item> = ids.iter().map(|id| Item { id: *id }).collect();

        let result = paginate_cursor(items, 50);
        assert_eq!(result.data.len(), 50);
        assert_eq!(result.next_cursor, None);
    }

    #[test]
    fn test_paginate_cursor_51_items_with_limit_50() {
        #[derive(Debug, Clone)]
        struct Item {
            id: Uuid,
        }
        impl HasId for Item {
            fn id(&self) -> Uuid {
                self.id
            }
        }

        let ids: Vec<Uuid> = (0..51).map(|_| Uuid::new_v4()).collect();
        let items: Vec<Item> = ids.iter().map(|id| Item { id: *id }).collect();

        let result = paginate_cursor(items, 50);
        assert_eq!(result.data.len(), 50);
        assert_eq!(result.next_cursor, Some(ids[49].to_string()));
    }

    #[test]
    fn test_paginate_offset_basic() {
        let items = vec!["a".to_string(), "b".to_string()];
        let result = paginate_offset(items, 100);
        assert_eq!(result.data, vec!["a", "b"]);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.total, Some(100));
    }

    #[test]
    fn test_paginate_offset_empty() {
        let items: Vec<String> = vec![];
        let result = paginate_offset(items, 0);
        assert_eq!(result.data.len(), 0);
        assert_eq!(result.next_cursor, None);
        assert_eq!(result.total, Some(0));
    }

    #[test]
    fn test_deserialize_pagination_params_from_json() {
        let json = r#"{"cursor":"550e8400-e29b-41d4-a716-446655440000","limit":10}"#;
        let params: PaginationParams = serde_json::from_str(json).unwrap();
        assert_eq!(
            params.cursor,
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.page, None);
        assert_eq!(params.per_page, None);
    }

    #[test]
    fn test_deserialize_pagination_params_empty() {
        let json = r#"{}"#;
        let params: PaginationParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.cursor, None);
        assert_eq!(params.limit, None);
        assert_eq!(params.page, None);
        assert_eq!(params.per_page, None);
    }

    #[test]
    fn test_deserialize_pagination_params_offset_mode() {
        let json = r#"{"page":2,"per_page":25}"#;
        let params: PaginationParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.page, Some(2));
        assert_eq!(params.per_page, Some(25));
        assert_eq!(params.cursor, None);
    }

    #[test]
    fn test_paginated_response_serialization_cursor() {
        let uuid = Uuid::new_v4();
        let resp: PaginatedResponse<String> = PaginatedResponse {
            data: vec!["a".into(), "b".into()],
            next_cursor: Some(uuid.to_string()),
            total: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("data"));
        assert!(obj.contains_key("nextCursor"));
        assert!(!obj.contains_key("total"));
        assert_eq!(obj.get("nextCursor").unwrap(), &serde_json::json!(uuid.to_string()));
    }

    #[test]
    fn test_paginated_response_serialization_offset() {
        let resp: PaginatedResponse<String> = PaginatedResponse {
            data: vec!["a".into()],
            next_cursor: None,
            total: Some(42),
        };
        let json = serde_json::to_value(&resp).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("data"));
        assert!(!obj.contains_key("nextCursor"));
        assert!(obj.contains_key("total"));
        assert_eq!(obj.get("total").unwrap(), 42);
    }

    #[test]
    fn test_pagination_mode_limit() {
        let cursor_mode = PaginationMode::Cursor {
            after: None,
            limit: 25,
        };
        assert_eq!(cursor_mode.limit(), 25);

        let offset_mode = PaginationMode::Offset {
            page: 2,
            per_page: 30,
            offset: 30,
        };
        assert_eq!(offset_mode.limit(), 30);
    }

    #[test]
    fn test_paginate_cursor_single_item_with_more() {
        #[derive(Debug, Clone)]
        struct Item {
            id: Uuid,
        }
        impl HasId for Item {
            fn id(&self) -> Uuid {
                self.id
            }
        }

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let items = vec![Item { id: id1 }, Item { id: id2 }];
        let result = paginate_cursor(items, 1);
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.next_cursor, Some(id1.to_string()));
    }

    #[test]
    fn test_cursor_mode_with_valid_uuid() {
        let uuid = Uuid::new_v4();
        let params = PaginationParams {
            cursor: Some(uuid.to_string()),
            limit: Some(25),
            page: None,
            per_page: None,
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Cursor { after, limit } = mode {
            assert_eq!(after, Some(uuid));
            assert_eq!(limit, 25);
        } else {
            panic!("Expected Cursor mode");
        }
    }

    #[test]
    fn test_offset_mode_with_negative_per_page_clamped() {
        let params = PaginationParams {
            cursor: None,
            limit: None,
            page: Some(1),
            per_page: Some(-5),
        };
        let mode = params.mode().unwrap();
        if let PaginationMode::Offset { per_page, .. } = mode {
            assert_eq!(per_page, 1);
        }
    }
}
