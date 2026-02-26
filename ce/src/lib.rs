use anyhow::Result;
use aws_sdk_costexplorer::types::{
    DateInterval, Expression, Granularity, GroupDefinition, GroupDefinitionType, TagValues,
};
use aws_sdk_costexplorer::Client;
use common::CostRow;

pub async fn new_client() -> Client {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    Client::new(&config)
}

pub async fn get_daily_cost_by_user_and_model(
    client: &Client,
    start: &str,
    end: &str,
) -> Result<Vec<CostRow>> {
    let mut results = Vec::new();
    let mut next_page_token: Option<String> = None;

    loop {
        let mut req = client
            .get_cost_and_usage()
            .time_period(DateInterval::builder().start(start).end(end).build()?)
            .granularity(Granularity::Daily)
            .metrics("BlendedCost")
            .group_by(
                GroupDefinition::builder()
                    .r#type(GroupDefinitionType::Tag)
                    .key("GatewayUserId")
                    .build(),
            )
            .group_by(
                GroupDefinition::builder()
                    .r#type(GroupDefinitionType::Tag)
                    .key("GatewayModelId")
                    .build(),
            )
            .filter(
                Expression::builder()
                    .and(
                        Expression::builder()
                            .not(
                                Expression::builder()
                                    .tags(
                                        TagValues::builder()
                                            .key("GatewayUserId")
                                            .match_options(
                                                aws_sdk_costexplorer::types::MatchOption::Absent,
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .and(
                        Expression::builder()
                            .not(
                                Expression::builder()
                                    .tags(
                                        TagValues::builder()
                                            .key("GatewayModelId")
                                            .match_options(
                                                aws_sdk_costexplorer::types::MatchOption::Absent,
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            );

        if let Some(token) = &next_page_token {
            req = req.next_page_token(token.clone());
        }

        let resp = req.send().await?;

        for result_by_time in resp.results_by_time() {
            let date = result_by_time
                .time_period()
                .map(|tp| tp.start().to_string())
                .unwrap_or_default();

            for group in result_by_time.groups() {
                let keys: Vec<&str> = group.keys().iter().map(|s| s.as_str()).collect();
                let user_id = keys
                    .first()
                    .map(|k| k.strip_prefix("GatewayUserId$").unwrap_or(k))
                    .unwrap_or_default();
                let model_id = keys
                    .get(1)
                    .map(|k| k.strip_prefix("GatewayModelId$").unwrap_or(k))
                    .unwrap_or_default();

                if user_id.is_empty() || model_id.is_empty() {
                    continue;
                }

                let (amount, currency) = extract_blended_cost(group.metrics());
                results.push(CostRow {
                    date: date.clone(),
                    user_id: user_id.to_string(),
                    model_id: model_id.to_string(),
                    amount,
                    currency,
                });
            }
        }

        next_page_token = resp.next_page_token().map(|s| s.to_string());
        if next_page_token.is_none() {
            break;
        }
    }

    Ok(results)
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

#[cfg(test)]
mod tests {
    use super::*;

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
