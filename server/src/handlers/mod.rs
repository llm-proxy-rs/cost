use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use tower_sessions::Session;

use crate::pages;
use crate::service::CostService;

#[cfg(feature = "admin")]
mod admin;
#[cfg(feature = "admin")]
pub use admin::*;

#[cfg(not(feature = "admin"))]
mod user;
#[cfg(not(feature = "admin"))]
pub use user::*;

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        log::error!("Internal error: {}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

pub async fn health_check(State(state): State<AppState>) -> Response {
    match state.service.health_check().await {
        Ok(()) => (StatusCode::OK, "ok").into_response(),
        Err(e) => {
            log::error!("Health check failed: {e}");
            (StatusCode::SERVICE_UNAVAILABLE, format!("error: {e}")).into_response()
        }
    }
}

/// GitHub landing — a hub presenting the two tag dimensions (By Org / By Repo). Shared by
/// all builds: org-wide infra cost, login-only, no per-user filtering or legacy remap.
pub async fn render_github_costs(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let (start, end) = resolve_period(&period);

    let orgs = state.service.get_github_orgs(start, end).await?;
    let total: f64 = orgs.iter().map(|o| o.amount).sum();
    let currency = orgs.first().map(|o| o.currency.as_str()).unwrap_or("USD"); // no rows: default to USD
    let repo_count = state.service.get_cost_by_github(start, end).await?.len();

    Ok(Html(pages::github::render_hub(
        &state.base_path,
        &period,
        total,
        currency,
        orgs.len(),
        repo_count,
    ))
    .into_response())
}

/// By Org: cost grouped by GithubOrgName.
pub async fn render_github_orgs(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    let orgs = state.service.get_github_orgs(start, end).await?;

    Ok(Html(pages::github::render_orgs(
        &state.base_path,
        &period,
        page,
        &orgs,
        sort,
        &order,
    ))
    .into_response())
}

/// By Repo: flat cost grouped by GithubOrgName + GithubRepoName.
pub async fn render_github_repos(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    let costs = state.service.get_cost_by_github(start, end).await?;

    Ok(Html(pages::github::render_repos(
        &state.base_path,
        &period,
        page,
        &costs,
        sort,
        &order,
    ))
    .into_response())
}

/// GitHub cost for one org — lists its repos.
pub async fn render_github_org(
    session: Session,
    State(state): State<AppState>,
    Path(org): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    let repos = state
        .service
        .get_github_repos_for_org(start, end, &org)
        .await?;

    Ok(Html(pages::github::render_org(
        &state.base_path,
        &period,
        page,
        &org,
        &repos,
        sort,
        &order,
    ))
    .into_response())
}

/// GitHub repo hub — total + Daily/Monthly subpages.
pub async fn render_github_repo_hub(
    session: Session,
    State(state): State<AppState>,
    Path((org, repo)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let (start, end) = resolve_period(&period);

    let daily = state
        .service
        .get_github_daily_for_repo(start, end, &org, &repo)
        .await?;
    let total: f64 = daily.iter().map(|r| r.amount).sum();
    let currency = daily.first().map(|r| r.currency.as_str()).unwrap_or("USD"); // no rows: default to USD

    Ok(Html(pages::github::render_repo_hub(
        &state.base_path,
        &period,
        &org,
        &repo,
        total,
        currency,
    ))
    .into_response())
}

pub async fn render_github_repo_daily(
    session: Session,
    State(state): State<AppState>,
    Path((org, repo)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    let costs = state
        .service
        .get_github_daily_for_repo(start, end, &org, &repo)
        .await?;
    let costs = pages::sort_records(costs, sort, &order);

    Ok(Html(pages::github::render_repo_daily(
        &state.base_path,
        &period,
        page,
        &org,
        &repo,
        &costs,
    ))
    .into_response())
}

pub async fn render_github_repo_monthly(
    session: Session,
    State(state): State<AppState>,
    Path((org, repo)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    let costs = state
        .service
        .get_github_monthly_for_repo(snap_to_month_start(start), end, &org, &repo)
        .await?;
    let costs = pages::sort_records(costs, sort, &order);

    Ok(Html(pages::github::render_repo_monthly(
        &state.base_path,
        &period,
        page,
        &org,
        &repo,
        &costs,
    ))
    .into_response())
}

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<dyn CostService>,
    pub base_path: String,
    pub legacy_email_map: Vec<(String, String)>,
    pub cognito_client_id: String,
    pub cognito_client_secret: String,
    pub cognito_domain: String,
    pub cognito_redirect_uri: String,
    pub cognito_region: String,
    pub cognito_user_pool_id: String,
}

/// Whether the current session has selected the legacy view.
pub async fn legacy_mode_active(session: &Session) -> bool {
    session
        .get::<bool>("legacy_mode")
        .await
        .ok()
        .flatten()
        // No flag set (or session read failed): default to the normal, non-legacy view.
        .unwrap_or(false)
}

/// The service to use for this request: in legacy mode (and with a configured map) the
/// real service is wrapped so user emails are remapped for both lookup and display;
/// otherwise the plain service is used. Only the non-admin (user) handlers consult it.
#[cfg(not(feature = "admin"))]
pub async fn effective_service(state: &AppState, session: &Session) -> Arc<dyn CostService> {
    if !state.legacy_email_map.is_empty() && legacy_mode_active(session).await {
        Arc::new(crate::service::LegacyEmailService::new(
            state.service.clone(),
            state.legacy_email_map.clone(),
        ))
    } else {
        state.service.clone()
    }
}

/// Top-bar mode toggle: set the session's legacy flag and return to the dashboard. Requires
/// login but not a profile, so a user whose email migrated (and 403s on the dashboard) can
/// still switch into the legacy view.
pub async fn set_mode_legacy(
    session: Session,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    if let Err(redirect) = require_login(&session).await {
        return Ok(redirect);
    }
    let _ = session.insert("legacy_mode", true).await;
    Ok(Redirect::to(&crate::pages::make_path(&state.base_path, "")).into_response())
}

pub async fn set_mode_normal(
    session: Session,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    if let Err(redirect) = require_login(&session).await {
        return Ok(redirect);
    }
    let _ = session.insert("legacy_mode", false).await;
    Ok(Redirect::to(&crate::pages::make_path(&state.base_path, "")).into_response())
}

#[derive(Deserialize)]
pub struct PeriodParams {
    pub period: Option<String>,
    pub page: Option<usize>,
    pub sort: Option<usize>,
    pub order: Option<String>,
}

fn resolve_period(period: &str) -> (NaiveDate, NaiveDate) {
    let today = Utc::now().date_naive();
    // The `from_ymd_opt(year, month, 1)` calls below can only return None for an invalid
    // year/month, which never happens for a real date's own fields — the fallbacks are
    // unreachable in practice and just keep the function total.
    match period {
        "7d" => {
            let start = today - chrono::Duration::days(6);
            (start, today)
        }
        "month" => {
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
            (start, today)
        }
        "last_month" => {
            let first_of_current =
                NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
            let last_of_prev = first_of_current - chrono::Duration::days(1);
            let first_of_prev =
                NaiveDate::from_ymd_opt(last_of_prev.year(), last_of_prev.month(), 1)
                    .unwrap_or(last_of_prev);
            (first_of_prev, last_of_prev)
        }
        "3m" => {
            let start = today - chrono::Duration::days(90);
            (start, today)
        }
        "6m" => {
            let start = today - chrono::Duration::days(180);
            (start, today)
        }
        "12m" => {
            let start = today - chrono::Duration::days(365);
            (start, today)
        }
        _ => {
            // default: 30d
            let start = today - chrono::Duration::days(29);
            (start, today)
        }
    }
}

fn snap_to_month_start(date: NaiveDate) -> NaiveDate {
    // Day 1 of a valid date's year/month always exists; fall back to the input otherwise.
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
}

fn get_period(params: &PeriodParams) -> String {
    params.period.as_deref().unwrap_or("30d").to_string() // default period when unset
}

fn get_page(params: &PeriodParams) -> usize {
    params.page.unwrap_or(1).max(1) // default to the first page when unset
}

fn get_sort(params: &PeriodParams) -> Option<usize> {
    params.sort
}

fn get_order(params: &PeriodParams) -> String {
    params
        .order
        .as_deref()
        .unwrap_or("asc") // default sort direction when unset
        .to_string()
}

fn parse_date(date: &str) -> Result<NaiveDate, Box<Response>> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        Box::new((StatusCode::BAD_REQUEST, format!("Invalid date: {date}")).into_response())
    })
}

