use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn get_user_email(pool: &PgPool, user_id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, String>("select user_email from users where user_id = $1::uuid")
        .bind(user_id.to_string().to_lowercase())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

pub async fn get_user_id_by_email(pool: &PgPool, email: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Uuid>("select user_id from users where user_email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

pub async fn get_model_name(pool: &PgPool, model_id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, String>("select model_name from models where model_id = $1::uuid")
        .bind(model_id.to_string().to_lowercase())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<(Uuid, String)>> {
    let rows = sqlx::query_as::<_, (Uuid, String)>(
        "select user_id, user_email from users order by user_email",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_models(pool: &PgPool) -> Result<Vec<(Uuid, String)>> {
    let rows = sqlx::query_as::<_, (Uuid, String)>(
        "select model_id, model_name from models order by model_name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
