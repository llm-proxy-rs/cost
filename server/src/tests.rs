use async_trait::async_trait;
use axum::body::Body;
use common::{CostByModel, CostByUser, CostRecord, ModelInfo, UserInfo};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

use crate::build_router;
use crate::handlers::AppState;
use crate::service::CostService;

struct MockCostService {
    users: Vec<CostByUser>,
    models: Vec<CostByModel>,
    daily: Vec<CostRecord>,
}

impl MockCostService {
    fn new() -> Self {
        Self {
            users: vec![CostByUser {
                user_id: "aaaa-bbbb".to_string(),
                user_email: Some("alice@example.com".to_string()),
                amount: 100.0,
                currency: "USD".to_string(),
            }],
            models: vec![CostByModel {
                model_id: "cccc-dddd".to_string(),
                model_name: Some("claude-3-sonnet".to_string()),
                amount: 80.0,
                currency: "USD".to_string(),
            }],
            daily: vec![CostRecord {
                date: "2024-01-15".to_string(),
                amount: 100.0,
                currency: "USD".to_string(),
            }],
        }
    }
}

#[async_trait]
impl CostService for MockCostService {
    async fn get_daily_cost(&self, _start: &str, _end: &str) -> Vec<CostRecord> {
        self.daily.clone()
    }

    async fn get_monthly_cost(&self, _start: &str, _end: &str) -> Vec<CostRecord> {
        vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 500.0,
            currency: "USD".to_string(),
        }]
    }

    async fn get_cost_by_user(&self, _start: &str, _end: &str) -> Vec<CostByUser> {
        self.users.clone()
    }

    async fn get_cost_by_model(&self, _start: &str, _end: &str) -> Vec<CostByModel> {
        self.models.clone()
    }

    async fn get_cost_by_model_for_user(
        &self,
        _start: &str,
        _end: &str,
        _user_id: &str,
    ) -> Vec<CostByModel> {
        self.models.clone()
    }

    async fn get_cost_by_user_for_model(
        &self,
        _start: &str,
        _end: &str,
        _model_id: &str,
    ) -> Vec<CostByUser> {
        self.users.clone()
    }

    async fn get_daily_cost_for_user(
        &self,
        _start: &str,
        _end: &str,
        _user_id: &str,
    ) -> Vec<CostRecord> {
        self.daily.clone()
    }

    async fn get_monthly_cost_for_user(
        &self,
        _start: &str,
        _end: &str,
        _user_id: &str,
    ) -> Vec<CostRecord> {
        self.daily.clone()
    }

    async fn get_daily_cost_for_model(
        &self,
        _start: &str,
        _end: &str,
        _model_id: &str,
    ) -> Vec<CostRecord> {
        self.daily.clone()
    }

    async fn get_monthly_cost_for_model(
        &self,
        _start: &str,
        _end: &str,
        _model_id: &str,
    ) -> Vec<CostRecord> {
        self.daily.clone()
    }

    async fn get_user_email(&self, _user_id: &str) -> Option<String> {
        Some("alice@example.com".to_string())
    }

    async fn get_model_name(&self, _model_id: &str) -> Option<String> {
        Some("claude-3-sonnet".to_string())
    }

    async fn list_users(&self) -> Vec<(String, String)> {
        vec![("aaaa-bbbb".to_string(), "alice@example.com".to_string())]
    }

    async fn list_models(&self) -> Vec<(String, String)> {
        vec![("cccc-dddd".to_string(), "claude-3-sonnet".to_string())]
    }

    async fn get_user_id_by_email(&self, _email: &str) -> Option<String> {
        Some("aaaa-bbbb".to_string())
    }

    async fn list_users_enriched(&self) -> Vec<UserInfo> {
        vec![UserInfo {
            user_id: "aaaa-bbbb".to_string(),
            user_email: "alice@example.com".to_string(),
            created_at: "2024-01-01".to_string(),
            api_key_count: 2,
            active_api_key_count: 1,
            inference_profile_count: 3,
        }]
    }

    async fn get_user_info(&self, _user_id: &str) -> Option<UserInfo> {
        Some(UserInfo {
            user_id: "aaaa-bbbb".to_string(),
            user_email: "alice@example.com".to_string(),
            created_at: "2024-01-01".to_string(),
            api_key_count: 2,
            active_api_key_count: 1,
            inference_profile_count: 3,
        })
    }

    async fn list_models_enriched(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            model_id: "cccc-dddd".to_string(),
            model_name: "claude-3-sonnet".to_string(),
            is_disabled: false,
            protected: false,
            user_count: 1,
        }]
    }

    async fn get_model_info(&self, _model_id: &str) -> Option<ModelInfo> {
        Some(ModelInfo {
            model_id: "cccc-dddd".to_string(),
            model_name: "claude-3-sonnet".to_string(),
            is_disabled: false,
            protected: false,
            user_count: 1,
        })
    }
}

