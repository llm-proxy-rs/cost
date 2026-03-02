use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct BatchConfig {
    #[serde(default = "default_database_url_cost")]
    database_url_cost: String,
    #[serde(default = "default_database_url_gateway_ro")]
    database_url_gateway_ro: String,
    #[serde(default = "default_incremental_days")]
    incremental_days: i64,
    start: Option<String>,
    end: Option<String>,
}

fn default_database_url_cost() -> String {
    "postgres://postgres:postgres@localhost/cost".to_string()
}

fn default_database_url_gateway_ro() -> String {
    "postgres://postgres:postgres@localhost/gateway".to_string()
}

fn default_incremental_days() -> i64 {
    3
}

fn load_config() -> Result<BatchConfig> {
    let cfg: BatchConfig = config::Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::Environment::default())
        .build()?
        .try_deserialize()?;
    Ok(cfg)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("batch=info"));

    let cfg = load_config()?;

    let today = Utc::now().date_naive();

    let (start, end) = if let (Some(s), Some(e)) = (&cfg.start, &cfg.end) {
        let _ = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
        let _ = NaiveDate::parse_from_str(e, "%Y-%m-%d")?;
        (s.clone(), e.clone())
    } else {
        // Incremental: last 3 days
        let start_date = today - chrono::Duration::days(cfg.incremental_days);
        (
            start_date.format("%Y-%m-%d").to_string(),
            today.format("%Y-%m-%d").to_string(),
        )
    };

    log::info!("Fetching CE data from {} to {}", start, end);

    let ce_client = ce::new_client().await;
    let rows = ce::get_daily_cost_by_user_and_model(&ce_client, &start, &end).await?;
    log::info!("Fetched {} cost rows from CE", rows.len());

    // Query gateway DB for known user_ids and model_ids
    let gateway_pool = db::init_pool(&cfg.database_url_gateway_ro).await?;
    let (known_users, known_models) = tokio::try_join!(
        db::list_user_ids(&gateway_pool),
        db::list_model_ids(&gateway_pool),
    )?;
    log::info!(
        "Gateway DB: {} known users, {} known models",
        known_users.len(),
        known_models.len()
    );

    // Filter CE rows to only known users and models
    let mut filtered_rows = Vec::new();
    let mut unknown_user_ids = std::collections::HashSet::new();
    let mut unknown_model_ids = std::collections::HashSet::new();
    let mut skipped_count = 0usize;

    for row in &rows {
        let user_known = known_users.contains(&row.user_id);
        let model_known = known_models.contains(&row.model_id);
        if user_known && model_known {
            filtered_rows.push(row.clone());
        } else {
            skipped_count += 1;
            if !user_known {
                unknown_user_ids.insert(row.user_id.clone());
            }
            if !model_known {
                unknown_model_ids.insert(row.model_id.clone());
            }
        }
    }

    if skipped_count > 0 {
        let sample_users: Vec<_> = unknown_user_ids.iter().take(5).cloned().collect();
        let sample_models: Vec<_> = unknown_model_ids.iter().take(5).cloned().collect();
        log::warn!(
            "Skipped {} rows with unknown entities ({} unknown user_ids, {} unknown model_ids). \
             Sample unknown user_ids: {:?}, sample unknown model_ids: {:?}",
            skipped_count,
            unknown_user_ids.len(),
            unknown_model_ids.len(),
            sample_users,
            sample_models,
        );
    }

    log::info!(
        "Filtered {} CE rows down to {} rows with known users/models",
        rows.len(),
        filtered_rows.len()
    );

    let pool = db::init_pool(&cfg.database_url_cost).await?;
    db::create_cost_table(&pool).await?;
    db::upsert_cost_rows(&pool, &filtered_rows).await?;
    log::info!("Upserted {} rows into cost table", filtered_rows.len());

    Ok(())
}
