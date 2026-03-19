use async_trait::async_trait;
use chrono::NaiveDate;
use common::{CostByModel, CostByUser, CostRecord, ModelInfo, UserInfo};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait CostService: Send + Sync {
    async fn health_check(&self) -> anyhow::Result<()>;
    async fn get_daily_cost(&self, start: NaiveDate, end: NaiveDate)
        -> anyhow::Result<Vec<CostRecord>>;
    async fn get_monthly_cost(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_cost_by_users(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByUser>>;
    async fn get_cost_by_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>>;
    async fn get_cost_by_models(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByModel>>;
    async fn get_cost_by_models_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostByModel>>;
    async fn get_cost_by_users_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>>;
    async fn get_cost_by_user_id_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>>;
    async fn get_daily_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_monthly_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_daily_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_monthly_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_daily_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_monthly_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_user_email(&self, user_id: &str) -> anyhow::Result<Option<String>>;
    async fn get_model_name(&self, model_id: &str) -> anyhow::Result<Option<String>>;
    async fn list_users(&self) -> anyhow::Result<Vec<(String, String)>>;
    async fn list_models(&self) -> anyhow::Result<Vec<(String, String)>>;
    async fn get_user_id_by_email(&self, email: &str) -> anyhow::Result<Option<String>>;
    async fn list_users_enriched(&self) -> anyhow::Result<Vec<UserInfo>>;
    async fn get_user_info(&self, user_id: &str) -> anyhow::Result<Option<UserInfo>>;
    async fn list_models_enriched(&self) -> anyhow::Result<Vec<ModelInfo>>;
    async fn list_models_enriched_by_user_id(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<ModelInfo>>;
    async fn get_model_info(&self, model_id: &str) -> anyhow::Result<Option<ModelInfo>>;
}

pub struct RealCostService {
    pub pool: PgPool,
    pub cost_pool: PgPool,
}

#[async_trait]
impl CostService for RealCostService {
    async fn health_check(&self) -> anyhow::Result<()> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.cost_pool)
            .await?;
        Ok(())
    }

    async fn get_daily_cost(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_daily_cost(&self.cost_pool, start, end).await?)
    }

    async fn get_monthly_cost(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_monthly_cost(&self.cost_pool, start, end).await?)
    }

    async fn get_cost_by_users(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs = db::get_cost_by_users(&self.cost_pool, start, end).await?;
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await?;
        }
        Ok(costs)
    }

    async fn get_cost_by_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs = db::get_cost_by_user_id(&self.cost_pool, start, end, user_id).await?;
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await?;
        }
        Ok(costs)
    }

    async fn get_cost_by_models(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByModel>> {
        let mut costs = db::get_cost_by_models(&self.cost_pool, start, end).await?;
        for cost in &mut costs {
            cost.model_name = self.get_model_name(&cost.model_id).await?;
        }
        Ok(costs)
    }

    async fn get_cost_by_models_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostByModel>> {
        let mut costs =
            db::get_cost_by_models_for_user_id(&self.cost_pool, start, end, user_id).await?;
        for cost in &mut costs {
            cost.model_name = self.get_model_name(&cost.model_id).await?;
        }
        Ok(costs)
    }

    async fn get_cost_by_users_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs =
            db::get_cost_by_users_for_model_id(&self.cost_pool, start, end, model_id).await?;
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await?;
        }
        Ok(costs)
    }

    async fn get_cost_by_user_id_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs =
            db::get_cost_by_user_id_for_model_id(&self.cost_pool, start, end, user_id, model_id)
                .await?;
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await?;
        }
        Ok(costs)
    }

    async fn get_daily_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_daily_cost_for_user_id(&self.cost_pool, start, end, user_id).await?)
    }

    async fn get_monthly_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_monthly_cost_for_user_id(&self.cost_pool, start, end, user_id).await?)
    }

    async fn get_daily_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_daily_cost_for_model_id(&self.cost_pool, start, end, model_id).await?)
    }

    async fn get_monthly_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_monthly_cost_for_model_id(&self.cost_pool, start, end, model_id).await?)
    }

    async fn get_daily_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(
            db::get_daily_cost_for_user_id_and_model_id(
                &self.cost_pool,
                start,
                end,
                user_id,
                model_id,
            )
            .await?,
        )
    }

    async fn get_monthly_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(
            db::get_monthly_cost_for_user_id_and_model_id(
                &self.cost_pool,
                start,
                end,
                user_id,
                model_id,
            )
            .await?,
        )
    }

    async fn get_user_email(&self, user_id: &str) -> anyhow::Result<Option<String>> {
        let uuid = Uuid::parse_str(user_id)?;
        Ok(db::get_user_email(&self.pool, uuid).await)
    }

    async fn get_model_name(&self, model_id: &str) -> anyhow::Result<Option<String>> {
        let uuid = Uuid::parse_str(model_id)?;
        Ok(db::get_model_name(&self.pool, uuid).await)
    }

    async fn list_users(&self) -> anyhow::Result<Vec<(String, String)>> {
        Ok(db::list_users(&self.pool)
            .await?
            .into_iter()
            .map(|(id, email)| (id.to_string(), email))
            .collect())
    }

    async fn list_models(&self) -> anyhow::Result<Vec<(String, String)>> {
        Ok(db::list_models(&self.pool)
            .await?
            .into_iter()
            .map(|(id, name)| (id.to_string(), name))
            .collect())
    }

    async fn get_user_id_by_email(&self, email: &str) -> anyhow::Result<Option<String>> {
        Ok(db::get_user_id_by_email(&self.pool, email)
            .await
            .map(|uuid| uuid.to_string()))
    }

    async fn list_users_enriched(&self) -> anyhow::Result<Vec<UserInfo>> {
        Ok(db::list_users_enriched(&self.pool).await?)
    }

    async fn get_user_info(&self, user_id: &str) -> anyhow::Result<Option<UserInfo>> {
        let uuid = Uuid::parse_str(user_id)?;
        Ok(db::get_user_info(&self.pool, uuid).await)
    }

    async fn list_models_enriched(&self) -> anyhow::Result<Vec<ModelInfo>> {
        Ok(db::list_models_enriched(&self.pool).await?)
    }

    async fn list_models_enriched_by_user_id(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<ModelInfo>> {
        let uuid = Uuid::parse_str(user_id)?;
        Ok(db::list_models_enriched_by_user_id(&self.pool, uuid).await?)
    }

    async fn get_model_info(&self, model_id: &str) -> anyhow::Result<Option<ModelInfo>> {
        let uuid = Uuid::parse_str(model_id)?;
        Ok(db::get_model_info(&self.pool, uuid).await)
    }
}
