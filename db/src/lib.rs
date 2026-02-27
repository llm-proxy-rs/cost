use anyhow::Result;
use common::{ApiKeyInfo, CostByModel, CostByUser, CostRecord, CostRow, InferenceProfileInfo, ModelInfo, UserInfo};
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

// --- Cost table functions ---

pub async fn create_cost_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS cost (
            date DATE NOT NULL,
            user_id TEXT NOT NULL,
            model_id TEXT NOT NULL,
            amount DOUBLE PRECISION NOT NULL,
            currency TEXT NOT NULL DEFAULT 'USD',
            PRIMARY KEY (date, user_id, model_id)
        )"#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_cost_rows(pool: &PgPool, rows: &[CostRow]) -> Result<()> {
    for row in rows {
        sqlx::query(
            r#"INSERT INTO cost (date, user_id, model_id, amount, currency)
               VALUES ($1::date, $2, $3, $4, $5)
               ON CONFLICT (date, user_id, model_id)
               DO UPDATE SET amount=EXCLUDED.amount, currency=EXCLUDED.currency"#,
        )
        .bind(&row.date)
        .bind(&row.user_id)
        .bind(&row.model_id)
        .bind(row.amount)
        .bind(&row.currency)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_daily_cost(pool: &PgPool, start: &str, end: &str) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT date::text, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date
           GROUP BY date ORDER BY date"#,
    )
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

pub async fn get_monthly_cost(pool: &PgPool, start: &str, end: &str) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT to_char(DATE_TRUNC('month', date), 'YYYY-MM-DD'), SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date
           GROUP BY DATE_TRUNC('month', date) ORDER BY DATE_TRUNC('month', date)"#,
    )
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

pub async fn get_cost_by_user(pool: &PgPool, start: &str, end: &str) -> Result<Vec<CostByUser>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT user_id, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date
           GROUP BY user_id ORDER BY SUM(amount) DESC"#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(user_id, amount, currency)| CostByUser {
            user_id,
            user_email: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_cost_by_model(
    pool: &PgPool,
    start: &str,
    end: &str,
) -> Result<Vec<CostByModel>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT model_id, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date
           GROUP BY model_id ORDER BY SUM(amount) DESC"#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(model_id, amount, currency)| CostByModel {
            model_id,
            model_name: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_cost_by_model_for_user(
    pool: &PgPool,
    start: &str,
    end: &str,
    user_id: &str,
) -> Result<Vec<CostByModel>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT model_id, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date AND user_id = $3
           GROUP BY model_id ORDER BY SUM(amount) DESC"#,
    )
    .bind(start)
    .bind(end)
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(model_id, amount, currency)| CostByModel {
            model_id,
            model_name: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_cost_by_user_for_model(
    pool: &PgPool,
    start: &str,
    end: &str,
    model_id: &str,
) -> Result<Vec<CostByUser>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT user_id, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date AND model_id = $3
           GROUP BY user_id ORDER BY SUM(amount) DESC"#,
    )
    .bind(start)
    .bind(end)
    .bind(model_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(user_id, amount, currency)| CostByUser {
            user_id,
            user_email: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_daily_cost_for_user(
    pool: &PgPool,
    start: &str,
    end: &str,
    user_id: &str,
) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT date::text, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date AND user_id = $3
           GROUP BY date ORDER BY date"#,
    )
    .bind(start)
    .bind(end)
    .bind(user_id)
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

pub async fn get_monthly_cost_for_user(
    pool: &PgPool,
    start: &str,
    end: &str,
    user_id: &str,
) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT to_char(DATE_TRUNC('month', date), 'YYYY-MM-DD'), SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date AND user_id = $3
           GROUP BY DATE_TRUNC('month', date) ORDER BY DATE_TRUNC('month', date)"#,
    )
    .bind(start)
    .bind(end)
    .bind(user_id)
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

pub async fn get_daily_cost_for_model(
    pool: &PgPool,
    start: &str,
    end: &str,
    model_id: &str,
) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT date::text, SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date AND model_id = $3
           GROUP BY date ORDER BY date"#,
    )
    .bind(start)
    .bind(end)
    .bind(model_id)
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

pub async fn get_monthly_cost_for_model(
    pool: &PgPool,
    start: &str,
    end: &str,
    model_id: &str,
) -> Result<Vec<CostRecord>> {
    let rows = sqlx::query_as::<_, (String, f64, String)>(
        r#"SELECT to_char(DATE_TRUNC('month', date), 'YYYY-MM-DD'), SUM(amount), MIN(currency)
           FROM cost WHERE date >= $1::date AND date < $2::date AND model_id = $3
           GROUP BY DATE_TRUNC('month', date) ORDER BY DATE_TRUNC('month', date)"#,
    )
    .bind(start)
    .bind(end)
    .bind(model_id)
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
