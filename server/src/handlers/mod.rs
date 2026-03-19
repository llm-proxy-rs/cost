use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use tower_sessions::Session;

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

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<dyn CostService>,
    pub base_path: String,
    pub cognito_client_id: String,
    pub cognito_client_secret: String,
    pub cognito_domain: String,
    pub cognito_redirect_uri: String,
    pub cognito_region: String,
    pub cognito_user_pool_id: String,
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
    match period {
        "7d" => {
            let start = today - chrono::Duration::days(6);
            (start, today)
        }
        "month" => {
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .unwrap_or(today);
            (start, today)
        }
        "last_month" => {
            let first_of_current = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .unwrap_or(today);
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
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date)
}

fn get_period(params: &PeriodParams) -> String {
    params.period.as_deref().unwrap_or("30d").to_string()
}

fn get_page(params: &PeriodParams) -> usize {
    params.page.unwrap_or(1).max(1)
}

fn get_sort(params: &PeriodParams) -> Option<usize> {
    params.sort
}

fn get_order(params: &PeriodParams) -> String {
    params
        .order
        .as_deref()
        .unwrap_or("asc")
        .to_string()
}

fn parse_month_range(month: &str) -> (NaiveDate, NaiveDate) {
    let start_str = format!("{}-01", month);
    let start =
        NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive());
    let (y, m) = if start.month() == 12 {
        (start.year() + 1, 1)
    } else {
        (start.year(), start.month() + 1)
    };
    let end = NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap_or(start) - chrono::Duration::days(1);
    (start, end)
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
        let next_month_first =
            NaiveDate::from_ymd_opt(end.year(), end.month(), 1).unwrap() + chrono::Duration::days(31);
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
        let (start, end) = parse_month_range("2024-01");
        assert_eq!(start.to_string(), "2024-01-01");
        assert_eq!(end.to_string(), "2024-01-31");
    }

    #[test]
    fn parse_month_range_february_leap() {
        let (start, end) = parse_month_range("2024-02");
        assert_eq!(start.to_string(), "2024-02-01");
        assert_eq!(end.to_string(), "2024-02-29");
    }

    #[test]
    fn parse_month_range_february_non_leap() {
        let (start, end) = parse_month_range("2023-02");
        assert_eq!(start.to_string(), "2023-02-01");
        assert_eq!(end.to_string(), "2023-02-28");
    }

    #[test]
    fn parse_month_range_december() {
        let (start, end) = parse_month_range("2024-12");
        assert_eq!(start.to_string(), "2024-12-01");
        assert_eq!(end.to_string(), "2024-12-31");
    }
}
