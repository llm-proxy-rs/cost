use anyhow::{Context, Result};
use aws_sdk_costexplorer::types::{
    DateInterval, Expression, Granularity, GroupDefinition, GroupDefinitionType, TagValues,
};
use aws_sdk_costexplorer::Client;
use chrono::NaiveDate;
use common::{CostRow, GithubCostRow};

pub async fn new_client() -> Client {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    Client::new(&config)
}

/// Build a Cost Explorer client whose credentials come from assuming `role_arn` — e.g. a
/// read-only role in another AWS account that owns the GitHub-tagged cost. The batch's own
/// principal must be allowed to `sts:AssumeRole` that role. Cost Explorer is a global
/// service reachable only via us-east-1, so the region is pinned there.
pub async fn new_client_for_role(role_arn: &str) -> Client {
    use aws_config::sts::AssumeRoleProvider;
    use aws_sdk_costexplorer::config::Region;

    let provider = AssumeRoleProvider::builder(role_arn)
        .session_name("cost-batch-github")
        .region(Region::new("us-east-1"))
        .build()
        .await;
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .credentials_provider(provider)
        .region(Region::new("us-east-1"))
        .load()
        .await;
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
            let date_str = result_by_time
                .time_period()
                .map(|tp| tp.start().to_string())
                .unwrap_or_default(); // no time period: "" makes the date parse below error
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("invalid date from CE API")?;

            for group in result_by_time.groups() {
                let keys = group.keys();
                let user_id = keys
                    .first()
                    .context("CE group missing key for tag GatewayUserId")?
                    .strip_prefix("GatewayUserId$")
                    .context("CE group key not in 'GatewayUserId$<value>' form")?;
                let model_id = keys
                    .get(1)
                    .context("CE group missing key for tag GatewayModelId")?
                    .strip_prefix("GatewayModelId$")
                    .context("CE group key not in 'GatewayModelId$<value>' form")?;

                let (amount, currency) = extract_blended_cost(group.metrics());
                results.push(CostRow {
                    date,
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

pub async fn get_daily_cost_by_github_org_and_repo(
    client: &Client,
    start: &str,
    end: &str,
    org_tag_key: &str,
    repo_tag_key: &str,
) -> Result<Vec<GithubCostRow>> {
    let org_group_prefix = format!("{}$", org_tag_key);
    let repo_group_prefix = format!("{}$", repo_tag_key);
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
                    .key(org_tag_key)
                    .build(),
            )
            .group_by(
                GroupDefinition::builder()
                    .r#type(GroupDefinitionType::Tag)
                    .key(repo_tag_key)
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
                                            .key(org_tag_key)
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
                                            .key(repo_tag_key)
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
            let date_str = result_by_time
                .time_period()
                .map(|tp| tp.start().to_string())
                .unwrap_or_default(); // no time period: "" makes the date parse below error
            let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .context("invalid date from CE API")?;

            for group in result_by_time.groups() {
                let keys = group.keys();
                let org_name = keys
                    .first()
                    .with_context(|| format!("CE group missing key for tag {org_tag_key}"))?
                    .strip_prefix(org_group_prefix.as_str())
                    .with_context(|| format!("CE group key not in '{org_tag_key}$<value>' form"))?;
                let repo_name = keys
                    .get(1)
                    .with_context(|| format!("CE group missing key for tag {repo_tag_key}"))?
                    .strip_prefix(repo_group_prefix.as_str())
                    .with_context(|| {
                        format!("CE group key not in '{repo_tag_key}$<value>' form")
                    })?;

                let (amount, currency) = extract_blended_cost(group.metrics());
                results.push(GithubCostRow {
                    date,
                    org_name: org_name.to_string(),
                    repo_name: repo_name.to_string(),
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
            // Lenient on metric contents: a missing/non-numeric amount counts as 0.0,
            // a missing unit as USD — a malformed metric shouldn't fail the whole batch.
            let amount = mv.amount().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let currency = mv.unit().unwrap_or("USD").to_string();
            (amount, currency)
        })
        // Group had no BlendedCost metric at all: treat as zero cost.
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
