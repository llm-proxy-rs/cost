use anyhow::Result;
use chrono::{NaiveDate, Utc};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "cost-batch")]
struct Args {
    #[arg(long, default_value = "config")]
    config_file: String,

    #[arg(long)]
    backfill: bool,

    #[arg(long)]
    start: Option<String>,

    #[arg(long)]
    end: Option<String>,
}

#[derive(Deserialize)]
struct BatchConfig {
    database_url_cost: String,
    database_url_gateway_ro: String,
}

fn load_config(config_file: &str) -> Result<BatchConfig> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name(config_file).required(false))
        .add_source(config::Environment::default())
        .build()?;
    let cfg: BatchConfig = settings.try_deserialize()?;
    Ok(cfg)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("batch=info"));

    let args = Args::parse();
    let cfg = load_config(&args.config_file)?;

    let today = Utc::now().date_naive();

    let (start, end) = if let (Some(s), Some(e)) = (&args.start, &args.end) {
        // Validate date format
        let _ = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
        let _ = NaiveDate::parse_from_str(e, "%Y-%m-%d")?;
        (s.clone(), e.clone())
    } else if args.backfill {
        let start_date = today - chrono::Months::new(14);
        (
            start_date.format("%Y-%m-%d").to_string(),
            today.format("%Y-%m-%d").to_string(),
        )
    } else {
        // Incremental: last 3 days
        let start_date = today - chrono::Duration::days(3);
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
