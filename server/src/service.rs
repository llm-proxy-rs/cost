use async_trait::async_trait;
use aws_sdk_costexplorer::Client as CeClient;
use chrono::Datelike;
use common::{
    ApiKeyInfo, CostByModel, CostByUser, CostRecord, InferenceProfileInfo, ModelInfo, UserInfo,
};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[async_trait]
pub trait CostService: Send + Sync {
    async fn get_daily_cost(&self, start: &str, end: &str) -> Vec<CostRecord>;
    async fn get_monthly_cost(&self, start: &str, end: &str) -> Vec<CostRecord>;
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
    async fn get_daily_cost_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_monthly_cost_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_daily_cost_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostRecord>;
    async fn get_monthly_cost_for_model(
        &self,
        start: &str,
        end: &str,
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
    async fn get_model_info(&self, model_id: &str) -> Option<ModelInfo>;
}

pub struct RealCostService {
    pub pool: PgPool,
    pub ce_client: CeClient,
    pub cache_pool: PgPool,
}

impl RealCostService {
    async fn cached_query<F>(
        &self,
        query_type: &str,
        filter_id: &str,
        start: &str,
        end: &str,
        ce_fn: F,
    ) -> Vec<CostRecord>
    where
        F: for<'a> Fn(
            &'a str,
            &'a str,
        ) -> Pin<Box<dyn Future<Output = Vec<CostRecord>> + Send + 'a>>,
    {
        let today = chrono::Utc::now().date_naive();
        let cutoff = if query_type == "monthly" {
            format!("{:04}-{:02}-01", today.year(), today.month())
        } else {
            today.format("%Y-%m-%d").to_string()
        };
        let cache_end = if cutoff.as_str() < end { &cutoff } else { end };

        let mut results = Vec::new();

        // Finalized portion
        if start < cache_end {
            let cached =
                db::get_cached_costs(&self.cache_pool, query_type, filter_id, start, cache_end)
                    .await
                    .unwrap_or_default();

            if !cached.is_empty() && cached[0].date.as_str() <= start {
                results.extend(cached);
            } else {
                let from_ce = ce_fn(start, cache_end).await;
                // Background upsert
                let pool = self.cache_pool.clone();
                let qt = query_type.to_string();
                let fi = filter_id.to_string();
                let records = from_ce.clone();
                tokio::spawn(async move {
                    let _ = db::upsert_cached_costs(&pool, &qt, &fi, &records).await;
                });
                results.extend(from_ce);
            }
        }

        // Live portion
        if cache_end < end {
            let live = ce_fn(cache_end, end).await;
            results.extend(live);
        }

        results
    }
}

#[async_trait]
impl CostService for RealCostService {
    async fn get_daily_cost(&self, start: &str, end: &str) -> Vec<CostRecord> {
        let client = self.ce_client.clone();
        self.cached_query("daily", "", start, end, |s, e| {
            let client = client.clone();
            Box::pin(async move {
                ce::get_daily_cost(&client, s, e).await.unwrap_or_else(|e| {
                    log::error!("Failed to call CE API (get_daily_cost): {e}");
                    Vec::new()
                })
            })
        })
        .await
    }

