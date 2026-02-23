use anyhow::Result;
use aws_sdk_costexplorer::types::{
    DateInterval, Expression, Granularity, GroupDefinition, GroupDefinitionType, TagValues,
};
use aws_sdk_costexplorer::Client;
use common::{CostByModel, CostByUser, CostRecord};

pub async fn new_client() -> Client {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    Client::new(&config)
}

fn tag_filter(tag_key: &str) -> Expression {
    Expression::builder()
        .tags(
            TagValues::builder()
                .key(tag_key)
                .match_options(aws_sdk_costexplorer::types::MatchOption::Absent)
                .build(),
        )
        .build()
}

fn negate_filter(tag_key: &str) -> Expression {
    Expression::builder().not(tag_filter(tag_key)).build()
}

pub async fn get_cost_by_user(client: &Client, start: &str, end: &str) -> Result<Vec<CostByUser>> {
    let resp = client
        .get_cost_and_usage()
        .time_period(DateInterval::builder().start(start).end(end).build()?)
        .granularity(Granularity::Monthly)
        .metrics("BlendedCost")
        .group_by(
            GroupDefinition::builder()
                .r#type(GroupDefinitionType::Tag)
                .key("GatewayUserId")
                .build(),
        )
        .filter(negate_filter("GatewayUserId"))
        .send()
        .await?;

    let mut results = Vec::new();
    for result_by_time in resp.results_by_time() {
        for group in result_by_time.groups() {
            let user_id = group
                .keys()
                .first()
                .map(|k| k.strip_prefix("GatewayUserId$").unwrap_or(k).to_string())
                .unwrap_or_default();
            if user_id.is_empty() {
                continue;
            }
            let (amount, currency) = extract_blended_cost(group.metrics());
            results.push(CostByUser {
                user_id,
                user_email: None,
                amount,
                currency,
            });
        }
    }

    // Aggregate across time periods
    Ok(aggregate_by_key(results, |r| r.user_id.clone())
        .into_iter()
        .map(|(user_id, amount, currency)| CostByUser {
            user_id,
            user_email: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_cost_by_model(
    client: &Client,
    start: &str,
    end: &str,
) -> Result<Vec<CostByModel>> {
    let resp = client
        .get_cost_and_usage()
        .time_period(DateInterval::builder().start(start).end(end).build()?)
        .granularity(Granularity::Monthly)
        .metrics("BlendedCost")
        .group_by(
            GroupDefinition::builder()
                .r#type(GroupDefinitionType::Tag)
                .key("GatewayModelId")
                .build(),
        )
        .filter(negate_filter("GatewayModelId"))
        .send()
        .await?;

    let mut results = Vec::new();
    for result_by_time in resp.results_by_time() {
        for group in result_by_time.groups() {
            let model_id = group
                .keys()
                .first()
                .map(|k| k.strip_prefix("GatewayModelId$").unwrap_or(k).to_string())
                .unwrap_or_default();
            if model_id.is_empty() {
                continue;
            }
            let (amount, currency) = extract_blended_cost(group.metrics());
            results.push(CostByModel {
                model_id,
                model_name: None,
                amount,
                currency,
            });
        }
    }

    Ok(aggregate_by_key(results, |r| r.model_id.clone())
        .into_iter()
        .map(|(model_id, amount, currency)| CostByModel {
            model_id,
            model_name: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_daily_cost(client: &Client, start: &str, end: &str) -> Result<Vec<CostRecord>> {
    let resp = client
        .get_cost_and_usage()
        .time_period(DateInterval::builder().start(start).end(end).build()?)
        .granularity(Granularity::Daily)
        .metrics("BlendedCost")
        .filter(negate_filter("GatewayUserId"))
        .send()
        .await?;

    let mut records = Vec::new();
    for result_by_time in resp.results_by_time() {
        let date = result_by_time
            .time_period()
            .map(|tp| tp.start().to_string())
            .unwrap_or_default();
        let (amount, currency) = extract_blended_cost_from_total(result_by_time.total());
        records.push(CostRecord {
            date,
            amount,
            currency,
        });
    }
    Ok(records)
}

pub async fn get_cost_by_model_for_user(
    client: &Client,
    start: &str,
    end: &str,
    user_id: &str,
) -> Result<Vec<CostByModel>> {
    let user_filter = Expression::builder()
        .tags(
            TagValues::builder()
                .key("GatewayUserId")
                .values(user_id)
                .match_options(aws_sdk_costexplorer::types::MatchOption::Equals)
                .build(),
        )
        .build();

    let resp = client
        .get_cost_and_usage()
        .time_period(DateInterval::builder().start(start).end(end).build()?)
        .granularity(Granularity::Monthly)
        .metrics("BlendedCost")
        .group_by(
            GroupDefinition::builder()
                .r#type(GroupDefinitionType::Tag)
                .key("GatewayModelId")
                .build(),
        )
        .filter(user_filter)
        .send()
        .await?;

    let mut results = Vec::new();
    for result_by_time in resp.results_by_time() {
        for group in result_by_time.groups() {
            let model_id = group
                .keys()
                .first()
                .map(|k| k.strip_prefix("GatewayModelId$").unwrap_or(k).to_string())
                .unwrap_or_default();
            if model_id.is_empty() {
                continue;
            }
            let (amount, currency) = extract_blended_cost(group.metrics());
            results.push(CostByModel {
                model_id,
                model_name: None,
                amount,
                currency,
            });
        }
    }

    Ok(aggregate_by_key(results, |r| r.model_id.clone())
        .into_iter()
        .map(|(model_id, amount, currency)| CostByModel {
            model_id,
            model_name: None,
            amount,
            currency,
        })
        .collect())
}

pub async fn get_cost_by_user_for_model(
    client: &Client,
    start: &str,
    end: &str,
    model_id: &str,
) -> Result<Vec<CostByUser>> {
    let model_filter = Expression::builder()
        .tags(
            TagValues::builder()
                .key("GatewayModelId")
                .values(model_id)
                .match_options(aws_sdk_costexplorer::types::MatchOption::Equals)
                .build(),
        )
        .build();

    let resp = client
        .get_cost_and_usage()
        .time_period(DateInterval::builder().start(start).end(end).build()?)
        .granularity(Granularity::Monthly)
        .metrics("BlendedCost")
        .group_by(
            GroupDefinition::builder()
                .r#type(GroupDefinitionType::Tag)
                .key("GatewayUserId")
                .build(),
        )
        .filter(model_filter)
        .send()
        .await?;

    let mut results = Vec::new();
    for result_by_time in resp.results_by_time() {
        for group in result_by_time.groups() {
            let user_id = group
                .keys()
                .first()
                .map(|k| k.strip_prefix("GatewayUserId$").unwrap_or(k).to_string())
                .unwrap_or_default();
            if user_id.is_empty() {
                continue;
            }
            let (amount, currency) = extract_blended_cost(group.metrics());
            results.push(CostByUser {
                user_id,
                user_email: None,
                amount,
                currency,
            });
        }
    }

    Ok(aggregate_by_key(results, |r| r.user_id.clone())
        .into_iter()
        .map(|(user_id, amount, currency)| CostByUser {
            user_id,
            user_email: None,
            amount,
            currency,
        })
        .collect())
}

fn extract_blended_cost(
    metrics: Option<&std::collections::HashMap<String, aws_sdk_costexplorer::types::MetricValue>>,
) -> (f64, String) {
    metrics
        .and_then(|m| m.get("BlendedCost"))
        .map(|mv| {
            let amount = mv.amount().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let currency = mv.unit().unwrap_or("USD").to_string();
            (amount, currency)
        })
        .unwrap_or((0.0, "USD".to_string()))
}

fn extract_blended_cost_from_total(
    total: Option<&std::collections::HashMap<String, aws_sdk_costexplorer::types::MetricValue>>,
) -> (f64, String) {
    extract_blended_cost(total)
}

fn aggregate_by_key<T>(items: Vec<T>, key_fn: impl Fn(&T) -> String) -> Vec<(String, f64, String)>
where
    T: HasAmountCurrency,
{
    let mut map: std::collections::HashMap<String, (f64, String)> =
        std::collections::HashMap::new();
    for item in &items {
        let key = key_fn(item);
        let entry = map.entry(key).or_insert((0.0, item.currency().to_string()));
        entry.0 += item.amount();
    }
    let mut result: Vec<_> = map
        .into_iter()
        .map(|(k, (amount, currency))| (k, amount, currency))
        .collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

trait HasAmountCurrency {
    fn amount(&self) -> f64;
    fn currency(&self) -> &str;
}

impl HasAmountCurrency for CostByUser {
    fn amount(&self) -> f64 {
        self.amount
    }
    fn currency(&self) -> &str {
        &self.currency
    }
}

impl HasAmountCurrency for CostByModel {
    fn amount(&self) -> f64 {
        self.amount
    }
    fn currency(&self) -> &str {
        &self.currency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_by_key_sums_duplicates() {
        let items = vec![
            CostByUser {
                user_id: "u1".to_string(),
                user_email: None,
                amount: 10.0,
                currency: "USD".to_string(),
            },
            CostByUser {
                user_id: "u1".to_string(),
                user_email: None,
                amount: 20.0,
                currency: "USD".to_string(),
            },
            CostByUser {
                user_id: "u2".to_string(),
                user_email: None,
                amount: 5.0,
                currency: "USD".to_string(),
            },
        ];
        let result = aggregate_by_key(items, |r| r.user_id.clone());
        assert_eq!(result.len(), 2);
        // Sorted descending by amount
        assert_eq!(result[0].0, "u1");
        assert!((result[0].1 - 30.0).abs() < f64::EPSILON);
        assert_eq!(result[1].0, "u2");
        assert!((result[1].1 - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn aggregate_by_key_empty() {
        let items: Vec<CostByUser> = vec![];
        let result = aggregate_by_key(items, |r| r.user_id.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn aggregate_by_key_single_item() {
        let items = vec![CostByModel {
            model_id: "m1".to_string(),
            model_name: None,
            amount: 42.5,
            currency: "EUR".to_string(),
        }];
        let result = aggregate_by_key(items, |r| r.model_id.clone());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "m1");
        assert!((result[0].1 - 42.5).abs() < f64::EPSILON);
        assert_eq!(result[0].2, "EUR");
    }

    #[test]
    fn aggregate_by_key_sorts_descending() {
        let items = vec![
            CostByUser {
                user_id: "low".to_string(),
                user_email: None,
                amount: 1.0,
                currency: "USD".to_string(),
            },
            CostByUser {
                user_id: "high".to_string(),
                user_email: None,
                amount: 100.0,
                currency: "USD".to_string(),
            },
            CostByUser {
                user_id: "mid".to_string(),
                user_email: None,
                amount: 50.0,
                currency: "USD".to_string(),
            },
        ];
        let result = aggregate_by_key(items, |r| r.user_id.clone());
        assert_eq!(result[0].0, "high");
        assert_eq!(result[1].0, "mid");
        assert_eq!(result[2].0, "low");
    }

    #[test]
    fn extract_blended_cost_none_metrics() {
        let (amount, currency) = extract_blended_cost(None);
        assert!((amount - 0.0).abs() < f64::EPSILON);
        assert_eq!(currency, "USD");
    }

    #[test]
    fn extract_blended_cost_with_value() {
        use aws_sdk_costexplorer::types::MetricValue;
        let mut metrics = std::collections::HashMap::new();
        metrics.insert(
            "BlendedCost".to_string(),
            MetricValue::builder().amount("123.45").unit("USD").build(),
        );
        let (amount, currency) = extract_blended_cost(Some(&metrics));
        assert!((amount - 123.45).abs() < f64::EPSILON);
        assert_eq!(currency, "USD");
    }

    #[test]
    fn extract_blended_cost_missing_key() {
        let metrics = std::collections::HashMap::new();
        let (amount, currency) = extract_blended_cost(Some(&metrics));
        assert!((amount - 0.0).abs() < f64::EPSILON);
        assert_eq!(currency, "USD");
    }
}
