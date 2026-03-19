use axum::extract::{Path, Query, State};
use axum::response::{Html, IntoResponse, Response};
use chrono::{NaiveDate, Utc};
use tower_sessions::Session;

use super::{
    get_order, get_page, get_period, get_sort, parse_month_range, require_login, resolve_period,
    snap_to_month_start, AppError, AppState, PeriodParams,
};
use crate::pages;

pub async fn render_home(
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

    let daily_cost = state.service.get_daily_cost(start, end).await?;
    let monthly_cost = state
        .service
        .get_monthly_cost(snap_to_month_start(start), end)
        .await?;
    let users = state.service.list_users().await?;
    let models = state.service.list_models().await?;

    let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
    let currency = daily_cost
        .first()
        .map(|r| r.currency.as_str())
        .unwrap_or("USD");

    Ok(Html(pages::home::render(
        &state.base_path,
        &period,
        total_cost,
        currency,
        daily_cost.len(),
        monthly_cost.len(),
        users.len(),
        models.len(),
    ))
    .into_response())
}

pub async fn render_daily_costs(
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

    let daily_cost = state.service.get_daily_cost(start, end).await?;
    let daily_cost = pages::sort_records(daily_cost, sort, &order);

    Ok(Html(pages::costs::render(
        &state.base_path,
        &period,
        page,
        &daily_cost,
    ))
    .into_response())
}

pub async fn render_users(
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

    let users_enriched = state.service.list_users_enriched().await?;
    let costs = state.service.get_cost_by_users(start, end).await?;

    Ok(Html(pages::users::render_index(
        &state.base_path,
        &period,
        page,
        &users_enriched,
        &costs,
        sort,
        &order,
    ))
    .into_response())
}

pub async fn render_models(
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

    let models_enriched = state.service.list_models_enriched().await?;
    let costs = state.service.get_cost_by_models(start, end).await?;

    Ok(Html(pages::models::render_index(
        &state.base_path,
        &period,
        page,
        &models_enriched,
        &costs,
        sort,
        &order,
    ))
    .into_response())
}

pub async fn render_user_hub(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let user_info = state.service.get_user_info(&user_id).await?;
    match user_info {
        Some(info) => {
            Ok(Html(pages::users::render_hub(&state.base_path, &period, &info)).into_response())
        }
        None => {
            let user_email = state
                .service
                .get_user_email(&user_id)
                .await?
                .unwrap_or_else(|| "unknown".to_string());
            let info = common::UserInfo {
                user_id: user_id.clone(),
                user_email,
                created_at: String::new(),
                api_key_count: 0,
                active_api_key_count: 0,
                inference_profile_count: 0,
            };
            Ok(Html(pages::users::render_hub(&state.base_path, &period, &info)).into_response())
        }
    }
}

pub async fn render_user_daily_costs(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
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
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_daily_cost_for_user_id(start, end, &user_id)
        .await?;
    let costs = pages::sort_records(costs, sort, &order);

    Ok(Html(pages::users::render_daily_costs(
        &state.base_path,
        &period,
        page,
        &user_id,
        &user_email,
        &costs,
    ))
    .into_response())
}

pub async fn render_user_monthly_costs(
    session: Session,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
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
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_monthly_cost_for_user_id(snap_to_month_start(start), end, &user_id)
        .await?;
    let costs = pages::sort_records(costs, sort, &order);

    Ok(Html(pages::users::render_monthly_costs(
        &state.base_path,
        &period,
        page,
        &user_id,
        &user_email,
        &costs,
    ))
    .into_response())
}

pub async fn render_model_hub(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let model_info = state.service.get_model_info(&model_id).await?;
    match model_info {
        Some(info) => {
            Ok(Html(pages::models::render_hub(&state.base_path, &period, &info)).into_response())
        }
        None => {
            let model_name = state
                .service
                .get_model_name(&model_id)
                .await?
                .unwrap_or_else(|| "unknown".to_string());
            let info = common::ModelInfo {
                model_id: model_id.clone(),
                model_name,
                is_disabled: false,
                protected: false,
                user_count: 1,
            };
            Ok(
                Html(pages::models::render_hub(&state.base_path, &period, &info))
                    .into_response(),
            )
        }
    }
}

pub async fn render_model_daily_costs(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
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
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_daily_cost_for_model_id(start, end, &model_id)
        .await?;
    let costs = pages::sort_records(costs, sort, &order);

    Ok(Html(pages::models::render_daily_costs(
        &state.base_path,
        &period,
        page,
        &model_id,
        &model_name,
        &costs,
    ))
    .into_response())
}

pub async fn render_model_monthly_costs(
    session: Session,
    State(state): State<AppState>,
    Path(model_id): Path<String>,
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
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_monthly_cost_for_model_id(snap_to_month_start(start), end, &model_id)
        .await?;
    let costs = pages::sort_records(costs, sort, &order);

    Ok(Html(pages::models::render_monthly_costs(
        &state.base_path,
        &period,
        page,
        &model_id,
        &model_name,
        &costs,
    ))
    .into_response())
}

// --- Daily cost drill-down handlers ---

pub async fn render_date_hub(
    session: Session,
    State(state): State<AppState>,
    Path(date): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);

    let daily_cost = state.service.get_daily_cost(date_nd, next_day).await?;
    let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
    let currency = daily_cost
        .first()
        .map(|r| r.currency.as_str())
        .unwrap_or("USD");
    let users = state.service.get_cost_by_users(date_nd, next_day).await?;
    let models = state
        .service
        .get_cost_by_models(date_nd, next_day)
        .await?;

    Ok(Html(pages::costs::render_hub(
        &state.base_path,
        &period,
        &date,
        total_cost,
        currency,
        users.len(),
        models.len(),
    ))
    .into_response())
}

