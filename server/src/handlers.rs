use std::sync::Arc;

use axum::extract::{Path, Query, State};
#[cfg(not(feature = "admin"))]
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use tower_sessions::Session;

use crate::pages;
use crate::service::CostService;

pub async fn health_check(State(state): State<AppState>) -> Response {
    match state.service.health_check().await {
        Ok(()) => (axum::http::StatusCode::OK, "ok").into_response(),
        Err(e) => {
            log::error!("Health check failed: {e}");
            (axum::http::StatusCode::SERVICE_UNAVAILABLE, format!("error: {e}")).into_response()
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

#[cfg(not(feature = "admin"))]
async fn resolve_current_user_id(service: &dyn CostService, email: &str) -> Option<String> {
    service.get_user_id_by_email(email).await
}

#[cfg(not(feature = "admin"))]
fn forbidden() -> Response {
    (StatusCode::FORBIDDEN, "Access denied. Your account does not have a user profile.").into_response()
}

pub async fn render_home(
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
        let daily_cost = state.service.get_daily_cost(start, end).await;
        let monthly_cost = state.service.get_monthly_cost(snap_to_month_start(start), end).await;
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
            state.service.get_daily_cost_for_user(start, end, uid).await
        } else {
            vec![]
        };
        let monthly_cost = if let Some(ref uid) = current_user_id {
            state.service.get_monthly_cost_for_user(snap_to_month_start(start), end, uid).await
        } else {
            vec![]
        };
        let model_count = if let Some(ref uid) = current_user_id {
            let costs = state
                .service
                .get_cost_by_models_for_user_id(start, end, uid)
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

pub async fn render_daily_costs(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(start, end).await;
        let daily_cost = pages::sort_records(daily_cost, sort, &order);

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
        let daily_cost = if let Some(ref uid) = current_user_id {
            state.service.get_daily_cost_for_user(start, end, uid).await
        } else {
            vec![]
        };
        let daily_cost = pages::sort_records(daily_cost, sort, &order);

        Html(pages::costs::render(
            &state.base_path,
            &period,
            page,
            &daily_cost,
        ))
        .into_response()
    }
}

pub async fn render_users(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let users_enriched = state.service.list_users_enriched().await;
        let costs = state.service.get_cost_by_users(start, end).await;

        Html(pages::users::render_index(
            &state.base_path,
            &period,
            page,
            &users_enriched,
            &costs,
            sort,
            &order,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = match resolve_current_user_id(state.service.as_ref(), &_email).await {
            Some(uid) => uid,
            None => return forbidden(),
        };
        let costs = state.service.get_cost_by_user_id(start, end, &current_user_id).await;
        let users_enriched = state
            .service
            .get_user_info(&current_user_id)
            .await
            .into_iter()
            .collect::<Vec<_>>();

        Html(pages::users::render_index(
            &state.base_path,
            &period,
            page,
            &users_enriched,
            &costs,
            sort,
            &order,
        ))
        .into_response()
    }
}

pub async fn render_models(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let models_enriched = state.service.list_models_enriched().await;
        let costs = state.service.get_cost_by_models(start, end).await;

        Html(pages::models::render_index(
            &state.base_path,
            &period,
            page,
            &models_enriched,
            &costs,
            sort,
            &order,
        ))
        .into_response()
    }

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let costs = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_models_for_user_id(start, end, uid)
                .await
        } else {
            vec![]
        };
        let models_enriched = if let Some(ref uid) = current_user_id {
            state.service.list_models_enriched_by_user_id(uid).await
        } else {
            vec![]
        };

        Html(pages::models::render_index(
            &state.base_path,
            &period,
            page,
            &models_enriched,
            &costs,
            sort,
            &order,
        ))
        .into_response()
    }
}

pub async fn render_user_hub(
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
            return forbidden();
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

pub async fn render_user_daily_costs(
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
            return forbidden();
        }
    }

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_daily_cost_for_user(start, end, &user_id)
        .await;
    let costs = pages::sort_records(costs, sort, &order);

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

pub async fn render_user_monthly_costs(
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
            return forbidden();
        }
    }

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_monthly_cost_for_user(snap_to_month_start(start), end, &user_id)
        .await;
    let costs = pages::sort_records(costs, sort, &order);

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

