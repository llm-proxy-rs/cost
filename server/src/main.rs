mod config;
mod handlers;
mod pages;
pub mod service;

#[cfg(test)]
mod tests;

use axum::routing::get;
use axum::Router;
use clap::Parser;
use handlers::AppState;
use myhandlers::{callback, login, logout};
use service::{DemoCostService, RealCostService};
use std::sync::Arc;
use tower_sessions::{ExpiredDeletion, Expiry, MemoryStore, SessionManagerLayer};

use crate::config::load_config;

#[derive(Parser)]
#[command(name = "cost-explorer")]
struct Args {
    #[arg(long, default_value = "config")]
    config_file: String,

    #[arg(long)]
    demo: bool,
}

pub fn build_router(state: AppState) -> Router {
    let base = state.base_path.clone();

    let auth_state = myhandlers::AppState {
        cognito_client_id: state.cognito_client_id.clone(),
        cognito_client_secret: state.cognito_client_secret.clone(),
        cognito_domain: state.cognito_domain.clone(),
        cognito_redirect_uri: state.cognito_redirect_uri.clone(),
        cognito_region: state.cognito_region.clone(),
        cognito_user_pool_id: state.cognito_user_pool_id.clone(),
    };

    let cost_routes = Router::new()
        .route("/", get(handlers::home))
        .route("/costs/daily", get(handlers::daily_costs))
        .route("/costs/daily/{date}", get(handlers::cost_date_detail))
        .route("/costs/daily/{date}/users", get(handlers::cost_date_users))
        .route(
            "/costs/daily/{date}/users/{user_id}",
            get(handlers::cost_date_user_models),
        )
        .route(
            "/costs/daily/{date}/models",
            get(handlers::cost_date_models),
        )
        .route(
            "/costs/daily/{date}/models/{model_id}",
            get(handlers::cost_date_model_users),
        )
        .route("/costs/monthly", get(handlers::monthly_costs))
        .route("/costs/monthly/{month}", get(handlers::cost_month_detail))
        .route(
            "/costs/monthly/{month}/users",
            get(handlers::cost_month_users),
        )
        .route(
            "/costs/monthly/{month}/users/{user_id}",
            get(handlers::cost_month_user_models),
        )
        .route(
            "/costs/monthly/{month}/models",
            get(handlers::cost_month_models),
        )
        .route(
            "/costs/monthly/{month}/models/{model_id}",
            get(handlers::cost_month_model_users),
        )
        .route("/users", get(handlers::users))
        .route("/models", get(handlers::models))
        .route("/users/{id}", get(handlers::user_detail))
        .route("/models/{id}", get(handlers::model_detail))
        .route("/users/{id}/daily", get(handlers::user_daily_costs))
        .route("/users/{id}/monthly", get(handlers::user_monthly_costs))
        .route("/models/{id}/daily", get(handlers::model_daily_costs))
        .route("/models/{id}/monthly", get(handlers::model_monthly_costs))
        .with_state(state);

    let cost_routes = if base == "/" {
        cost_routes
    } else {
        Router::new().nest(&base, cost_routes)
    };

    Router::new()
        .route("/callback", get(callback))
        .route("/login", get(login))
        .route("/logout", get(logout))
        .with_state(auth_state)
        .merge(cost_routes)
}

