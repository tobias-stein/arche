use rand::Rng;
use sqlx::PgPool;

fn generate_api_key() -> String {
    let random_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    format!("arche_k_{}", random_part)
}

pub async fn bootstrap_super_admin(pool: &PgPool) -> Option<String> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM api_keys WHERE is_super = true)",
    )
    .fetch_one(pool)
    .await
    .expect("failed to check for existing super admin key");

    if exists {
        return None;
    }

    let raw_key = generate_api_key();
    let key_hash =
        bcrypt::hash(&raw_key, bcrypt::DEFAULT_COST).expect("failed to hash super admin key");

    let permissions: Vec<String> = vec!["admin".to_string()];

    sqlx::query(
        "INSERT INTO api_keys (name, key_hash, permissions, is_super, client_id) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind("Super Admin")
    .bind(&key_hash)
    .bind(&permissions)
    .bind(true)
    .bind(None::<uuid::Uuid>)
    .execute(pool)
    .await
    .expect("failed to insert super admin key");

    Some(raw_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_prefix() {
        let key = generate_api_key();
        assert!(
            key.starts_with("arche_k_"),
            "key should start with 'arche_k_', got: {}",
            key
        );
    }

    #[test]
    fn test_generate_api_key_length() {
        let key = generate_api_key();
        assert_eq!(
            key.len(),
            56,
            "key should be 56 chars (8 prefix + 48 random), got {}: '{}'",
            key.len(),
            key
        );
    }

    #[test]
    fn test_generate_api_key_randomness() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2, "successive calls should produce different keys");
    }

    #[test]
    fn test_generate_api_key_alphanumeric_after_prefix() {
        let key = generate_api_key();
        let after_prefix = key.strip_prefix("arche_k_").unwrap();
        assert!(
            after_prefix.chars().all(|c| c.is_ascii_alphanumeric()),
            "random part should be alphanumeric"
        );
    }
}