pub async fn render_model_hub(
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

    #[cfg(not(feature = "admin"))]
    {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let has_access = if let Some(ref uid) = current_user_id {
            let (start, end) = resolve_period("12m");
            let costs = state
                .service
                .get_cost_by_models_for_user_id(start, end, uid)
                .await;
            costs.iter().any(|c| c.model_id == model_id)
        } else {
            false
        };
        if !has_access {
            return forbidden();
        }
    }

    let model_info = state.service.get_model_info(&model_id).await;
    match model_info {
        Some(mut info) => {
            #[cfg(not(feature = "admin"))]
            {
                info.user_count = 1;
            }
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
                user_count: 1,
            };
            Html(pages::models::render_hub(&state.base_path, &period, &info)).into_response()
        }
    }
}

pub async fn render_model_daily_costs(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());

    #[cfg(feature = "admin")]
    let costs = state
        .service
        .get_daily_cost_for_model(start, end, &model_id)
        .await;

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        if let Some(ref uid) = current_user_id {
            state
                .service
                .get_daily_cost_for_user_and_model(start, end, uid, &model_id)
                .await
        } else {
            vec![]
        }
    };

    let costs = pages::sort_records(costs, sort, &order);

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

pub async fn render_model_monthly_costs(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());

    #[cfg(feature = "admin")]
    let costs = state
        .service
        .get_monthly_cost_for_model(snap_to_month_start(start), end, &model_id)
        .await;

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        if let Some(ref uid) = current_user_id {
            state
                .service
                .get_monthly_cost_for_user_and_model(snap_to_month_start(start), end, uid, &model_id)
                .await
        } else {
            vec![]
        }
    };

    let costs = pages::sort_records(costs, sort, &order);

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

pub async fn render_date_hub(
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
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(date_nd, next_day).await;
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = state.service.get_cost_by_users(date_nd, next_day).await;
        let models = state.service.get_cost_by_models(date_nd, next_day).await;

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
        let daily_cost = if let Some(ref uid) = current_user_id {
            state.service.get_daily_cost_for_user(date_nd, next_day, uid).await
        } else {
            vec![]
        };
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = if let Some(ref uid) = current_user_id {
            state.service.get_cost_by_user_id(date_nd, next_day, uid).await
        } else {
            vec![]
        };
        let models = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_models_for_user_id(date_nd, next_day, uid)
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

pub async fn render_date_users(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_users(date_nd, next_day).await;
        let costs = pages::sort_by_user(costs, sort, &order);

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
        let current_user_id = match resolve_current_user_id(state.service.as_ref(), &_email).await {
            Some(uid) => uid,
            None => return forbidden(),
        };
        let costs = state.service.get_cost_by_user_id(date_nd, next_day, &current_user_id).await;
        let costs = pages::sort_by_user(costs, sort, &order);

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

pub async fn render_date_models(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_models(date_nd, next_day).await;
        let costs = pages::sort_by_model(costs, sort, &order);

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
                .get_cost_by_models_for_user_id(date_nd, next_day, uid)
                .await
        } else {
            vec![]
        };
        let costs = pages::sort_by_model(costs, sort, &order);

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

pub async fn render_date_models_for_user(
    session: Session,
    State(state): State<AppState>,
    Path((date, user_id)): Path<(String, String)>,
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
            return forbidden();
        }
    }

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_models_for_user_id(date_nd, next_day, &user_id)
        .await;
    let costs = pages::sort_by_model(costs, sort, &order);

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

pub async fn render_date_users_for_model(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());

    #[cfg(feature = "admin")]
    let costs = state
        .service
        .get_cost_by_users_for_model_id(date_nd, next_day, &model_id)
        .await;

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let all = state
            .service
            .get_cost_by_users_for_model_id(date_nd, next_day, &model_id)
            .await;
        if let Some(ref uid) = current_user_id {
            all.into_iter().filter(|c| c.user_id == *uid).collect()
        } else {
            vec![]
        }
    };

    let costs = pages::sort_by_user(costs, sort, &order);

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

