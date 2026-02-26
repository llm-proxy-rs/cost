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

    let pool = db::init_pool(&cfg.database_url_cost).await?;
    db::create_cost_table(&pool).await?;
    db::upsert_cost_rows(&pool, &rows).await?;
    log::info!("Upserted {} rows into cost table", rows.len());

    Ok(())
}
