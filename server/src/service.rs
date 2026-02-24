use async_trait::async_trait;
use aws_sdk_costexplorer::Client as CeClient;
use common::{CostByModel, CostByUser, CostRecord};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait CostService: Send + Sync {
    async fn get_daily_cost(&self, start: &str, end: &str) -> Vec<CostRecord>;
    async fn get_cost_by_user(&self, start: &str, end: &str) -> Vec<CostByUser>;
    async fn get_cost_by_model(&self, start: &str, end: &str) -> Vec<CostByModel>;
    async fn get_cost_by_model_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostByModel>;
    async fn get_cost_by_user_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostByUser>;
    async fn get_user_email(&self, user_id: &str) -> Option<String>;
    async fn get_model_name(&self, model_id: &str) -> Option<String>;
    async fn list_users(&self) -> Vec<(String, String)>;
    async fn list_models(&self) -> Vec<(String, String)>;
    async fn get_user_id_by_email(&self, email: &str) -> Option<String>;
}

pub struct RealCostService {
    pub pool: PgPool,
    pub ce_client: CeClient,
}

#[async_trait]
impl CostService for RealCostService {
    async fn get_daily_cost(&self, start: &str, end: &str) -> Vec<CostRecord> {
        ce::get_daily_cost(&self.ce_client, start, end)
            .await
            .unwrap_or_default()
    }

    async fn get_cost_by_user(&self, start: &str, end: &str) -> Vec<CostByUser> {
        let mut costs = ce::get_cost_by_user(&self.ce_client, start, end)
            .await
            .unwrap_or_default();
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_cost_by_model(&self, start: &str, end: &str) -> Vec<CostByModel> {
        let mut costs = ce::get_cost_by_model(&self.ce_client, start, end)
            .await
            .unwrap_or_default();
        for cost in &mut costs {
            cost.model_name = self.get_model_name(&cost.model_id).await;
        }
        costs
    }

    async fn get_cost_by_model_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostByModel> {
        let mut costs = ce::get_cost_by_model_for_user(&self.ce_client, start, end, user_id)
            .await
            .unwrap_or_default();
        for cost in &mut costs {
            cost.model_name = self.get_model_name(&cost.model_id).await;
        }
        costs
    }

    async fn get_cost_by_user_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostByUser> {
        let mut costs = ce::get_cost_by_user_for_model(&self.ce_client, start, end, model_id)
            .await
            .unwrap_or_default();
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_user_email(&self, user_id: &str) -> Option<String> {
        let uuid = Uuid::parse_str(user_id).ok()?;
        db::get_user_email(&self.pool, uuid).await
    }

    async fn get_model_name(&self, model_id: &str) -> Option<String> {
        let uuid = Uuid::parse_str(model_id).ok()?;
        db::get_model_name(&self.pool, uuid).await
    }

    async fn list_users(&self) -> Vec<(String, String)> {
        db::list_users(&self.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(id, email)| (id.to_string(), email))
            .collect()
    }

    async fn list_models(&self) -> Vec<(String, String)> {
        db::list_models(&self.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|(id, name)| (id.to_string(), name))
            .collect()
    }

    async fn get_user_id_by_email(&self, email: &str) -> Option<String> {
        db::get_user_id_by_email(&self.pool, email)
            .await
            .map(|uuid| uuid.to_string())
    }
}
