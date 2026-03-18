use async_trait::async_trait;
use chrono::NaiveDate;
use common::{CostByModel, CostByUser, CostRecord, ModelInfo, UserInfo};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait CostService: Send + Sync {
    async fn health_check(&self) -> Result<(), String>;
    async fn get_daily_cost(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostRecord>;
    async fn get_monthly_cost(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostRecord>;
    async fn get_cost_by_users(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostByUser>;
    async fn get_cost_by_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostByUser>;
    async fn get_cost_by_models(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostByModel>;
    async fn get_cost_by_models_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostByModel>;
    async fn get_cost_by_users_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> Vec<CostByUser>;
    async fn get_cost_by_user_id_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> Vec<CostByUser>;
    async fn get_daily_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_monthly_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_daily_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_monthly_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_daily_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_monthly_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_user_email(&self, user_id: &str) -> Option<String>;
    async fn get_model_name(&self, model_id: &str) -> Option<String>;
    async fn list_users(&self) -> Vec<(String, String)>;
    async fn list_models(&self) -> Vec<(String, String)>;
    async fn get_user_id_by_email(&self, email: &str) -> Option<String>;
    async fn list_users_enriched(&self) -> Vec<UserInfo>;
    async fn get_user_info(&self, user_id: &str) -> Option<UserInfo>;
    async fn list_models_enriched(&self) -> Vec<ModelInfo>;
    async fn list_models_enriched_by_user_id(&self, user_id: &str) -> Vec<ModelInfo>;
    async fn get_model_info(&self, model_id: &str) -> Option<ModelInfo>;
}

pub struct RealCostService {
    pub pool: PgPool,
    pub cost_pool: PgPool,
}

#[async_trait]
impl CostService for RealCostService {
    async fn health_check(&self) -> Result<(), String> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("gateway db: {e}"))?;
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.cost_pool)
            .await
            .map_err(|e| format!("cost db: {e}"))?;
        Ok(())
    }

    async fn get_daily_cost(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostRecord> {
        db::get_daily_cost(&self.cost_pool, start, end)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query daily cost: {e}");
                Vec::new()
            })
    }

    async fn get_monthly_cost(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostRecord> {
        db::get_monthly_cost(&self.cost_pool, start, end)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query monthly cost: {e}");
                Vec::new()
            })
    }

    async fn get_cost_by_users(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostByUser> {
        let mut costs = db::get_cost_by_users(&self.cost_pool, start, end)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query cost by user: {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_cost_by_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostByUser> {
        let mut costs = db::get_cost_by_user_id(&self.cost_pool, start, end, user_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query cost by user id: {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_cost_by_models(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostByModel> {
        let mut costs = db::get_cost_by_models(&self.cost_pool, start, end)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query cost by model: {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.model_name = self.get_model_name(&cost.model_id).await;
        }
        costs
    }

    async fn get_cost_by_models_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostByModel> {
        let mut costs = db::get_cost_by_models_for_user_id(&self.cost_pool, start, end, user_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query cost by model for user: {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.model_name = self.get_model_name(&cost.model_id).await;
        }
        costs
    }

    async fn get_cost_by_users_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> Vec<CostByUser> {
        let mut costs = db::get_cost_by_users_for_model_id(&self.cost_pool, start, end, model_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query cost by user for model: {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_cost_by_user_id_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> Vec<CostByUser> {
        let mut costs =
            db::get_cost_by_user_id_for_model_id(&self.cost_pool, start, end, user_id, model_id)
                .await
                .unwrap_or_else(|e| {
                    log::error!("Failed to query cost by user id for model: {e}");
                    Vec::new()
                });
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_daily_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostRecord> {
        db::get_daily_cost_for_user_id(&self.cost_pool, start, end, user_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query daily cost for user: {e}");
                Vec::new()
            })
    }

    async fn get_monthly_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> Vec<CostRecord> {
        db::get_monthly_cost_for_user_id(&self.cost_pool, start, end, user_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query monthly cost for user: {e}");
                Vec::new()
            })
    }

    async fn get_daily_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> Vec<CostRecord> {
        db::get_daily_cost_for_model_id(&self.cost_pool, start, end, model_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query daily cost for model: {e}");
                Vec::new()
            })
    }

    async fn get_monthly_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> Vec<CostRecord> {
        db::get_monthly_cost_for_model_id(&self.cost_pool, start, end, model_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query monthly cost for model: {e}");
                Vec::new()
            })
    }

    async fn get_daily_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> Vec<CostRecord> {
        db::get_daily_cost_for_user_id_and_model_id(&self.cost_pool, start, end, user_id, model_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query daily cost for user and model: {e}");
                Vec::new()
            })
    }

    async fn get_monthly_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> Vec<CostRecord> {
        db::get_monthly_cost_for_user_id_and_model_id(&self.cost_pool, start, end, user_id, model_id)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to query monthly cost for user and model: {e}");
                Vec::new()
            })
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

    async fn list_users_enriched(&self) -> Vec<UserInfo> {
        db::list_users_enriched(&self.pool)
            .await
            .unwrap_or_default()
    }

    async fn get_user_info(&self, user_id: &str) -> Option<UserInfo> {
        let uuid = Uuid::parse_str(user_id).ok()?;
        db::get_user_info(&self.pool, uuid).await
    }

    async fn list_models_enriched(&self) -> Vec<ModelInfo> {
        db::list_models_enriched(&self.pool)
            .await
            .unwrap_or_default()
    }

    async fn list_models_enriched_by_user_id(&self, user_id: &str) -> Vec<ModelInfo> {
        let Some(uuid) = uuid::Uuid::parse_str(user_id).ok() else {
            return Vec::new();
        };
        db::list_models_enriched_by_user_id(&self.pool, uuid)
            .await
            .unwrap_or_default()
    }

    async fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let uuid = Uuid::parse_str(model_id).ok()?;
        db::get_model_info(&self.pool, uuid).await
    }
}