pub async fn render_date_users(
    session: Session,
    State(state): State<AppState>,
    Path(date): Path<String>,
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
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);

    let costs = state.service.get_cost_by_users(date_nd, next_day).await?;
    let costs = pages::sort_by_user(costs, sort, &order);

    Ok(Html(pages::costs::render_users(
        &state.base_path,
        &period,
        page,
        &date,
        &costs,
    ))
    .into_response())
}

pub async fn render_date_models(
    session: Session,
    State(state): State<AppState>,
    Path(date): Path<String>,
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
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);

    let costs = state
        .service
        .get_cost_by_models(date_nd, next_day)
        .await?;
    let costs = pages::sort_by_model(costs, sort, &order);

    Ok(Html(pages::costs::render_models(
        &state.base_path,
        &period,
        page,
        &date,
        &costs,
    ))
    .into_response())
}

pub async fn render_date_models_for_user(
    session: Session,
    State(state): State<AppState>,
    Path((date, user_id)): Path<(String, String)>,
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
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_models_for_user_id(date_nd, next_day, &user_id)
        .await?;
    let costs = pages::sort_by_model(costs, sort, &order);

    Ok(Html(pages::costs::render_user_models(
        &state.base_path,
        &period,
        page,
        &date,
        &user_email,
        &costs,
    ))
    .into_response())
}

pub async fn render_date_users_for_model(
    session: Session,
    State(state): State<AppState>,
    Path((date, model_id)): Path<(String, String)>,
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
    let date_nd = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());
    let next_day = date_nd + chrono::Duration::days(1);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_users_for_model_id(date_nd, next_day, &model_id)
        .await?;
    let costs = pages::sort_by_user(costs, sort, &order);

    Ok(Html(pages::costs::render_model_users(
        &state.base_path,
        &period,
        page,
        &date,
        &model_name,
        &costs,
    ))
    .into_response())
}

// --- Monthly cost handlers ---

pub async fn render_monthly_costs(
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

    let monthly_cost = state
        .service
        .get_monthly_cost(snap_to_month_start(start), end)
        .await?;
    let monthly_cost = pages::sort_records(monthly_cost, sort, &order);

    Ok(Html(pages::monthly::render(
        &state.base_path,
        &period,
        page,
        &monthly_cost,
    ))
    .into_response())
}

pub async fn render_month_hub(
    session: Session,
    State(state): State<AppState>,
    Path(month): Path<String>,
    Query(params): Query<PeriodParams>,
) -> Result<Response, AppError> {
    let _email = match require_login(&session).await {
        Ok(email) => email,
        Err(redirect) => return Ok(redirect),
    };

    let period = get_period(&params);
    let (start, end) = parse_month_range(&month);

    let daily_cost = state.service.get_daily_cost(start, end).await?;
    let total_cost: f64 = daily_cost.iter().map(|r| r.amount).sum();
    let currency = daily_cost
        .first()
        .map(|r| r.currency.as_str())
        .unwrap_or("USD");
    let users = state.service.get_cost_by_users(start, end).await?;
    let models = state.service.get_cost_by_models(start, end).await?;

    Ok(Html(pages::monthly::render_hub(
        &state.base_path,
        &period,
        &month,
        total_cost,
        currency,
        users.len(),
        models.len(),
    ))
    .into_response())
}

pub async fn render_month_users(
    session: Session,
    State(state): State<AppState>,
    Path(month): Path<String>,
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
    let (start, end) = parse_month_range(&month);

    let costs = state.service.get_cost_by_users(start, end).await?;
    let costs = pages::sort_by_user(costs, sort, &order);

    Ok(Html(pages::monthly::render_users(
        &state.base_path,
        &period,
        page,
        &month,
        &costs,
    ))
    .into_response())
}

pub async fn render_month_models(
    session: Session,
    State(state): State<AppState>,
    Path(month): Path<String>,
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
    let (start, end) = parse_month_range(&month);

    let costs = state.service.get_cost_by_models(start, end).await?;
    let costs = pages::sort_by_model(costs, sort, &order);

    Ok(Html(pages::monthly::render_models(
        &state.base_path,
        &period,
        page,
        &month,
        &costs,
    ))
    .into_response())
}

pub async fn render_month_models_for_user(
    session: Session,
    State(state): State<AppState>,
    Path((month, user_id)): Path<(String, String)>,
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
    let (start, end) = parse_month_range(&month);
    let user_email = state
        .service
        .get_user_email(&user_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_models_for_user_id(start, end, &user_id)
        .await?;
    let costs = pages::sort_by_model(costs, sort, &order);

    Ok(Html(pages::monthly::render_user_models(
        &state.base_path,
        &period,
        page,
        &month,
        &user_email,
        &costs,
    ))
    .into_response())
}

pub async fn render_month_users_for_model(
    session: Session,
    State(state): State<AppState>,
    Path((month, model_id)): Path<(String, String)>,
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
    let (start, end) = parse_month_range(&month);
    let model_name = state
        .service
        .get_model_name(&model_id)
        .await?
        .unwrap_or_else(|| "unknown".to_string());
    let costs = state
        .service
        .get_cost_by_users_for_model_id(start, end, &model_id)
        .await?;
    let costs = pages::sort_by_user(costs, sort, &order);

    Ok(Html(pages::monthly::render_model_users(
        &state.base_path,
        &period,
        page,
        &month,
        &model_name,
        &costs,
    ))
    .into_response())
}
