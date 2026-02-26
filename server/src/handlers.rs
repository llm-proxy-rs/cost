use axum::extract::{Path, Query, State};
#[cfg(not(feature = "admin"))]
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
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
}

#[derive(Deserialize)]
pub struct PeriodParams {
    pub period: Option<String>,
    pub page: Option<usize>,
}

fn resolve_period(period: &str) -> (String, String) {
    let today = Utc::now().date_naive();
    match period {
        "7d" => {
            let start = today - chrono::Duration::days(6);
            (
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
        "month" => {
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            (
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
        "last_month" => {
            let first_of_current = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            let last_of_prev = first_of_current - chrono::Duration::days(1);
            let first_of_prev =
                NaiveDate::from_ymd_opt(last_of_prev.year(), last_of_prev.month(), 1).unwrap();
            (
                first_of_prev.format("%Y-%m-%d").to_string(),
                last_of_prev.format("%Y-%m-%d").to_string(),
            )
        }
        "3m" => {
            let start = today - chrono::Duration::days(90);
            (
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
        "6m" => {
            let start = today - chrono::Duration::days(180);
            (
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
        "12m" => {
            let start = today - chrono::Duration::days(365);
            (
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
        _ => {
            // default: 30d
            let start = today - chrono::Duration::days(29);
            (
                start.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string(),
            )
        }
    }
}

fn get_period(params: &PeriodParams) -> String {
    params.period.as_deref().unwrap_or("30d").to_string()
}

fn get_page(params: &PeriodParams) -> usize {
    params.page.unwrap_or(1).max(1)
}

fn month_to_range(month: &str) -> (String, String) {
    let start = format!("{}-01", month);
    let parsed =
        NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive());
    let (y, m) = if parsed.month() == 12 {
        (parsed.year() + 1, 1)
    } else {
        (parsed.year(), parsed.month() + 1)
    };
    let last_day = NaiveDate::from_ymd_opt(y, m, 1).unwrap() - chrono::Duration::days(1);
    let end = last_day.format("%Y-%m-%d").to_string();
    (start, end)
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
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(&start, &end).await;
        let monthly_cost = state.service.get_monthly_cost(&start, &end).await;
        let users = state.service.list_users().await;
        let models = state.service.list_models().await;

        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");

        Html(pages::home::render(
            &state.base_path,
            &period,
            total_cost,
            currency,
            daily_cost.len(),
            monthly_cost.len(),
            users.len(),
            models.len(),
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let daily_cost = if let Some(ref uid) = current_user_id {
            state.service.get_daily_cost_for_user(&start, &end, uid).await
        } else {
            vec![]
        };
        let monthly_cost = if let Some(ref uid) = current_user_id {
            state.service.get_monthly_cost_for_user(&start, &end, uid).await
        } else {
            vec![]
        };
        let model_count = if let Some(ref uid) = current_user_id {
            let costs = state
                .service
                .get_cost_by_model_for_user(&start, &end, uid)
                .await;
            costs.len()
        } else {
            0
        };

        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");

        Html(pages::home::render(
            &state.base_path,
            &period,
            total_cost,
            currency,
            daily_cost.len(),
            monthly_cost.len(),
            1,
            model_count,
        ))
        .into_response()
    }
}

pub async fn daily_costs(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(&start, &end).await;

        Html(pages::costs::render(
            &state.base_path,
            &period,
            page,
            &daily_cost,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let daily_cost = if let Some(ref _uid) = current_user_id {
            state.service.get_daily_cost(&start, &end).await
        } else {
            vec![]
        };

        Html(pages::costs::render(
            &state.base_path,
            &period,
            page,
            &daily_cost,
        ))
        .into_response()
    }
}

pub async fn users(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let users_enriched = state.service.list_users_enriched().await;
        let costs = state.service.get_cost_by_user(&start, &end).await;

        Html(pages::users::render_index(
            &state.base_path,
            &period,
            page,
            &users_enriched,
            &costs,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = state.service.get_cost_by_user(&start, &end).await;
        let costs: Vec<_> = if let Some(ref uid) = current_user_id {
            costs.into_iter().filter(|c| c.user_id == *uid).collect()
        } else {
            costs
        };
        let users_enriched = state.service.list_users_enriched().await;
        let users_enriched: Vec<_> = if let Some(ref uid) = current_user_id {
            users_enriched
                .into_iter()
                .filter(|u| u.user_id == *uid)
                .collect()
        } else {
            users_enriched
        };

        Html(pages::users::render_index(
            &state.base_path,
            &period,
            page,
            &users_enriched,
            &costs,
        ))
        .into_response()
    }
}

pub async fn models(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let models_enriched = state.service.list_models_enriched().await;
        let costs = state.service.get_cost_by_model(&start, &end).await;

        Html(pages::models::render_index(
            &state.base_path,
            &period,
            page,
            &models_enriched,
            &costs,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_model_for_user(&start, &end, uid)
                .await
        } else {
            vec![]
        };
        let models_enriched = state.service.list_models_enriched().await;

        Html(pages::models::render_index(
            &state.base_path,
            &period,
            page,
            &models_enriched,
            &costs,
        ))
        .into_response()
    }
}

pub async fn user_detail(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<PeriodParams>,
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

    let period = get_period(&params);
    let user_info = state.service.get_user_info(&user_id).await;
    match user_info {
        Some(info) => {
            Html(pages::users::render_hub(&state.base_path, &period, &info)).into_response()
        }
        None => {
            // Fallback: construct minimal UserInfo from email lookup
            let user_email = state
                .service
                .get_user_email(&user_id)
                .await
                .unwrap_or_else(|| "unknown".to_string());
            let info = common::UserInfo {
                user_id: user_id.clone(),
                user_email,
                created_at: String::new(),
                api_key_count: 0,
                active_api_key_count: 0,
                inference_profile_count: 0,
            };
            Html(pages::users::render_hub(&state.base_path, &period, &info)).into_response()
        }
    }
}

pub async fn user_daily_costs(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<PeriodParams>,
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

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_daily_cost_for_user(&start, &end, &user_id)
        .await;

    Html(pages::users::render_daily_costs(
        &state.base_path,
        &period,
        page,
        &user_id,
        &user_email,
        &costs,
    ))
    .into_response()
}

pub async fn user_monthly_costs(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<PeriodParams>,
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

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_monthly_cost_for_user(&start, &end, &user_id)
        .await;

    Html(pages::users::render_monthly_costs(
        &state.base_path,
        &period,
        page,
        &user_id,
        &user_email,
        &costs,
    ))
    .into_response()
}

pub async fn model_detail(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let model_info = state.service.get_model_info(&model_id).await;
    match model_info {
        Some(info) => {
            Html(pages::models::render_hub(&state.base_path, &period, &info)).into_response()
        }
        None => {
            let model_name = state
                .service
                .get_model_name(&model_id)
                .await
                .unwrap_or_else(|| "unknown".to_string());
            let info = common::ModelInfo {
                model_id: model_id.clone(),
                model_name,
                is_disabled: false,
                protected: false,
                user_count: 0,
            };
            Html(pages::models::render_hub(&state.base_path, &period, &info)).into_response()
        }
    }
}

pub async fn model_daily_costs(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_daily_cost_for_model(&start, &end, &model_id)
        .await;

    Html(pages::models::render_daily_costs(
        &state.base_path,
        &period,
        page,
        &model_id,
        &model_name,
        &costs,
    ))
    .into_response()
}

pub async fn model_monthly_costs(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_monthly_cost_for_model(&start, &end, &model_id)
        .await;

    Html(pages::models::render_monthly_costs(
        &state.base_path,
        &period,
        page,
        &model_id,
        &model_name,
        &costs,
    ))
    .into_response()
}

// --- Daily cost drill-down handlers ---

pub async fn cost_date_detail(
    session: Session,
    State(state): State<AppState>,
    Path(date): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(&date, &date).await;
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = state.service.get_cost_by_user(&date, &date).await;
        let models = state.service.get_cost_by_model(&date, &date).await;

        Html(pages::costs::render_hub(
            &state.base_path,
            &period,
            &date,
            total_cost,
            currency,
            users.len(),
            models.len(),
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let daily_cost = if current_user_id.is_some() {
            state.service.get_daily_cost(&date, &date).await
        } else {
            vec![]
        };
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = if let Some(ref uid) = current_user_id {
            let all = state.service.get_cost_by_user(&date, &date).await;
            all.into_iter()
                .filter(|c| c.user_id == *uid)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let models = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_model_for_user(&date, &date, uid)
                .await
        } else {
            vec![]
        };

        Html(pages::costs::render_hub(
            &state.base_path,
            &period,
            &date,
            total_cost,
            currency,
            users.len(),
            models.len(),
        ))
        .into_response()
    }
}

pub async fn cost_date_users(
    session: Session,
    State(state): State<AppState>,
    Path(date): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_user(&date, &date).await;

        Html(pages::costs::render_users(
            &state.base_path,
            &period,
            page,
            &date,
            &costs,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = state.service.get_cost_by_user(&date, &date).await;
        let costs: Vec<_> = if let Some(ref uid) = current_user_id {
            costs.into_iter().filter(|c| c.user_id == *uid).collect()
        } else {
            costs
        };

        Html(pages::costs::render_users(
            &state.base_path,
            &period,
            page,
            &date,
            &costs,
        ))
        .into_response()
    }
}

pub async fn cost_date_models(
    session: Session,
    State(state): State<AppState>,
    Path(date): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_model(&date, &date).await;

        Html(pages::costs::render_models(
            &state.base_path,
            &period,
            page,
            &date,
            &costs,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_model_for_user(&date, &date, uid)
                .await
        } else {
            vec![]
        };

        Html(pages::costs::render_models(
            &state.base_path,
            &period,
            page,
            &date,
            &costs,
        ))
        .into_response()
    }
}

pub async fn cost_date_user_models(
    session: Session,
    State(state): State<AppState>,
    Path((date, user_id)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_model_for_user(&date, &date, &user_id)
        .await;

    Html(pages::costs::render_user_models(
        &state.base_path,
        &period,
        page,
        &date,
        &user_email,
        &costs,
    ))
    .into_response()
}

pub async fn cost_date_model_users(
    session: Session,
    State(state): State<AppState>,
    Path((date, model_id)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_user_for_model(&date, &date, &model_id)
        .await;

    Html(pages::costs::render_model_users(
        &state.base_path,
        &period,
        page,
        &date,
        &model_name,
        &costs,
    ))
    .into_response()
}

// --- Monthly cost handlers ---

pub async fn monthly_costs(
    session: Session,
    State(state): State<AppState>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let monthly_cost = state.service.get_monthly_cost(&start, &end).await;

        Html(pages::monthly::render(
            &state.base_path,
            &period,
            page,
            &monthly_cost,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let monthly_cost = if current_user_id.is_some() {
            state.service.get_monthly_cost(&start, &end).await
        } else {
            vec![]
        };

        Html(pages::monthly::render(
            &state.base_path,
            &period,
            page,
            &monthly_cost,
        ))
        .into_response()
    }
}

pub async fn cost_month_detail(
    session: Session,
    State(state): State<AppState>,
    Path(month): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let (start, end) = month_to_range(&month);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(&start, &end).await;
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = state.service.get_cost_by_user(&start, &end).await;
        let models = state.service.get_cost_by_model(&start, &end).await;

        Html(pages::monthly::render_hub(
            &state.base_path,
            &period,
            &month,
            total_cost,
            currency,
            users.len(),
            models.len(),
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let daily_cost = if current_user_id.is_some() {
            state.service.get_daily_cost(&start, &end).await
        } else {
            vec![]
        };
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = if let Some(ref uid) = current_user_id {
            let all = state.service.get_cost_by_user(&start, &end).await;
            all.into_iter()
                .filter(|c| c.user_id == *uid)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let models = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_model_for_user(&start, &end, uid)
                .await
        } else {
            vec![]
        };

        Html(pages::monthly::render_hub(
            &state.base_path,
            &period,
            &month,
            total_cost,
            currency,
            users.len(),
            models.len(),
        ))
        .into_response()
    }
}

pub async fn cost_month_users(
    session: Session,
    State(state): State<AppState>,
    Path(month): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = month_to_range(&month);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_user(&start, &end).await;

        Html(pages::monthly::render_users(
            &state.base_path,
            &period,
            page,
            &month,
            &costs,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = state.service.get_cost_by_user(&start, &end).await;
        let costs: Vec<_> = if let Some(ref uid) = current_user_id {
            costs.into_iter().filter(|c| c.user_id == *uid).collect()
        } else {
            costs
        };

        Html(pages::monthly::render_users(
            &state.base_path,
            &period,
            page,
            &month,
            &costs,
        ))
        .into_response()
    }
}

pub async fn cost_month_models(
    session: Session,
    State(state): State<AppState>,
    Path(month): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = month_to_range(&month);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_model(&start, &end).await;

        Html(pages::monthly::render_models(
            &state.base_path,
            &period,
            page,
            &month,
            &costs,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_model_for_user(&start, &end, uid)
                .await
        } else {
            vec![]
        };

        Html(pages::monthly::render_models(
            &state.base_path,
            &period,
            page,
            &month,
            &costs,
        ))
        .into_response()
    }
}

pub async fn cost_month_user_models(
    session: Session,
    State(state): State<AppState>,
    Path((month, user_id)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = month_to_range(&month);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_model_for_user(&start, &end, &user_id)
        .await;

    Html(pages::monthly::render_user_models(
        &state.base_path,
        &period,
        page,
        &month,
        &user_email,
        &costs,
    ))
    .into_response()
}

pub async fn cost_month_model_users(
    session: Session,
    State(state): State<AppState>,
    Path((month, model_id)): Path<(String, String)>,
    Query(params): Query<PeriodParams>,
) -> Response {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return redirect,
    };

    let period = get_period(&params);
    let page = get_page(&params);
    let (start, end) = month_to_range(&month);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_user_for_model(&start, &end, &model_id)
        .await;

    Html(pages::monthly::render_model_users(
        &state.base_path,
        &period,
        page,
        &month,
        &model_name,
        &costs,
    ))
    .into_response()
}

pub async fn demo_login(session: Session) -> Response {
    let _ = session.insert("email", "alice@example.com").await;
    Redirect::to("/").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_period_7d() {
        let (start, end) = resolve_period("7d");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!((e - s).num_days(), 6);
    }

    #[test]
    fn resolve_period_30d() {
        let (start, end) = resolve_period("30d");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!((e - s).num_days(), 29);
    }

    #[test]
    fn resolve_period_month() {
        let (start, end) = resolve_period("month");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        assert_eq!(s.day(), 1);
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!(s.month(), e.month());
    }

    #[test]
    fn resolve_period_last_month() {
        let (start, end) = resolve_period("last_month");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!(s.day(), 1);
        assert_eq!(s.month(), e.month());
        // end should be last day of that month
        let next_month_first =
            NaiveDate::from_ymd_opt(e.year(), e.month(), 1).unwrap() + chrono::Duration::days(31);
        let last_day =
            NaiveDate::from_ymd_opt(next_month_first.year(), next_month_first.month(), 1).unwrap()
                - chrono::Duration::days(1);
        // The end should be within that month
        assert!(e.day() >= 28);
        assert_eq!(e, last_day);
    }

    #[test]
    fn resolve_period_3m() {
        let (start, end) = resolve_period("3m");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!((e - s).num_days(), 90);
    }

    #[test]
    fn resolve_period_6m() {
        let (start, end) = resolve_period("6m");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!((e - s).num_days(), 180);
    }

    #[test]
    fn resolve_period_12m() {
        let (start, end) = resolve_period("12m");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!((e - s).num_days(), 365);
    }

    #[test]
    fn resolve_period_default() {
        let (start, end) = resolve_period("unknown");
        let s = NaiveDate::parse_from_str(&start, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end, "%Y-%m-%d").unwrap();
        assert_eq!((e - s).num_days(), 29);
    }

    #[test]
    fn get_period_default() {
        let params = PeriodParams {
            period: None,
            page: None,
        };
        assert_eq!(get_period(&params), "30d");
    }

    #[test]
    fn get_period_specified() {
        let params = PeriodParams {
            period: Some("7d".to_string()),
            page: None,
        };
        assert_eq!(get_period(&params), "7d");
    }

    #[test]
    fn month_to_range_january() {
        let (start, end) = month_to_range("2024-01");
        assert_eq!(start, "2024-01-01");
        assert_eq!(end, "2024-01-31");
    }

    #[test]
    fn month_to_range_february_leap() {
        let (start, end) = month_to_range("2024-02");
        assert_eq!(start, "2024-02-01");
        assert_eq!(end, "2024-02-29");
    }

    #[test]
    fn month_to_range_february_non_leap() {
        let (start, end) = month_to_range("2023-02");
        assert_eq!(start, "2023-02-01");
        assert_eq!(end, "2023-02-28");
    }

    #[test]
    fn month_to_range_december() {
        let (start, end) = month_to_range("2024-12");
        assert_eq!(start, "2024-12-01");
        assert_eq!(end, "2024-12-31");
    }
}