    async fn get_monthly_cost(&self, start: &str, end: &str) -> Vec<CostRecord> {
        let client = self.ce_client.clone();
        self.cached_query("monthly", "", start, end, |s, e| {
            let client = client.clone();
            Box::pin(async move {
                ce::get_monthly_cost(&client, s, e)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to call CE API (get_monthly_cost): {e}");
                        Vec::new()
                    })
            })
        })
        .await
    }

    async fn get_cost_by_user(&self, start: &str, end: &str) -> Vec<CostByUser> {
        let mut costs = ce::get_cost_by_user(&self.ce_client, start, end)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to call CE API (get_cost_by_user): {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_cost_by_model(&self, start: &str, end: &str) -> Vec<CostByModel> {
        let mut costs = ce::get_cost_by_model(&self.ce_client, start, end)
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to call CE API (get_cost_by_model): {e}");
                Vec::new()
            });
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
            .unwrap_or_else(|e| {
                log::error!("Failed to call CE API (get_cost_by_model_for_user): {e}");
                Vec::new()
            });
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
            .unwrap_or_else(|e| {
                log::error!("Failed to call CE API (get_cost_by_user_for_model): {e}");
                Vec::new()
            });
        for cost in &mut costs {
            cost.user_email = self.get_user_email(&cost.user_id).await;
        }
        costs
    }

    async fn get_daily_cost_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostRecord> {
        let client = self.ce_client.clone();
        let uid = user_id.to_string();
        let filter_id = format!("user:{}", user_id);
        self.cached_query("daily", &filter_id, start, end, |s, e| {
            let client = client.clone();
            let uid = uid.clone();
            Box::pin(async move {
                ce::get_daily_cost_for_user(&client, s, e, &uid)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to call CE API (get_daily_cost_for_user): {e}");
                        Vec::new()
                    })
            })
        })
        .await
    }

    async fn get_monthly_cost_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostRecord> {
        let client = self.ce_client.clone();
        let uid = user_id.to_string();
        let filter_id = format!("user:{}", user_id);
        self.cached_query("monthly", &filter_id, start, end, |s, e| {
            let client = client.clone();
            let uid = uid.clone();
            Box::pin(async move {
                ce::get_monthly_cost_for_user(&client, s, e, &uid)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to call CE API (get_monthly_cost_for_user): {e}");
                        Vec::new()
                    })
            })
        })
        .await
    }

    async fn get_daily_cost_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostRecord> {
        let client = self.ce_client.clone();
        let mid = model_id.to_string();
        let filter_id = format!("model:{}", model_id);
        self.cached_query("daily", &filter_id, start, end, |s, e| {
            let client = client.clone();
            let mid = mid.clone();
            Box::pin(async move {
                ce::get_daily_cost_for_model(&client, s, e, &mid)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to call CE API (get_daily_cost_for_model): {e}");
                        Vec::new()
                    })
            })
        })
        .await
    }

    async fn get_monthly_cost_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostRecord> {
        let client = self.ce_client.clone();
        let mid = model_id.to_string();
        let filter_id = format!("model:{}", model_id);
        self.cached_query("monthly", &filter_id, start, end, |s, e| {
            let client = client.clone();
            let mid = mid.clone();
            Box::pin(async move {
                ce::get_monthly_cost_for_model(&client, s, e, &mid)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Failed to call CE API (get_monthly_cost_for_model): {e}");
                        Vec::new()
                    })
            })
        })
        .await
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

    async fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let uuid = Uuid::parse_str(model_id).ok()?;
        db::get_model_info(&self.pool, uuid).await
    }
}

// --- Demo service for --demo mode ---

pub struct DemoCostService;

const ALICE_ID: &str = "00000000-0000-0000-0000-000000000001";
const BOB_ID: &str = "00000000-0000-0000-0000-000000000002";
const CHARLIE_ID: &str = "00000000-0000-0000-0000-000000000003";

const OPUS_ID: &str = "00000000-0000-0000-0000-000000000011";
const SONNET_ID: &str = "00000000-0000-0000-0000-000000000012";
const HAIKU_ID: &str = "00000000-0000-0000-0000-000000000013";
const SONNET35_ID: &str = "00000000-0000-0000-0000-000000000014";

impl DemoCostService {
    fn users() -> Vec<(String, String)> {
        vec![
            (ALICE_ID.into(), "alice@example.com".into()),
            (BOB_ID.into(), "bob@example.com".into()),
            (CHARLIE_ID.into(), "charlie@example.com".into()),
        ]
    }

    fn models() -> Vec<(String, String)> {
        vec![
            (OPUS_ID.into(), "claude-3-opus".into()),
            (SONNET_ID.into(), "claude-3-sonnet".into()),
            (HAIKU_ID.into(), "claude-3-haiku".into()),
            (SONNET35_ID.into(), "claude-3.5-sonnet".into()),
        ]
    }

    fn daily_costs() -> Vec<CostRecord> {
        let today = chrono::Utc::now().date_naive();
        let amounts = [45.20, 52.80, 38.90, 61.40, 55.10, 48.60, 42.30];
        (0..7)
            .map(|i| CostRecord {
                date: (today - chrono::Duration::days(6 - i))
                    .format("%Y-%m-%d")
                    .to_string(),
                amount: amounts[i as usize],
                currency: "USD".into(),
            })
            .collect()
    }

    fn monthly_costs() -> Vec<CostRecord> {
        let today = chrono::Utc::now().date_naive();
        let amounts = [820.50, 945.30, 780.10, 1102.40, 890.70, 960.20];
        (0..6)
            .map(|i| {
                let months_back = 5 - i as i32;
                let mut y = today.year();
                let mut m = today.month() as i32 - months_back;
                while m <= 0 {
                    m += 12;
                    y -= 1;
                }
                let date = format!("{:04}-{:02}-01", y, m);
                CostRecord {
                    date,
                    amount: amounts[i],
                    currency: "USD".into(),
                }
            })
            .collect()
    }

