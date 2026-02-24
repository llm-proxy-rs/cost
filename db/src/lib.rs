use anyhow::Result;
use common::{ApiKeyInfo, CostRecord, InferenceProfileInfo, ModelInfo, UserInfo};
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

pub fn init_pool_lazy(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(database_url)?;
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

pub async fn list_users_enriched(pool: &PgPool) -> Result<Vec<UserInfo>> {
    let rows = sqlx::query_as::<_, (Uuid, String, String, i64, i64, i64)>(
        r#"select
            u.user_id,
            u.user_email,
            coalesce(to_char(u.created_at, 'YYYY-MM-DD'), ''),
            (select count(*) from api_keys ak where ak.user_id = u.user_id),
            (select count(*) from api_keys ak where ak.user_id = u.user_id and not ak.is_disabled),
            (select count(*) from inference_profiles ip where ip.user_id = u.user_id)
        from users u
        order by u.user_email"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                user_id,
                user_email,
                created_at,
                api_key_count,
                active_api_key_count,
                inference_profile_count,
            )| {
                UserInfo {
                    user_id: user_id.to_string(),
                    user_email,
                    created_at,
                    api_key_count,
                    active_api_key_count,
                    inference_profile_count,
                }
            },
        )
        .collect())
}

pub async fn get_user_info(pool: &PgPool, user_id: Uuid) -> Option<UserInfo> {
    let row = sqlx::query_as::<_, (Uuid, String, String, i64, i64, i64)>(
        r#"select
            u.user_id,
            u.user_email,
            coalesce(to_char(u.created_at, 'YYYY-MM-DD'), ''),
            (select count(*) from api_keys ak where ak.user_id = u.user_id),
            (select count(*) from api_keys ak where ak.user_id = u.user_id and not ak.is_disabled),
            (select count(*) from inference_profiles ip where ip.user_id = u.user_id)
        from users u
        where u.user_id = $1::uuid"#,
    )
    .bind(user_id.to_string().to_lowercase())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let (
        user_id,
        user_email,
        created_at,
        api_key_count,
        active_api_key_count,
        inference_profile_count,
    ) = row;
    Some(UserInfo {
        user_id: user_id.to_string(),
        user_email,
        created_at,
        api_key_count,
        active_api_key_count,
        inference_profile_count,
    })
}

pub async fn list_models_enriched(pool: &PgPool) -> Result<Vec<ModelInfo>> {
    let rows = sqlx::query_as::<_, (Uuid, String, bool, bool, i64)>(
        r#"select
            m.model_id,
            m.model_name,
            coalesce(m.is_disabled, false),
            coalesce(m.protected, false),
            (select count(distinct ip.user_id) from inference_profiles ip where ip.model_id = m.model_id)
        from models m
        order by m.model_name"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(model_id, model_name, is_disabled, protected, user_count)| ModelInfo {
                model_id: model_id.to_string(),
                model_name,
                is_disabled,
                protected,
                user_count,
            },
        )
        .collect())
}

pub async fn get_model_info(pool: &PgPool, model_id: Uuid) -> Option<ModelInfo> {
    let row = sqlx::query_as::<_, (Uuid, String, bool, bool, i64)>(
        r#"select
            m.model_id,
            m.model_name,
            coalesce(m.is_disabled, false),
            coalesce(m.protected, false),
            (select count(distinct ip.user_id) from inference_profiles ip where ip.model_id = m.model_id)
        from models m
        where m.model_id = $1::uuid"#,
    )
    .bind(model_id.to_string().to_lowercase())
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    let (model_id, model_name, is_disabled, protected, user_count) = row;
    Some(ModelInfo {
        model_id: model_id.to_string(),
        model_name,
        is_disabled,
        protected,
        user_count,
    })
}

