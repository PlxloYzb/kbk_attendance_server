use crate::models::UserInfo;
use sqlx::PgPool;

pub async fn verify_passkey(pool: &PgPool, passkey: &str) -> Result<Option<UserInfo>, sqlx::Error> {
    let user = sqlx::query_as::<_, UserInfo>(
        "SELECT * FROM user_info WHERE passkey = $1"
    )
    .bind(passkey)
    .fetch_optional(pool)
    .await?;
    
    Ok(user)
}

pub async fn verify_user_passkey(
    pool: &PgPool,
    user_id: &str,
    passkey: &str,
) -> Result<bool, sqlx::Error> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_info WHERE user_id = $1 AND passkey = $2"
    )
    .bind(user_id)
    .bind(passkey)
    .fetch_one(pool)
    .await?;
    
    Ok(result.0 > 0)
}