fn mock_state(base: &str) -> AppState {
    AppState {
        service: Arc::new(MockCostService::new()),
        base_path: base.to_string(),
        cognito_client_id: String::new(),
        cognito_client_secret: String::new(),
        cognito_domain: String::new(),
        cognito_redirect_uri: String::new(),
        cognito_region: String::new(),
        cognito_user_pool_id: String::new(),
    }
}

fn test_app() -> axum::Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(3600)));
    build_router(mock_state("/")).layer(session_layer)
}

fn test_app_with_base(base: &str) -> axum::Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(3600)));
    build_router(mock_state(base)).layer(session_layer)
}

async fn get_from(app: axum::Router, uri: &str) -> (u16, String) {
    let req = axum::http::Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    (status, text)
}

async fn get(uri: &str) -> (u16, String) {
    get_from(test_app(), uri).await
}

#[tokio::test]
async fn unauthenticated_home_redirects_to_login() {
    let (status, _) = get("/").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_daily_costs_redirects_to_login() {
    let (status, _) = get("/costs/daily").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_users_redirects_to_login() {
    let (status, _) = get("/users").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_models_redirects_to_login() {
    let (status, _) = get("/models").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_user_detail_redirects_to_login() {
    let (status, _) = get("/users/aaaa-bbbb").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_model_detail_redirects_to_login() {
    let (status, _) = get("/models/cccc-dddd").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_user_daily_costs_redirects_to_login() {
    let (status, _) = get("/users/aaaa-bbbb/daily").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_user_monthly_costs_redirects_to_login() {
    let (status, _) = get("/users/aaaa-bbbb/monthly").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_model_daily_costs_redirects_to_login() {
    let (status, _) = get("/models/cccc-dddd/daily").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_model_monthly_costs_redirects_to_login() {
    let (status, _) = get("/models/cccc-dddd/monthly").await;
    assert!(status == 303 || status == 302 || status == 307);
}

// Daily cost drill-down redirects
#[tokio::test]
async fn unauthenticated_cost_date_detail_redirects_to_login() {
    let (status, _) = get("/costs/daily/2024-01-15").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_date_users_redirects_to_login() {
    let (status, _) = get("/costs/daily/2024-01-15/users").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_date_models_redirects_to_login() {
    let (status, _) = get("/costs/daily/2024-01-15/models").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_date_user_models_redirects_to_login() {
    let (status, _) = get("/costs/daily/2024-01-15/users/aaaa-bbbb").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_date_model_users_redirects_to_login() {
    let (status, _) = get("/costs/daily/2024-01-15/models/cccc-dddd").await;
    assert!(status == 303 || status == 302 || status == 307);
}

// Monthly cost redirects
#[tokio::test]
async fn unauthenticated_monthly_costs_redirects_to_login() {
    let (status, _) = get("/costs/monthly").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_month_detail_redirects_to_login() {
    let (status, _) = get("/costs/monthly/2024-01").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_month_users_redirects_to_login() {
    let (status, _) = get("/costs/monthly/2024-01/users").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_month_models_redirects_to_login() {
    let (status, _) = get("/costs/monthly/2024-01/models").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_month_user_models_redirects_to_login() {
    let (status, _) = get("/costs/monthly/2024-01/users/aaaa-bbbb").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn unauthenticated_cost_month_model_users_redirects_to_login() {
    let (status, _) = get("/costs/monthly/2024-01/models/cccc-dddd").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn nonexistent_route_returns_404() {
    let (status, _) = get("/nonexistent").await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn nested_base_path_redirects_to_login() {
    let app = test_app_with_base("/_dashboard");
    let (status, _) = get_from(app, "/_dashboard").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn nested_base_path_users_redirects() {
    let app = test_app_with_base("/_dashboard");
    let (status, _) = get_from(app, "/_dashboard/users").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn nested_base_path_new_routes_redirect() {
    let app = test_app_with_base("/_dashboard");
    let (status, _) = get_from(app, "/_dashboard/users/aaaa-bbbb/daily").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn nested_base_path_daily_costs_redirect() {
    let app = test_app_with_base("/_dashboard");
    let (status, _) = get_from(app, "/_dashboard/costs/daily").await;
    assert!(status == 303 || status == 302 || status == 307);
}

#[tokio::test]
async fn nested_base_path_monthly_costs_redirect() {
    let app = test_app_with_base("/_dashboard");
    let (status, _) = get_from(app, "/_dashboard/costs/monthly").await;
    assert!(status == 303 || status == 302 || status == 307);
}
