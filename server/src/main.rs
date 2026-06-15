mod config;
mod handlers;
mod pages;
pub mod service;

#[cfg(test)]
mod tests;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use handlers::AppState;
use myhandlers::{callback, login, logout};
use service::RealCostService;
use std::sync::Arc;
use tower_sessions::{ExpiredDeletion, Expiry, Session, SessionManagerLayer};

use crate::config::load_config;

#[derive(Parser)]
#[command(name = "cost-explorer")]
struct Args {
    #[arg(long, default_value = "config")]
    config_file: String,
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

    let health_route = Router::new()
        .route("/health", get(handlers::health_check))
        .with_state(state.clone());

    let cost_routes = Router::new()
        .route("/", get(handlers::render_home))
        .route("/costs/daily", get(handlers::render_daily_costs))
        .route("/costs/daily/{date}", get(handlers::render_date_hub))
        .route(
            "/costs/daily/{date}/users",
            get(handlers::render_date_users),
        )
        .route(
            "/costs/daily/{date}/users/{user_id}",
            get(handlers::render_date_models_for_user),
        )
        .route(
            "/costs/daily/{date}/models",
            get(handlers::render_date_models),
        )
        .route(
            "/costs/daily/{date}/models/{model_id}",
            get(handlers::render_date_users_for_model),
        )
        .route("/costs/monthly", get(handlers::render_monthly_costs))
        .route("/costs/monthly/{month}", get(handlers::render_month_hub))
        .route(
            "/costs/monthly/{month}/users",
            get(handlers::render_month_users),
        )
        .route(
            "/costs/monthly/{month}/users/{user_id}",
            get(handlers::render_month_models_for_user),
        )
        .route(
            "/costs/monthly/{month}/models",
            get(handlers::render_month_models),
        )
        .route(
            "/costs/monthly/{month}/models/{model_id}",
            get(handlers::render_month_users_for_model),
        )
        .route("/costs/github", get(handlers::render_github_costs))
        .route("/costs/github/orgs", get(handlers::render_github_orgs))
        .route("/costs/github/repos", get(handlers::render_github_repos))
        .route("/costs/github/orgs/{org}", get(handlers::render_github_org))
        .route(
            "/costs/github/orgs/{org}/{repo}",
            get(handlers::render_github_repo_hub),
        )
        .route(
            "/costs/github/orgs/{org}/{repo}/daily",
            get(handlers::render_github_repo_daily),
        )
        .route(
            "/costs/github/orgs/{org}/{repo}/monthly",
            get(handlers::render_github_repo_monthly),
        )
        .route("/costs/github/daily", get(handlers::render_github_daily))
        .route(
            "/costs/github/daily/{date}",
            get(handlers::render_github_date_hub),
        )
        .route(
            "/costs/github/daily/{date}/orgs",
            get(handlers::render_github_date_orgs),
        )
        .route(
            "/costs/github/daily/{date}/orgs/{org}",
            get(handlers::render_github_date_org),
        )
        .route(
            "/costs/github/daily/{date}/repos",
            get(handlers::render_github_date_repos),
        )
        .route(
            "/costs/github/monthly",
            get(handlers::render_github_monthly),
        )
        .route(
            "/costs/github/monthly/{month}",
            get(handlers::render_github_month_hub),
        )
        .route(
            "/costs/github/monthly/{month}/orgs",
            get(handlers::render_github_month_orgs),
        )
        .route(
            "/costs/github/monthly/{month}/orgs/{org}",
            get(handlers::render_github_month_org),
        )
        .route(
            "/costs/github/monthly/{month}/repos",
            get(handlers::render_github_month_repos),
        )
        .route("/mode/legacy", get(handlers::set_mode_legacy))
        .route("/mode/normal", get(handlers::set_mode_normal))
        .route("/users", get(handlers::render_users))
        .route("/models", get(handlers::render_models))
        .route("/users/{id}", get(handlers::render_user_hub))
        .route("/models/{id}", get(handlers::render_model_hub))
        .route("/users/{id}/daily", get(handlers::render_user_daily_costs))
        .route(
            "/users/{id}/monthly",
            get(handlers::render_user_monthly_costs),
        )
        .route(
            "/models/{id}/daily",
            get(handlers::render_model_daily_costs),
        )
        .route(
            "/models/{id}/monthly",
            get(handlers::render_model_monthly_costs),
        )
        .with_state(state.clone())
        .layer(from_fn_with_state(state, inject_top_bar));

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
        .merge(health_route)
        .merge(cost_routes)
}

/// Response middleware: insert the top navigation bar into every HTML page served by the
/// cost routes. Reads the base path from state and the legacy-view flag from the session.
async fn inject_top_bar(
    State(state): State<AppState>,
    session: Session,
    req: Request,
    next: Next,
) -> Response {
    let legacy_on = handlers::legacy_mode_active(&session).await;
    let on_github = req.uri().path().contains("/costs/github");
    let resp = next.run(req).await;

    let is_html = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);
    if !is_html {
        return resp;
    }

    let (mut parts, body) = resp.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let html = String::from_utf8_lossy(&bytes);

    // `show_legacy` is false in the admin build (admin is its own build, no legacy toggle)
    // and when no legacy email map is configured (the toggle would be a no-op). The default
    // dashboard mode is labeled "Admin" in the admin build, "Normal" otherwise.
    let show_legacy = cfg!(not(feature = "admin")) && !state.legacy_email_map.is_empty();
    let normal_label = if cfg!(feature = "admin") {
        "Admin"
    } else {
        "Normal"
    };
    let bar = templates::top_bar(
        &state.base_path,
        normal_label,
        legacy_on,
        show_legacy,
        on_github,
    );
    let new_html = match html.find("<body>") {
        Some(idx) => {
            let at = idx + "<body>".len();
            format!("{}{}{}", &html[..at], bar, &html[at..])
        }
        None => html.into_owned(),
    };

    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(new_html))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("server=info"));

    let args = Args::parse();

    if cfg!(feature = "admin") {
        log::info!("Running in ADMIN mode (all users visible)");
    } else {
        log::info!("Running in USER mode (per-user filtering)");
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
    log::info!("Gateway DB pool initialized");
    let cost_pool = db::init_pool(&app_config.database_url_cost).await?;
    log::info!("Cost DB connected successfully");

    db::create_cost_table(&cost_pool).await?;

    let session_store = tower_sessions_sqlx_store::PostgresStore::new(cost_pool.clone());
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
        cost_pool,
    };
    let state = AppState {
        service: Arc::new(service),
        base_path: app_config.base_path,
        csv_export: templates::CsvExportLimit {
            max_rows: app_config.csv_export_max_rows,
        },
        legacy_email_map: app_config
            .legacy_email_map
            .into_iter()
            .map(|m| (m.from, m.to))
            .collect(),
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