    fn user_fraction(user_id: &str) -> f64 {
        let data = Self::cost_by_user_model();
        let user_total: f64 = data
            .iter()
            .filter(|(uid, _, _)| uid == user_id)
            .map(|(_, _, a)| a)
            .sum();
        let grand_total: f64 = data.iter().map(|(_, _, a)| a).sum();
        if grand_total == 0.0 {
            0.0
        } else {
            user_total / grand_total
        }
    }

    fn model_fraction(model_id: &str) -> f64 {
        let data = Self::cost_by_user_model();
        let model_total: f64 = data
            .iter()
            .filter(|(_, mid, _)| mid == model_id)
            .map(|(_, _, a)| a)
            .sum();
        let grand_total: f64 = data.iter().map(|(_, _, a)| a).sum();
        if grand_total == 0.0 {
            0.0
        } else {
            model_total / grand_total
        }
    }

    /// (user_id, model_id, amount)
    fn cost_by_user_model() -> Vec<(String, String, f64)> {
        vec![
            (ALICE_ID.into(), OPUS_ID.into(), 48.30),
            (ALICE_ID.into(), SONNET_ID.into(), 35.20),
            (ALICE_ID.into(), HAIKU_ID.into(), 22.00),
            (ALICE_ID.into(), SONNET35_ID.into(), 15.00),
            (BOB_ID.into(), OPUS_ID.into(), 32.50),
            (BOB_ID.into(), SONNET_ID.into(), 25.80),
            (BOB_ID.into(), HAIKU_ID.into(), 12.60),
            (BOB_ID.into(), SONNET35_ID.into(), 18.40),
            (CHARLIE_ID.into(), OPUS_ID.into(), 15.00),
            (CHARLIE_ID.into(), SONNET_ID.into(), 11.40),
            (CHARLIE_ID.into(), HAIKU_ID.into(), 8.00),
            (CHARLIE_ID.into(), SONNET35_ID.into(), 10.80),
        ]
    }

    fn all_api_keys() -> Vec<(String, ApiKeyInfo)> {
        vec![
            (
                ALICE_ID.into(),
                ApiKeyInfo {
                    api_key_id: "ak-alice-001".into(),
                    api_key_preview: "sk-...a1b2".into(),
                    is_disabled: false,
                    created_at: "2024-01-15".into(),
                },
            ),
            (
                ALICE_ID.into(),
                ApiKeyInfo {
                    api_key_id: "ak-alice-002".into(),
                    api_key_preview: "sk-...c3d4".into(),
                    is_disabled: true,
                    created_at: "2024-02-20".into(),
                },
            ),
            (
                BOB_ID.into(),
                ApiKeyInfo {
                    api_key_id: "ak-bob-001".into(),
                    api_key_preview: "sk-...e5f6".into(),
                    is_disabled: false,
                    created_at: "2024-03-10".into(),
                },
            ),
            (
                BOB_ID.into(),
                ApiKeyInfo {
                    api_key_id: "ak-bob-002".into(),
                    api_key_preview: "sk-...g7h8".into(),
                    is_disabled: true,
                    created_at: "2024-01-05".into(),
                },
            ),
            (
                CHARLIE_ID.into(),
                ApiKeyInfo {
                    api_key_id: "ak-charlie-001".into(),
                    api_key_preview: "sk-...i9j0".into(),
                    is_disabled: false,
                    created_at: "2024-04-01".into(),
                },
            ),
            (
                CHARLIE_ID.into(),
                ApiKeyInfo {
                    api_key_id: "ak-charlie-002".into(),
                    api_key_preview: "sk-...k1l2".into(),
                    is_disabled: true,
                    created_at: "2024-02-14".into(),
                },
            ),
        ]
    }

