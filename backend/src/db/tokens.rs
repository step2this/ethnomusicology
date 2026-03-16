use chrono::NaiveDateTime;
use sqlx::PgPool;

pub async fn store_tokens(
    pool: &PgPool,
    user_id: &str,
    access_encrypted: &[u8],
    refresh_encrypted: &[u8],
    expires_at: NaiveDateTime,
    scopes: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_spotify_tokens (user_id, access_token_encrypted, refresh_token_encrypted, expires_at, scopes, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW())
         ON CONFLICT(user_id) DO UPDATE SET
           access_token_encrypted = excluded.access_token_encrypted,
           refresh_token_encrypted = excluded.refresh_token_encrypted,
           expires_at = excluded.expires_at,
           scopes = excluded.scopes,
           updated_at = NOW()",
    )
    .bind(user_id)
    .bind(access_encrypted)
    .bind(refresh_encrypted)
    .bind(expires_at)
    .bind(scopes)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_tokens(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<(Vec<u8>, Vec<u8>, NaiveDateTime, String)>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Vec<u8>, Vec<u8>, NaiveDateTime, String)>(
        "SELECT access_token_encrypted, refresh_token_encrypted, expires_at, scopes
         FROM user_spotify_tokens WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn delete_tokens(pool: &PgPool, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_spotify_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::create_test_pool;

    #[tokio::test]
    async fn test_get_tokens_not_found() {
        let pool = create_test_pool().await;

        let result = get_tokens(&pool, "nonexistent-user").await.unwrap();
        assert!(result.is_none());
        pool.close().await;
    }
}
