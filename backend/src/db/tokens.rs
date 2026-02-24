use chrono::NaiveDateTime;
use sqlx::SqlitePool;

pub async fn store_tokens(
    pool: &SqlitePool,
    user_id: &str,
    access_encrypted: &[u8],
    refresh_encrypted: &[u8],
    expires_at: NaiveDateTime,
    scopes: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_spotify_tokens (user_id, access_token_encrypted, refresh_token_encrypted, expires_at, scopes, updated_at)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(user_id) DO UPDATE SET
           access_token_encrypted = excluded.access_token_encrypted,
           refresh_token_encrypted = excluded.refresh_token_encrypted,
           expires_at = excluded.expires_at,
           scopes = excluded.scopes,
           updated_at = CURRENT_TIMESTAMP",
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
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<(Vec<u8>, Vec<u8>, NaiveDateTime, String)>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Vec<u8>, Vec<u8>, NaiveDateTime, String)>(
        "SELECT access_token_encrypted, refresh_token_encrypted, expires_at, scopes
         FROM user_spotify_tokens WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn delete_tokens(pool: &SqlitePool, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_spotify_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_test_pool, create_test_user};
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_store_and_get_tokens() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        let expires = NaiveDate::from_ymd_opt(2026, 3, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let access = b"encrypted_access_token";
        let refresh = b"encrypted_refresh_token";

        store_tokens(
            &pool,
            &user_id,
            access,
            refresh,
            expires,
            "user-read-private",
        )
        .await
        .unwrap();

        let (got_access, got_refresh, got_expires, got_scopes) =
            get_tokens(&pool, &user_id).await.unwrap().unwrap();

        assert_eq!(got_access, access);
        assert_eq!(got_refresh, refresh);
        assert_eq!(got_expires, expires);
        assert_eq!(got_scopes, "user-read-private");
    }

    #[tokio::test]
    async fn test_store_tokens_upsert() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        let expires1 = NaiveDate::from_ymd_opt(2026, 3, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        store_tokens(&pool, &user_id, b"access1", b"refresh1", expires1, "scope1")
            .await
            .unwrap();

        let expires2 = NaiveDate::from_ymd_opt(2026, 4, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        store_tokens(&pool, &user_id, b"access2", b"refresh2", expires2, "scope2")
            .await
            .unwrap();

        let (got_access, _, got_expires, got_scopes) =
            get_tokens(&pool, &user_id).await.unwrap().unwrap();

        assert_eq!(got_access, b"access2");
        assert_eq!(got_expires, expires2);
        assert_eq!(got_scopes, "scope2");
    }

    #[tokio::test]
    async fn test_delete_tokens() {
        let pool = create_test_pool().await;
        let user_id = create_test_user(&pool).await;

        let expires = NaiveDate::from_ymd_opt(2026, 3, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        store_tokens(&pool, &user_id, b"access", b"refresh", expires, "scope")
            .await
            .unwrap();

        delete_tokens(&pool, &user_id).await.unwrap();

        let result = get_tokens(&pool, &user_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_tokens_not_found() {
        let pool = create_test_pool().await;

        let result = get_tokens(&pool, "nonexistent-user").await.unwrap();
        assert!(result.is_none());
    }
}
