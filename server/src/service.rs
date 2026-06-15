use async_trait::async_trait;
use chrono::NaiveDate;
use common::{
    CostByGithub, CostByModel, CostByUser, CostRecord, GithubOrgCost, ModelInfo, UserInfo,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait CostService: Send + Sync {
    async fn health_check(&self) -> anyhow::Result<()>;
    async fn get_daily_cost(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>>;
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
    async fn get_cost_by_github(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByGithub>>;
    async fn get_github_orgs(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<GithubOrgCost>>;
    async fn get_github_repos_for_org(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
    ) -> anyhow::Result<Vec<CostByGithub>>;
    async fn get_github_daily_for_repo(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
        repo_name: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_github_monthly_for_repo(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
        repo_name: &str,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_github_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>>;
    async fn get_github_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>>;
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

    async fn get_cost_by_github(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByGithub>> {
        Ok(db::get_cost_by_github(&self.cost_pool, start, end).await?)
    }

    async fn get_github_orgs(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<GithubOrgCost>> {
        Ok(db::get_github_orgs(&self.cost_pool, start, end).await?)
    }

    async fn get_github_repos_for_org(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
    ) -> anyhow::Result<Vec<CostByGithub>> {
        Ok(db::get_github_repos_for_org(&self.cost_pool, start, end, org_name).await?)
    }

    async fn get_github_daily_for_repo(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
        repo_name: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_github_daily_for_repo(&self.cost_pool, start, end, org_name, repo_name).await?)
    }

    async fn get_github_monthly_for_repo(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
        repo_name: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(
            db::get_github_monthly_for_repo(&self.cost_pool, start, end, org_name, repo_name)
                .await?,
        )
    }

    async fn get_github_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_github_daily(&self.cost_pool, start, end).await?)
    }

    async fn get_github_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_github_monthly(&self.cost_pool, start, end).await?)
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
        Ok(db::get_daily_cost_for_user_id_and_model_id(
            &self.cost_pool,
            start,
            end,
            user_id,
            model_id,
        )
        .await?)
    }

    async fn get_monthly_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        Ok(db::get_monthly_cost_for_user_id_and_model_id(
            &self.cost_pool,
            start,
            end,
            user_id,
            model_id,
        )
        .await?)
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

/// If `email` ends with one of the legacy `from` domains, return the email with that
/// suffix replaced by the matching `to`. Returns `None` when no legacy domain matches.
pub fn rewrite_legacy_email(email: &str, map: &[(String, String)]) -> Option<String> {
    for (from, to) in map {
        if let Some(stem) = email.strip_suffix(from.as_str()) {
            return Some(format!("{}{}", stem, to));
        }
    }
    None
}

/// Wraps another `CostService`, remapping user emails through a legacy domain map. Both
/// inbound lookups (a migrated login email -> the stored record) and outbound display
/// emails are rewritten, so the `legacy` build behaves like user mode for accounts whose
/// email domain changed. Non-email data is passed straight through.
pub struct LegacyEmailService {
    inner: Arc<dyn CostService>,
    map: Vec<(String, String)>,
}

impl LegacyEmailService {
    pub fn new(inner: Arc<dyn CostService>, map: Vec<(String, String)>) -> Self {
        Self { inner, map }
    }

    fn remap(&self, email: &str) -> String {
        // No legacy domain matched: leave the email unchanged.
        rewrite_legacy_email(email, &self.map).unwrap_or_else(|| email.to_string())
    }

    fn remap_opt(&self, email: Option<String>) -> Option<String> {
        email.map(|e| self.remap(&e))
    }
}

#[async_trait]
impl CostService for LegacyEmailService {
    async fn health_check(&self) -> anyhow::Result<()> {
        self.inner.health_check().await
    }

    async fn get_daily_cost(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner.get_daily_cost(start, end).await
    }

    async fn get_monthly_cost(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner.get_monthly_cost(start, end).await
    }

    async fn get_cost_by_users(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs = self.inner.get_cost_by_users(start, end).await?;
        for c in &mut costs {
            c.user_email = self.remap_opt(c.user_email.take());
        }
        Ok(costs)
    }

    async fn get_cost_by_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs = self.inner.get_cost_by_user_id(start, end, user_id).await?;
        for c in &mut costs {
            c.user_email = self.remap_opt(c.user_email.take());
        }
        Ok(costs)
    }

    async fn get_cost_by_models(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByModel>> {
        self.inner.get_cost_by_models(start, end).await
    }

    async fn get_cost_by_github(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostByGithub>> {
        self.inner.get_cost_by_github(start, end).await
    }

    async fn get_github_orgs(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<GithubOrgCost>> {
        self.inner.get_github_orgs(start, end).await
    }

    async fn get_github_repos_for_org(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
    ) -> anyhow::Result<Vec<CostByGithub>> {
        self.inner
            .get_github_repos_for_org(start, end, org_name)
            .await
    }

    async fn get_github_daily_for_repo(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
        repo_name: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_github_daily_for_repo(start, end, org_name, repo_name)
            .await
    }

    async fn get_github_monthly_for_repo(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        org_name: &str,
        repo_name: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_github_monthly_for_repo(start, end, org_name, repo_name)
            .await
    }

    async fn get_github_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner.get_github_daily(start, end).await
    }

    async fn get_github_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner.get_github_monthly(start, end).await
    }

    async fn get_cost_by_models_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostByModel>> {
        self.inner
            .get_cost_by_models_for_user_id(start, end, user_id)
            .await
    }

    async fn get_cost_by_users_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostByUser>> {
        let mut costs = self
            .inner
            .get_cost_by_users_for_model_id(start, end, model_id)
            .await?;
        for c in &mut costs {
            c.user_email = self.remap_opt(c.user_email.take());
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
        let mut costs = self
            .inner
            .get_cost_by_user_id_for_model_id(start, end, user_id, model_id)
            .await?;
        for c in &mut costs {
            c.user_email = self.remap_opt(c.user_email.take());
        }
        Ok(costs)
    }

    async fn get_daily_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_daily_cost_for_user_id(start, end, user_id)
            .await
    }

    async fn get_monthly_cost_for_user_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_monthly_cost_for_user_id(start, end, user_id)
            .await
    }

    async fn get_daily_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_daily_cost_for_model_id(start, end, model_id)
            .await
    }

    async fn get_monthly_cost_for_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_monthly_cost_for_model_id(start, end, model_id)
            .await
    }

    async fn get_daily_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_daily_cost_for_user_id_and_model_id(start, end, user_id, model_id)
            .await
    }

    async fn get_monthly_cost_for_user_id_and_model_id(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        user_id: &str,
        model_id: &str,
    ) -> anyhow::Result<Vec<CostRecord>> {
        self.inner
            .get_monthly_cost_for_user_id_and_model_id(start, end, user_id, model_id)
            .await
    }

    async fn get_user_email(&self, user_id: &str) -> anyhow::Result<Option<String>> {
        Ok(self.remap_opt(self.inner.get_user_email(user_id).await?))
    }

    async fn get_model_name(&self, model_id: &str) -> anyhow::Result<Option<String>> {
        self.inner.get_model_name(model_id).await
    }

    async fn list_users(&self) -> anyhow::Result<Vec<(String, String)>> {
        Ok(self
            .inner
            .list_users()
            .await?
            .into_iter()
            .map(|(id, email)| (id, self.remap(&email)))
            .collect())
    }

    async fn list_models(&self) -> anyhow::Result<Vec<(String, String)>> {
        self.inner.list_models().await
    }

    async fn get_user_id_by_email(&self, email: &str) -> anyhow::Result<Option<String>> {
        self.inner.get_user_id_by_email(&self.remap(email)).await
    }

    async fn list_users_enriched(&self) -> anyhow::Result<Vec<UserInfo>> {
        let mut users = self.inner.list_users_enriched().await?;
        for u in &mut users {
            u.user_email = self.remap(&u.user_email);
        }
        Ok(users)
    }

    async fn get_user_info(&self, user_id: &str) -> anyhow::Result<Option<UserInfo>> {
        let mut info = self.inner.get_user_info(user_id).await?;
        if let Some(i) = &mut info {
            i.user_email = self.remap(&i.user_email);
        }
        Ok(info)
    }

    async fn list_models_enriched(&self) -> anyhow::Result<Vec<ModelInfo>> {
        self.inner.list_models_enriched().await
    }

    async fn list_models_enriched_by_user_id(
        &self,
        user_id: &str,
    ) -> anyhow::Result<Vec<ModelInfo>> {
        self.inner.list_models_enriched_by_user_id(user_id).await
    }

    async fn get_model_info(&self, model_id: &str) -> anyhow::Result<Option<ModelInfo>> {
        self.inner.get_model_info(model_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::rewrite_legacy_email;

    fn map() -> Vec<(String, String)> {
        vec![("@example.com".to_string(), "@example.org".to_string())]
    }

    #[test]
    fn rewrite_matches_suffix() {
        assert_eq!(
            rewrite_legacy_email("alice@example.com", &map()),
            Some("alice@example.org".to_string())
        );
    }

    #[test]
    fn rewrite_no_match_returns_none() {
        assert_eq!(rewrite_legacy_email("bob@other.net", &map()), None);
        assert_eq!(rewrite_legacy_email("carol@example.org", &map()), None);
    }

    #[test]
    fn rewrite_empty_map_returns_none() {
        assert_eq!(rewrite_legacy_email("alice@example.com", &[]), None);
    }

    #[test]
    fn rewrite_first_match_wins() {
        let m = vec![
            ("@a.example".to_string(), "@x.example".to_string()),
            ("@b.example".to_string(), "@y.example".to_string()),
        ];
        assert_eq!(
            rewrite_legacy_email("u@b.example", &m),
            Some("u@y.example".to_string())
        );
    }
}