    fn all_profiles() -> Vec<InferenceProfileInfo> {
        vec![
            InferenceProfileInfo {
                inference_profile_id: "ip-alice-opus".into(),
                model_id: OPUS_ID.into(),
                model_name: Some("claude-3-opus".into()),
                user_id: ALICE_ID.into(),
                user_email: Some("alice@example.com".into()),
                created_at: "2024-01-20".into(),
            },
            InferenceProfileInfo {
                inference_profile_id: "ip-alice-sonnet".into(),
                model_id: SONNET_ID.into(),
                model_name: Some("claude-3-sonnet".into()),
                user_id: ALICE_ID.into(),
                user_email: Some("alice@example.com".into()),
                created_at: "2024-01-20".into(),
            },
            InferenceProfileInfo {
                inference_profile_id: "ip-bob-sonnet".into(),
                model_id: SONNET_ID.into(),
                model_name: Some("claude-3-sonnet".into()),
                user_id: BOB_ID.into(),
                user_email: Some("bob@example.com".into()),
                created_at: "2024-03-15".into(),
            },
            InferenceProfileInfo {
                inference_profile_id: "ip-bob-haiku".into(),
                model_id: HAIKU_ID.into(),
                model_name: Some("claude-3-haiku".into()),
                user_id: BOB_ID.into(),
                user_email: Some("bob@example.com".into()),
                created_at: "2024-03-15".into(),
            },
            InferenceProfileInfo {
                inference_profile_id: "ip-charlie-haiku".into(),
                model_id: HAIKU_ID.into(),
                model_name: Some("claude-3-haiku".into()),
                user_id: CHARLIE_ID.into(),
                user_email: Some("charlie@example.com".into()),
                created_at: "2024-04-05".into(),
            },
            InferenceProfileInfo {
                inference_profile_id: "ip-charlie-sonnet35".into(),
                model_id: SONNET35_ID.into(),
                model_name: Some("claude-3.5-sonnet".into()),
                user_id: CHARLIE_ID.into(),
                user_email: Some("charlie@example.com".into()),
                created_at: "2024-04-05".into(),
            },
        ]
    }
}

#[async_trait]
impl CostService for DemoCostService {
    async fn get_daily_cost(&self, start: &str, end: &str) -> Vec<CostRecord> {
        Self::daily_costs()
            .into_iter()
            .filter(|r| r.date.as_str() >= start && r.date.as_str() <= end)
            .collect()
    }

    async fn get_monthly_cost(&self, start: &str, end: &str) -> Vec<CostRecord> {
        Self::monthly_costs()
            .into_iter()
            .filter(|r| r.date.as_str() >= start && r.date.as_str() <= end)
            .collect()
    }

    async fn get_cost_by_user(&self, _start: &str, _end: &str) -> Vec<CostByUser> {
        let mut map: HashMap<String, f64> = HashMap::new();
        for (uid, _, amt) in Self::cost_by_user_model() {
            *map.entry(uid).or_default() += amt;
        }
        let users = Self::users();
        map.into_iter()
            .map(|(uid, amount)| {
                let email = users
                    .iter()
                    .find(|(id, _)| *id == uid)
                    .map(|(_, e)| e.clone());
                CostByUser {
                    user_id: uid,
                    user_email: email,
                    amount,
                    currency: "USD".into(),
                }
            })
            .collect()
    }

    async fn get_cost_by_model(&self, _start: &str, _end: &str) -> Vec<CostByModel> {
        let mut map: HashMap<String, f64> = HashMap::new();
        for (_, mid, amt) in Self::cost_by_user_model() {
            *map.entry(mid).or_default() += amt;
        }
        let models = Self::models();
        map.into_iter()
            .map(|(mid, amount)| {
                let name = models
                    .iter()
                    .find(|(id, _)| *id == mid)
                    .map(|(_, n)| n.clone());
                CostByModel {
                    model_id: mid,
                    model_name: name,
                    amount,
                    currency: "USD".into(),
                }
            })
            .collect()
    }

    async fn get_cost_by_model_for_user(
        &self,
        _start: &str,
        _end: &str,
        user_id: &str,
    ) -> Vec<CostByModel> {
        let models = Self::models();
        Self::cost_by_user_model()
            .into_iter()
            .filter(|(uid, _, _)| uid == user_id)
            .map(|(_, mid, amount)| {
                let name = models
                    .iter()
                    .find(|(id, _)| *id == mid)
                    .map(|(_, n)| n.clone());
                CostByModel {
                    model_id: mid,
                    model_name: name,
                    amount,
                    currency: "USD".into(),
                }
            })
            .collect()
    }