pub async fn render_monthly_costs(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = resolve_period(&period);

    #[cfg(feature = "admin")]
    {
        let monthly_cost = state.service.get_monthly_cost(snap_to_month_start(start), end).await;
        let monthly_cost = pages::sort_records(monthly_cost, sort, &order);

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
        let monthly_cost = if let Some(ref uid) = current_user_id {
            state.service.get_monthly_cost_for_user(snap_to_month_start(start), end, uid).await
        } else {
            vec![]
        };
        let monthly_cost = pages::sort_records(monthly_cost, sort, &order);

        Html(pages::monthly::render(
            &state.base_path,
            &period,
            page,
            &monthly_cost,
        ))
        .into_response()
    }
}

pub async fn render_month_hub(
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
    let (start, end) = parse_month_range(&month);

    #[cfg(feature = "admin")]
    {
        let daily_cost = state.service.get_daily_cost(start, end).await;
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = state.service.get_cost_by_users(start, end).await;
        let models = state.service.get_cost_by_models(start, end).await;

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
        let daily_cost = if let Some(ref uid) = current_user_id {
            state.service.get_daily_cost_for_user(start, end, uid).await
        } else {
            vec![]
        };
        let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
        let currency = daily_cost
            .first()
            .map(|r| r.currency.as_str())
            .unwrap_or("USD");
        let users = if let Some(ref uid) = current_user_id {
            state.service.get_cost_by_user_id(start, end, uid).await
        } else {
            vec![]
        };
        let models = if let Some(ref uid) = current_user_id {
            state
                .service
                .get_cost_by_models_for_user_id(start, end, uid)
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

pub async fn render_month_users(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = parse_month_range(&month);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_users(start, end).await;
        let costs = pages::sort_by_user(costs, sort, &order);

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
        let current_user_id = match resolve_current_user_id(state.service.as_ref(), &_email).await {
            Some(uid) => uid,
            None => return forbidden(),
        };
        let costs = state.service.get_cost_by_user_id(start, end, &current_user_id).await;
        let costs = pages::sort_by_user(costs, sort, &order);

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

pub async fn render_month_models(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = parse_month_range(&month);

    #[cfg(feature = "admin")]
    {
        let costs = state.service.get_cost_by_models(start, end).await;
        let costs = pages::sort_by_model(costs, sort, &order);

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
                .get_cost_by_models_for_user_id(start, end, uid)
                .await
        } else {
            vec![]
        };
        let costs = pages::sort_by_model(costs, sort, &order);

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

pub async fn render_month_models_for_user(
    session: Session,
    State(state): State<AppState>,
    Path((month, user_id)): Path<(String, String)>,
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
            return forbidden();
        }
    }

    let period = get_period(&params);
    let page = get_page(&params);
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = parse_month_range(&month);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_models_for_user_id(start, end, &user_id)
        .await;
    let costs = pages::sort_by_model(costs, sort, &order);

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

pub async fn render_month_users_for_model(
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
    let sort = get_sort(&params);
    let order = get_order(&params);
    let (start, end) = parse_month_range(&month);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await
        .unwrap_or_else(|| "unknown".to_string());

    #[cfg(feature = "admin")]
    let costs = state
        .service
        .get_cost_by_users_for_model_id(start, end, &model_id)
        .await;

    #[cfg(not(feature = "admin"))]
    let costs = {
        let current_user_id = resolve_current_user_id(state.service.as_ref(), &_email).await;
        let all = state
            .service
            .get_cost_by_users_for_model_id(start, end, &model_id)
            .await;
        if let Some(ref uid) = current_user_id {
            all.into_iter().filter(|c| c.user_id == *uid).collect()
        } else {
            vec![]
        }
    };

    let costs = pages::sort_by_user(costs, sort, &order);

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
