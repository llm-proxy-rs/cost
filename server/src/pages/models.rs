use super::make_path;
use common::CostByModel;
use common::CostByUser;
use leptos::either::Either;
use leptos::prelude::*;
use templates::{Breadcrumb, InfoRow, NavLink, Page};

pub fn render_index(base: &str, start: &str, end: &str, costs: &[CostByModel]) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();

    let content = view! {
        <h2>"Cost by Model"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this period."</p>
            })
        } else {
            Either::Right(view! {
                <table>
                    <tr>
                        <th>"Model"</th>
                        <th>"Cost"</th>
                    </tr>
                    {costs.into_iter().map(|c| {
                        let display = c.model_name.clone()
                            .unwrap_or_else(|| c.model_id.clone());
                        let href = make_path(&base_owned, &format!("/models/{}", c.model_id));
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{display}</a></td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
            })
        }}
    };

    Page {
        title: "Cost Explorer - Models".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", make_path(base, "")),
            Breadcrumb::current("Models"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date Range", &format!("{} to {}", start, end)),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_detail(
    base: &str,
    start: &str,
    end: &str,
    model_id: &str,
    model_name: Option<&str>,
    costs: &[CostByUser],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let display_name = model_name.unwrap_or(model_id);
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();

    let content = view! {
        <h2>"Cost by User"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this model in this period."</p>
            })
        } else {
            Either::Right(view! {
                <table>
                    <tr>
                        <th>"User"</th>
                        <th>"Cost"</th>
                    </tr>
                    {costs.into_iter().map(|c| {
                        let display = c.user_email.clone()
                            .unwrap_or_else(|| c.user_id.clone());
                        let href = make_path(&base_owned, &format!("/users/{}", c.user_id));
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{display}</a></td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
            })
        }}
    };

    Page {
        title: format!("Cost Explorer - {}", display_name),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", make_path(base, "")),
            Breadcrumb::link("Models", make_path(base, "/models")),
            Breadcrumb::current(display_name),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date Range", &format!("{} to {}", start, end)),
            InfoRow::new("Model ID", model_id),
            InfoRow::new("Model Name", model_name.unwrap_or("unknown")),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_index_empty() {
        let html = render_index("/", "2024-01-01", "2024-01-31", &[]);
        assert!(html.contains("No cost data found for this period."));
        assert!(html.contains("Cost Explorer - Models"));
    }

    #[test]
    fn render_index_with_data() {
        let costs = vec![
            CostByModel {
                model_id: "model-1".to_string(),
                model_name: Some("claude-3".to_string()),
                amount: 100.0,
                currency: "USD".to_string(),
            },
            CostByModel {
                model_id: "model-2".to_string(),
                model_name: None,
                amount: 50.0,
                currency: "USD".to_string(),
            },
        ];
        let html = render_index("/", "2024-01-01", "2024-01-31", &costs);
        assert!(html.contains("claude-3"));
        assert!(html.contains("model-2"));
        assert!(html.contains("100.00 USD"));
        assert!(html.contains("50.00 USD"));
        assert!(html.contains("150.00 USD"));
        assert!(html.contains("/models/model-1"));
    }

    #[test]
    fn render_index_custom_base() {
        let costs = vec![CostByModel {
            model_id: "model-1".to_string(),
            model_name: None,
            amount: 10.0,
            currency: "USD".to_string(),
        }];
        let html = render_index("/_dashboard", "2024-01-01", "2024-01-31", &costs);
        assert!(html.contains("/_dashboard/models/model-1"));
    }

    #[test]
    fn render_detail_empty() {
        let html = render_detail(
            "/",
            "2024-01-01",
            "2024-01-31",
            "model-1",
            Some("claude-3"),
            &[],
        );
        assert!(html.contains("No cost data found for this model"));
        assert!(html.contains("claude-3"));
    }

    #[test]
    fn render_detail_with_data() {
        let costs = vec![CostByUser {
            user_id: "user-1".to_string(),
            user_email: Some("bob@example.com".to_string()),
            amount: 75.0,
            currency: "USD".to_string(),
        }];
        let html = render_detail(
            "/",
            "2024-01-01",
            "2024-01-31",
            "model-1",
            Some("claude-3"),
            &costs,
        );
        assert!(html.contains("bob@example.com"));
        assert!(html.contains("75.00 USD"));
        assert!(html.contains("/users/user-1"));
        assert!(html.contains("model-1"));
    }

    #[test]
    fn render_detail_unknown_model_name() {
        let html = render_detail("/", "2024-01-01", "2024-01-31", "model-1", None, &[]);
        assert!(html.contains("unknown"));
    }
}
