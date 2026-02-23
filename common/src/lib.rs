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