pub async fn list_api_keys_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<ApiKeyInfo>> {
    let rows = sqlx::query_as::<_, (Uuid, String, bool, String)>(
        r#"select
            ak.api_key_id,
            right(ak.api_key, 8),
            ak.is_disabled,
            coalesce(to_char(ak.created_at, 'YYYY-MM-DD'), '')
        from api_keys ak
        where ak.user_id = $1::uuid
        order by ak.created_at desc"#,
    )
    .bind(user_id.to_string().to_lowercase())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(api_key_id, api_key_preview, is_disabled, created_at)| ApiKeyInfo {
                api_key_id: api_key_id.to_string(),
                api_key_preview,
                is_disabled,
                created_at,
            },
        )
        .collect())
}

pub async fn list_profiles_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<InferenceProfileInfo>> {
    let rows = sqlx::query_as::<_, (Uuid, Uuid, Option<String>, Uuid, Option<String>, String)>(
        r#"select
            ip.inference_profile_id,
            ip.model_id,
            m.model_name,
            ip.user_id,
            u.user_email,
            coalesce(to_char(ip.created_at, 'YYYY-MM-DD'), '')
        from inference_profiles ip
        left join models m on m.model_id = ip.model_id
        left join users u on u.user_id = ip.user_id
        where ip.user_id = $1::uuid
        order by ip.created_at desc"#,
    )
    .bind(user_id.to_string().to_lowercase())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(inference_profile_id, model_id, model_name, user_id, user_email, created_at)| {
                InferenceProfileInfo {
                    inference_profile_id: inference_profile_id.to_string(),
                    model_id: model_id.to_string(),
                    model_name,
                    user_id: user_id.to_string(),
                    user_email,
                    created_at,
                }
            },
        )
        .collect())
}

// --- Cost cache functions ---

pub async fn create_cost_cache_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS cost_cache (
            query_type TEXT NOT NULL,
            filter_id TEXT NOT NULL DEFAULT '',
            date TEXT NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            currency TEXT NOT NULL,
            PRIMARY KEY (query_type, filter_id, date)
        )"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_cached_costs(
    pool: &PgPool,
    query_type: &str,
    filter_id: &str,
    start: &str,
    end: &str,
) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        "SELECT date, amount, currency FROM cost_cache WHERE query_type=$1 AND filter_id=$2 AND date>=$3 AND date<$4 ORDER BY date",
    )
    .bind(query_type)
    .bind(filter_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(date, amount, currency)| CostRecord {
            date,
            amount,
            currency,
        })
        .collect())
}

pub async fn upsert_cached_costs(
    pool: &PgPool,
    query_type: &str,
    filter_id: &str,
    records: &[CostRecord],
) -> Result<()> {
    for record in records {
        sqlx::query(
            r#"INSERT INTO cost_cache (query_type, filter_id, date, amount, currency)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (query_type, filter_id, date)
               DO UPDATE SET amount=EXCLUDED.amount, currency=EXCLUDED.currency"#,
        )
        .bind(query_type)
        .bind(filter_id)
        .bind(&record.date)
        .bind(record.amount)
        .bind(&record.currency)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn list_profiles_for_model(
    pool: &PgPool,
    model_id: Uuid,
) -> Result<Vec<InferenceProfileInfo>> {
    let rows = sqlx::query_as::<_, (Uuid, Uuid, Option<String>, Uuid, Option<String>, String)>(
        r#"select
            ip.inference_profile_id,
            ip.model_id,
            m.model_name,
            ip.user_id,
            u.user_email,
            coalesce(to_char(ip.created_at, 'YYYY-MM-DD'), '')
        from inference_profiles ip
        left join models m on m.model_id = ip.model_id
        left join users u on u.user_id = ip.user_id
        where ip.model_id = $1::uuid
        order by ip.created_at desc"#,
    )
    .bind(model_id.to_string().to_lowercase())
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(inference_profile_id, model_id, model_name, user_id, user_email, created_at)| {
                InferenceProfileInfo {
                    inference_profile_id: inference_profile_id.to_string(),
                    model_id: model_id.to_string(),
                    model_name,
                    user_id: user_id.to_string(),
                    user_email,
                    created_at,
                }
            },
        )
        .collect())
}