fn parse_month_range(month: &str) -> Result<(NaiveDate, NaiveDate), Box<Response>> {
    let start_str = format!("{}-01", month);
    let start = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").map_err(|_| {
        Box::new((StatusCode::BAD_REQUEST, format!("Invalid month: {month}")).into_response())
    })?;
    let (y, m) = if start.month() == 12 {
        (start.year() + 1, 1)
    } else {
        (start.year(), start.month() + 1)
    };
    // First of the month after `start`; `from_ymd_opt` here can't fail (m is 1..=12),
    // the fallback to `start` just keeps it total. Minus one day = last day of the month.
    let end = NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(start) - chrono::Duration::days(1);
    Ok((start, end))
}

async fn require_login(session: &Session) -> Result<String, Response> {
    match session.get::<String>("email").await {
        Ok(Some(email)) => Ok(email),
        _ => Err(Redirect::to("/login").into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_period_7d() {
        let (start, end) = resolve_period("7d");
        assert_eq!((end - start).num_days(), 6);
    }

    #[test]
    fn resolve_period_30d() {
        let (start, end) = resolve_period("30d");
        assert_eq!((end - start).num_days(), 29);
    }

    #[test]
    fn resolve_period_month() {
        let (start, end) = resolve_period("month");
        assert_eq!(start.day(), 1);
        assert_eq!(start.month(), end.month());
    }

    #[test]
    fn resolve_period_last_month() {
        let (start, end) = resolve_period("last_month");
        assert_eq!(start.day(), 1);
        assert_eq!(start.month(), end.month());
        let next_month_first = NaiveDate::from_ymd_opt(end.year(), end.month(), 1).unwrap()
            + chrono::Duration::days(31);
        let last_day =
            NaiveDate::from_ymd_opt(next_month_first.year(), next_month_first.month(), 1).unwrap()
                - chrono::Duration::days(1);
        assert!(end.day() >= 28);
        assert_eq!(end, last_day);
    }

    #[test]
    fn resolve_period_3m() {
        let (start, end) = resolve_period("3m");
        assert_eq!((end - start).num_days(), 90);
    }

    #[test]
    fn resolve_period_6m() {
        let (start, end) = resolve_period("6m");
        assert_eq!((end - start).num_days(), 180);
    }

    #[test]
    fn resolve_period_12m() {
        let (start, end) = resolve_period("12m");
        assert_eq!((end - start).num_days(), 365);
    }

    #[test]
    fn resolve_period_default() {
        let (start, end) = resolve_period("unknown");
        assert_eq!((end - start).num_days(), 29);
    }

    #[test]
    fn get_period_default() {
        let params = PeriodParams {
            period: None,
            page: None,
            sort: None,
            order: None,
        };
        assert_eq!(get_period(&params), "30d");
    }

    #[test]
    fn get_period_specified() {
        let params = PeriodParams {
            period: Some("7d".to_string()),
            page: None,
            sort: None,
            order: None,
        };
        assert_eq!(get_period(&params), "7d");
    }

    #[test]
    fn parse_month_range_january() {
        let (start, end) = parse_month_range("2024-01").unwrap();
        assert_eq!(start.to_string(), "2024-01-01");
        assert_eq!(end.to_string(), "2024-01-31");
    }

    #[test]
    fn parse_month_range_february_leap() {
        let (start, end) = parse_month_range("2024-02").unwrap();
        assert_eq!(start.to_string(), "2024-02-01");
        assert_eq!(end.to_string(), "2024-02-29");
    }

    #[test]
    fn parse_month_range_february_non_leap() {
        let (start, end) = parse_month_range("2023-02").unwrap();
        assert_eq!(start.to_string(), "2023-02-01");
        assert_eq!(end.to_string(), "2023-02-28");
    }

    #[test]
    fn parse_month_range_december() {
        let (start, end) = parse_month_range("2024-12").unwrap();
        assert_eq!(start.to_string(), "2024-12-01");
        assert_eq!(end.to_string(), "2024-12-31");
    }
}