    async fn get_cost_by_user_for_model(
        &self,
        _start: &str,
        _end: &str,
        model_id: &str,
    ) -> Vec<CostByUser> {
        let users = Self::users();
        Self::cost_by_user_model()
            .into_iter()
            .filter(|(_, mid, _)| mid == model_id)
            .map(|(uid, _, amount)| {
                let email = users
                    .iter()
                    .find(|(id, _)| *id == uid)
                    .map(|(_, e)| e.clone());
                CostByUser {
                    user_id: uid,
                    user_email: email,
                    amount,
                    currency: "USD".into(),
                }
            })
            .collect()
    }

    async fn get_daily_cost_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostRecord> {
        let fraction = Self::user_fraction(user_id);
        Self::daily_costs()
            .into_iter()
            .filter(|r| r.date.as_str() >= start && r.date.as_str() <= end)
            .map(|r| CostRecord {
                amount: r.amount * fraction,
                ..r
            })
            .collect()
    }

    async fn get_monthly_cost_for_user(
        &self,
        start: &str,
        end: &str,
        user_id: &str,
    ) -> Vec<CostRecord> {
        let fraction = Self::user_fraction(user_id);
        Self::monthly_costs()
            .into_iter()
            .filter(|r| r.date.as_str() >= start && r.date.as_str() <= end)
            .map(|r| CostRecord {
                amount: r.amount * fraction,
                ..r
            })
            .collect()
    }

    async fn get_daily_cost_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostRecord> {
        let fraction = Self::model_fraction(model_id);
        Self::daily_costs()
            .into_iter()
            .filter(|r| r.date.as_str() >= start && r.date.as_str() <= end)
            .map(|r| CostRecord {
                amount: r.amount * fraction,
                ..r
            })
            .collect()
    }

    async fn get_monthly_cost_for_model(
        &self,
        start: &str,
        end: &str,
        model_id: &str,
    ) -> Vec<CostRecord> {
        let fraction = Self::model_fraction(model_id);
        Self::monthly_costs()
            .into_iter()
            .filter(|r| r.date.as_str() >= start && r.date.as_str() <= end)
            .map(|r| CostRecord {
                amount: r.amount * fraction,
                ..r
            })
            .collect()
    }

    async fn get_user_email(&self, user_id: &str) -> Option<String> {
        Self::users()
            .into_iter()
            .find(|(id, _)| id == user_id)
            .map(|(_, e)| e)
    }

    async fn get_model_name(&self, model_id: &str) -> Option<String> {
        Self::models()
            .into_iter()
            .find(|(id, _)| id == model_id)
            .map(|(_, n)| n)
    }

    async fn list_users(&self) -> Vec<(String, String)> {
        Self::users()
    }

    async fn list_models(&self) -> Vec<(String, String)> {
        Self::models()
    }

    async fn get_user_id_by_email(&self, email: &str) -> Option<String> {
        Self::users()
            .into_iter()
            .find(|(_, e)| e == email)
            .map(|(id, _)| id)
    }

    async fn list_users_enriched(&self) -> Vec<UserInfo> {
        let all_keys = Self::all_api_keys();
        let all_profiles = Self::all_profiles();
        Self::users()
            .into_iter()
            .map(|(uid, email)| {
                let user_keys: Vec<_> = all_keys.iter().filter(|(id, _)| *id == uid).collect();
                let active = user_keys.iter().filter(|(_, k)| !k.is_disabled).count() as i64;
                let profile_count = all_profiles.iter().filter(|p| p.user_id == uid).count() as i64;
                UserInfo {
                    user_id: uid,
                    user_email: email,
                    created_at: "2024-01-01".into(),
                    api_key_count: user_keys.len() as i64,
                    active_api_key_count: active,
                    inference_profile_count: profile_count,
                }
            })
            .collect()
    }

    async fn get_user_info(&self, user_id: &str) -> Option<UserInfo> {
        self.list_users_enriched()
            .await
            .into_iter()
            .find(|u| u.user_id == user_id)
    }

    async fn list_models_enriched(&self) -> Vec<ModelInfo> {
        let all_profiles = Self::all_profiles();
        Self::models()
            .into_iter()
            .map(|(mid, name)| {
                let user_count = all_profiles
                    .iter()
                    .filter(|p| p.model_id == mid)
                    .map(|p| &p.user_id)
                    .collect::<HashSet<_>>()
                    .len() as i64;
                ModelInfo {
                    model_id: mid,
                    model_name: name,
                    is_disabled: false,
                    protected: false,
                    user_count,
                }
            })
            .collect()
    }

    async fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        self.list_models_enriched()
            .await
            .into_iter()
            .find(|m| m.model_id == model_id)
    }
}
