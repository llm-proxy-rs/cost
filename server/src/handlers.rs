use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tower_sessions::Session;

use crate::pages;
use crate::service::CostService;

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
    pub db_pool: Arc<PgPool>,
}

#[derive(Deserialize)]
pub struct DateRangeParams {
    pub start: Option<String>,
    pub end: Option<String>,
}

fn resolve_date_range(params: &DateRangeParams) -> (String, String) {
    let start = params
        .start
        .as_deref()
        .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
        .unwrap_or_else(|| (Utc::now() - chrono::Duration::days(30)).date_naive());

    let end = params
        .end
        .as_deref()
        .and_then(|v| NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
        .unwrap_or_else(|| Utc::now().date_naive());

    (
        start.format("%Y-%m-%d").to_string(),
        end.format("%Y-%m-%d").to_string(),
    )
}

async fn require_login(session: &Session) -> Result<String, Response> {
    match session.get::<String>("email").await {
        Ok(Some(email)) => Ok(email),
        _ => Err(Redirect::to("/login").into_response()),
    }
}

pub async fn home(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let (start, end) = resolve_date_range(&params);

    let daily_cost = state.service.get_daily_cost(&start, &end).await;
    let total: f64 = daily_cost.iter().map(|r| r.amount).sum();
    let currency = daily_cost
        .first()
        .map(|r| r.currency.clone())
        .unwrap_or_else(|| "USD".to_string());

    let users = state.service.list_users().await;
    let models = state.service.list_models().await;

    Html(pages::home::render(
        &state.base_path,
        &start,
        &end,
        total,
        &currency,
        users.len(),
        models.len(),
    ))
    .into_response()
}

pub async fn users(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let (start, end) = resolve_date_range(&params);
    let costs = state.service.get_cost_by_user(&start, &end).await;
    Html(pages::users::render_index(
        &state.base_path,
        &start,
        &end,
        &costs,
    ))
    .into_response()
}

pub async fn models(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let (start, end) = resolve_date_range(&params);
    let costs = state.service.get_cost_by_model(&start, &end).await;
    Html(pages::models::render_index(
        &state.base_path,
        &start,
        &end,
        &costs,
    ))
    .into_response()
}

pub async fn user_detail(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<DateRangeParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let (start, end) = resolve_date_range(&params);
    let user_email = state.service.get_user_email(&user_id).await;
    let costs = state
        .service
        .get_cost_by_model_for_user(&start, &end, &user_id)
        .await;
    Html(pages::users::render_detail(
        &state.base_path,
        &start,
        &end,
        &user_id,
        user_email.as_deref(),
        &costs,
    ))
    .into_response()
}

pub async fn model_detail(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Query(params): Query<DateRangeParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let (start, end) = resolve_date_range(&params);
    let model_name = state.service.get_model_name(&model_id).await;
    let costs = state
        .service
        .get_cost_by_user_for_model(&start, &end, &model_id)
        .await;
    Html(pages::models::render_detail(
        &state.base_path,
        &start,
        &end,
        &model_id,
        model_name.as_deref(),
        &costs,
    ))
    .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_date_range_defaults() {
        let params = DateRangeParams {
            start: None,
            end: None,
        };
        let (start, end) = resolve_date_range(&params);
        assert!(NaiveDate::parse_from_str(&start, "%Y-%m-%d").is_ok());
        assert!(NaiveDate::parse_from_str(&end, "%Y-%m-%d").is_ok());
        assert!(start < end);
    }

    #[test]
    fn resolve_date_range_with_values() {
        let params = DateRangeParams {
            start: Some("2024-01-01".to_string()),
            end: Some("2024-01-31".to_string()),
        };
        let (start, end) = resolve_date_range(&params);
        assert_eq!(start, "2024-01-01");
        assert_eq!(end, "2024-01-31");
    }

    #[test]
    fn resolve_date_range_invalid_falls_back() {
        let params = DateRangeParams {
            start: Some("not-a-date".to_string()),
            end: Some("also-bad".to_string()),
        };
        let (start, end) = resolve_date_range(&params);
        assert!(NaiveDate::parse_from_str(&start, "%Y-%m-%d").is_ok());
        assert!(NaiveDate::parse_from_str(&end, "%Y-%m-%d").is_ok());
    }
}
