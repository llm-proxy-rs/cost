use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CostByUser {
    pub user_id: String,
    pub user_email: Option<String>,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostByModel {
    pub model_id: String,
    pub model_name: Option<String>,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostRecord {
    pub date: String,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub user_id: String,
    pub user_email: String,
    pub created_at: String,
    pub api_key_count: i64,
    pub active_api_key_count: i64,
    pub inference_profile_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub model_name: String,
    pub is_disabled: bool,
    pub protected: bool,
    pub user_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyInfo {
    pub api_key_id: String,
    pub api_key_preview: String,
    pub is_disabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InferenceProfileInfo {
    pub inference_profile_id: String,
    pub model_id: String,
    pub model_name: Option<String>,
    pub user_id: String,
    pub user_email: Option<String>,
    pub created_at: String,
}
