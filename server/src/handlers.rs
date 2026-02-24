use axum::extract::{Path, Query, State};
#[cfg(not(feature = "admin"))]
use axum::http::StatusCode;
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

#[cfg(not(feature = "admin"))]
async fn resolve_current_user_id(service: &dyn CostService, email: &str) -> Option<String> {
    service.get_user_id_by_email(email).await
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

    #[cfg(feature = "admin")]
    {
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

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let (total, currency, model_count) = if let Some(ref uid) = current_user_id {
            let costs = state
                .service
                .get_cost_by_model_for_user(&start, &end, uid)
                .await;
            let total: f64 = costs.iter().map(|r| r.amount).sum();
            let currency = costs
                .first()
                .map(|r| r.currency.clone())
                .unwrap_or_else(|| "USD".to_string());
            (total, currency, costs.len())
        } else {
            (0.0, "USD".to_string(), 0)
        };

        Html(pages::home::render(
            &state.base_path,
            &start,
            &end,
            total,
            &currency,
            1,
            model_count,
        ))
        .into_response()
    }
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

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        if let Some(uid) = current_user_id {
            costs
                .into_iter()
                .filter(|c| c.user_id == uid)
                .collect::<Vec<_>>()
        } else {
            costs
        }
    };

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

    #[cfg(feature = "admin")]
    let costs = state.service.get_cost_by_model(&start, &end).await;

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_model_for_user(&start, &end, uid)
                .await
        } else {
            vec![]
        }
    };

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

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        if current_user_id.as_deref() != Some(user_id.as_str()) {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

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

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        if let Some(uid) = current_user_id {
            costs
                .into_iter()
                .filter(|c| c.user_id == uid)
                .collect::<Vec<_>>()
        } else {
            costs
        }
    };

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