pub fn build_demo_router(state: AppState) -> Router {
    let base = state.base_path.clone();

    let cost_routes = Router::new()
        .route("/", get(handlers::home))
        .route("/costs/daily", get(handlers::daily_costs))
        .route("/costs/daily/{date}", get(handlers::cost_date_detail))
        .route("/costs/daily/{date}/users", get(handlers::cost_date_users))
        .route(
            "/costs/daily/{date}/users/{user_id}",
            get(handlers::cost_date_user_models),
        )
        .route(
            "/costs/daily/{date}/models",
            get(handlers::cost_date_models),
        )
        .route(
            "/costs/daily/{date}/models/{model_id}",
            get(handlers::cost_date_model_users),
        )
        .route("/costs/monthly", get(handlers::monthly_costs))
        .route("/costs/monthly/{month}", get(handlers::cost_month_detail))
        .route(
            "/costs/monthly/{month}/users",
            get(handlers::cost_month_users),
        )
        .route(
            "/costs/monthly/{month}/users/{user_id}",
            get(handlers::cost_month_user_models),
        )
        .route(
            "/costs/monthly/{month}/models",
            get(handlers::cost_month_models),
        )
        .route(
            "/costs/monthly/{month}/models/{model_id}",
            get(handlers::cost_month_model_users),
        )
        .route("/users", get(handlers::users))
        .route("/models", get(handlers::models))
        .route("/users/{id}", get(handlers::user_detail))
        .route("/models/{id}", get(handlers::model_detail))
        .route("/users/{id}/daily", get(handlers::user_daily_costs))
        .route("/users/{id}/monthly", get(handlers::user_monthly_costs))
        .route("/models/{id}/daily", get(handlers::model_daily_costs))
        .route("/models/{id}/monthly", get(handlers::model_monthly_costs))
        .with_state(state);

    let cost_routes = if base == "/" {
        cost_routes
    } else {
        Router::new().nest(&base, cost_routes)
    };

    Router::new()
        .route("/login", get(handlers::demo_login))
        .merge(cost_routes)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("server=info"));

    let args = Args::parse();

    if args.demo {
        log::info!("Running in DEMO mode");

        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_expiry(Expiry::OnInactivity(time::Duration::seconds(86400)))
            .with_same_site(tower_sessions::cookie::SameSite::Lax);

        let state = AppState {
            service: Arc::new(DemoCostService),
            base_path: "/".to_string(),
            cognito_client_id: String::new(),
            cognito_client_secret: String::new(),
            cognito_domain: String::new(),
            cognito_redirect_uri: String::new(),
            cognito_region: String::new(),
            cognito_user_pool_id: String::new(),
        };

        let app = build_demo_router(state).layer(session_layer);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
        log::info!("Listening on http://127.0.0.1:8080");

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal_simple())
            .await?;

        return Ok(());
    }

    if cfg!(feature = "admin") {
        log::info!("Running in ADMIN mode (all users visible)");
    } else {
        log::info!("Running in NORMAL mode (per-user filtering)");
    }

    let app_config = load_config(&args.config_file).await?;

    if app_config.cognito_client_id.is_empty()
        || app_config.cognito_client_secret.is_empty()
        || app_config.cognito_domain.is_empty()
    {
        log::error!(
            "Missing required Cognito configuration. Check config file or environment variables."
        );
    }

    let gateway_pool = db::init_pool_lazy(&app_config.database_url_gateway_ro)?;
    let cost_pool = db::init_pool(&app_config.database_url_cost).await?;
    let cache_pool = cost_pool.clone();
    let ce_client = ce::new_client().await;

    db::create_cost_cache_table(&cache_pool).await?;

    let session_store = tower_sessions_sqlx_store::PostgresStore::new(cost_pool);
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(3600)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(86400)))
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let service = RealCostService {
        pool: gateway_pool,
        ce_client,
        cache_pool,
    };
    let state = AppState {
        service: Arc::new(service),
        base_path: app_config.base_path,
        cognito_client_id: app_config.cognito_client_id,
        cognito_client_secret: app_config.cognito_client_secret,
        cognito_domain: app_config.cognito_domain,
        cognito_redirect_uri: app_config.cognito_redirect_uri,
        cognito_region: app_config.cognito_region,
        cognito_user_pool_id: app_config.cognito_user_pool_id,
    };

    let app = build_router(state).layer(session_layer);

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", app_config.host, app_config.port)).await?;
    log::info!(
        "Listening on http://{}:{}",
        app_config.host,
        app_config.port
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await?;

    deletion_task.await??;

    Ok(())
}

async fn shutdown_signal(deletion_task_abort_handle: tokio::task::AbortHandle) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}

async fn shutdown_signal_simple() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
