use super::{make_path, paginate, with_period, PAGE_SIZE};
use common::{CostByModel, CostByUser, CostRecord};
use leptos::either::Either;
use leptos::prelude::*;
use templates::{pagination_nav, period_links, Breadcrumb, InfoRow, NavLink, Page, Subpage};

pub fn render(base: &str, period: &str, page: usize, monthly_cost: &[CostRecord]) -> String {
    let monthly_cost = monthly_cost.to_vec();
    let total: f64 = monthly_cost.iter().map(|r| r.amount).sum();
    let currency = monthly_cost
        .first()
        .map(|r| r.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let empty = monthly_cost.is_empty();
    let start = monthly_cost.first().map(|r| r.date.as_str()).unwrap_or("");
    let end = monthly_cost.last().map(|r| r.date.as_str()).unwrap_or("");
    let start_owned = start.to_string();
    let end_owned = end.to_string();
    let base_owned = base.to_string();
    let (page_items, page) = paginate(&monthly_cost, page);
    let self_path = with_period(&make_path(base, "/costs/monthly"), period);
    let pagination_html = pagination_nav(&self_path, page, monthly_cost.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Monthly Cost Breakdown"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="monthly_cost" data-start={start_owned} data-end={end_owned}>
                    <tr>
                        <th>"Month"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|r| {
                        let month = r.date.strip_suffix("-01").unwrap_or(&r.date).to_string();
                        let month_href = make_path(&base_owned, &format!("/costs/monthly/{}", month));
                        let cost_str = format!("{:.2} {}", r.amount, r.currency);
                        let month_display = month.clone();
                        view! {
                            <tr>
                                <td><a href={month_href}>{month_display}</a></td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: "Cost Explorer - Monthly Cost".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::current("Monthly Cost"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(&make_path(base, "/costs/monthly"), period),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_hub(
    base: &str,
    period: &str,
    month: &str,
    total_cost: f64,
    currency: &str,
    user_count: usize,
    model_count: usize,
) -> String {
    Page {
        title: format!("Cost Explorer - {}", month),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/monthly"), period),
            ),
            Breadcrumb::current(month),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Month", month),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "By User",
                make_path(base, &format!("/costs/monthly/{}/users", month)),
                user_count,
            ),
            Subpage::new(
                "By Model",
                make_path(base, &format!("/costs/monthly/{}/models", month)),
                model_count,
            ),
        ],
    }
    .render()
}

pub fn render_users(
    base: &str,
    period: &str,
    page: usize,
    month: &str,
    costs: &[CostByUser],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();
    let month_owned = month.to_string();
    let (page_items, page) = paginate(&costs, page);
    let self_path = make_path(base, &format!("/costs/monthly/{}/users", month));
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Cost by User"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this month."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_user">
                    <tr>
                        <th>"Email"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let display = c.user_email.clone()
                            .unwrap_or_else(|| c.user_id.clone());
                        let href = make_path(&base_owned, &format!("/costs/monthly/{}/users/{}", month_owned, c.user_id));
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{display}</a></td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: format!("Cost Explorer - {} - By User", month),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/monthly"), period),
            ),
            Breadcrumb::link(month, make_path(base, &format!("/costs/monthly/{}", month))),
            Breadcrumb::current("By User"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Month", month),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_models(
    base: &str,
    period: &str,
    page: usize,
    month: &str,
    costs: &[CostByModel],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();
    let month_owned = month.to_string();
    let (page_items, page) = paginate(&costs, page);
    let self_path = make_path(base, &format!("/costs/monthly/{}/models", month));
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Cost by Model"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this month."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_model">
                    <tr>
                        <th>"Model"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let display = c.model_name.clone()
                            .unwrap_or_else(|| c.model_id.clone());
                        let href = make_path(&base_owned, &format!("/costs/monthly/{}/models/{}", month_owned, c.model_id));
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{display}</a></td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: format!("Cost Explorer - {} - By Model", month),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/monthly"), period),
            ),
            Breadcrumb::link(month, make_path(base, &format!("/costs/monthly/{}", month))),
            Breadcrumb::current("By Model"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Month", month),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_user_models(
    base: &str,
    period: &str,
    page: usize,
    month: &str,
    user_email: &str,
    costs: &[CostByModel],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let (page_items, page) = paginate(&costs, page);
    let self_path = make_path(
        base,
        &format!("/costs/monthly/{}/users/{}", month, user_email),
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Models for "{user_email}</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="user_models">
                    <tr>
                        <th>"Model"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let display = c.model_name.clone()
                            .unwrap_or_else(|| c.model_id.clone());
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td>{display}</td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: format!("Cost Explorer - {} - {} - Models", month, user_email),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/monthly"), period),
            ),
            Breadcrumb::link(month, make_path(base, &format!("/costs/monthly/{}", month))),
            Breadcrumb::link(
                "By User",
                make_path(base, &format!("/costs/monthly/{}/users", month)),
            ),
            Breadcrumb::current(user_email),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Month", month),
            InfoRow::new("User", user_email),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_model_users(
    base: &str,
    period: &str,
    page: usize,
    month: &str,
    model_name: &str,
    costs: &[CostByUser],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let (page_items, page) = paginate(&costs, page);
    let self_path = make_path(
        base,
        &format!("/costs/monthly/{}/models/{}", month, model_name),
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Users for "{model_name}</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="model_users">
                    <tr>
                        <th>"Email"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let display = c.user_email.clone()
                            .unwrap_or_else(|| c.user_id.clone());
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td>{display}</td>
                                <td>{cost_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: format!("Cost Explorer - {} - {} - Users", month, model_name),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/monthly"), period),
            ),
            Breadcrumb::link(month, make_path(base, &format!("/costs/monthly/{}", month))),
            Breadcrumb::link(
                "By Model",
                make_path(base, &format!("/costs/monthly/{}/models", month)),
            ),
            Breadcrumb::current(model_name),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Month", month),
            InfoRow::new("Model", model_name),
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
    fn render_contains_title() {
        let monthly = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 820.50,
            currency: "USD".to_string(),
        }];
        let html = render("/", "30d", 1, &monthly);
        assert!(html.contains("<title>Cost Explorer - Monthly Cost</title>"));
    }

    #[test]
    fn render_contains_breadcrumbs() {
        let html = render("/", "30d", 1, &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Monthly Cost"));
    }

    #[test]
    fn render_contains_period_links() {
        let html = render("/", "30d", 1, &[]);
        assert!(html.contains("<b>Past 30 Days</b>"));
        assert!(html.contains("?period=7d"));
    }

    #[test]
    fn render_months_display_without_day() {
        let monthly = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 820.50,
            currency: "USD".to_string(),
        }];
        let html = render("/", "30d", 1, &monthly);
        assert!(html.contains(">2024-01<"));
    }

    #[test]
    fn render_months_are_links() {
        let monthly = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 820.50,
            currency: "USD".to_string(),
        }];
        let html = render("/", "30d", 1, &monthly);
        assert!(html.contains("/costs/monthly/2024-01"));
        assert!(html.contains("<a href=\"/costs/monthly/2024-01\">"));
    }

    #[test]
    fn render_empty_monthly_cost() {
        let html = render("/", "30d", 1, &[]);
        assert!(html.contains("No cost data found for this period."));
    }

    #[test]
    fn render_uses_custom_base_path() {
        let html = render("/_dashboard", "30d", 1, &[]);
        assert!(html.contains("/_dashboard/costs/monthly"));
    }

    #[test]
    fn render_hub_contains_title() {
        let html = render_hub("/", "30d", "2024-01", 820.50, "USD", 3, 2);
        assert!(html.contains("<title>Cost Explorer - 2024-01</title>"));
    }

    #[test]
    fn render_hub_contains_breadcrumbs() {
        let html = render_hub("/", "30d", "2024-01", 820.50, "USD", 3, 2);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("2024-01"));
    }

    #[test]
    fn render_hub_contains_subpage_links() {
        let html = render_hub("/", "30d", "2024-01", 820.50, "USD", 3, 2);
        assert!(html.contains("By User"));
        assert!(html.contains("By Model"));
        assert!(html.contains("/costs/monthly/2024-01/users"));
        assert!(html.contains("/costs/monthly/2024-01/models"));
    }

    #[test]
    fn render_hub_custom_base() {
        let html = render_hub("/_dashboard", "30d", "2024-01", 50.0, "USD", 1, 1);
        assert!(html.contains("/_dashboard/costs/monthly/2024-01/users"));
        assert!(html.contains("/_dashboard/costs/monthly/2024-01/models"));
    }

    #[test]
    fn render_users_empty() {
        let html = render_users("/", "30d", 1, "2024-01", &[]);
        assert!(html.contains("No cost data found for this month."));
    }

    #[test]
    fn render_users_with_data() {
        let costs = vec![CostByUser {
            user_id: "user-1".to_string(),
            user_email: Some("alice@example.com".to_string()),
            amount: 42.0,
            currency: "USD".to_string(),
        }];
        let html = render_users("/", "30d", 1, "2024-01", &costs);
        assert!(html.contains("alice@example.com"));
        assert!(html.contains("42.00 USD"));
        assert!(html.contains("/costs/monthly/2024-01/users/user-1"));
    }

    #[test]
    fn render_users_breadcrumbs() {
        let html = render_users("/", "30d", 1, "2024-01", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("2024-01"));
        assert!(html.contains("By User"));
    }

    #[test]
    fn render_models_empty() {
        let html = render_models("/", "30d", 1, "2024-01", &[]);
        assert!(html.contains("No cost data found for this month."));
    }

    #[test]
    fn render_models_with_data() {
        let costs = vec![CostByModel {
            model_id: "model-1".to_string(),
            model_name: Some("claude-3".to_string()),
            amount: 55.0,
            currency: "USD".to_string(),
        }];
        let html = render_models("/", "30d", 1, "2024-01", &costs);
        assert!(html.contains("claude-3"));
        assert!(html.contains("55.00 USD"));
        assert!(html.contains("/costs/monthly/2024-01/models/model-1"));
    }

    #[test]
    fn render_models_breadcrumbs() {
        let html = render_models("/", "30d", 1, "2024-01", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("2024-01"));
        assert!(html.contains("By Model"));
    }

    #[test]
    fn render_user_models_empty() {
        let html = render_user_models("/", "30d", 1, "2024-01", "alice@example.com", &[]);
        assert!(html.contains("No cost data found."));
    }

    #[test]
    fn render_user_models_with_data() {
        let costs = vec![CostByModel {
            model_id: "model-1".to_string(),
            model_name: Some("claude-3".to_string()),
            amount: 30.0,
            currency: "USD".to_string(),
        }];
        let html = render_user_models("/", "30d", 1, "2024-01", "alice@example.com", &costs);
        assert!(html.contains("claude-3"));
        assert!(html.contains("30.00 USD"));
        // Leaf page: model names are plain text, not links
        assert!(!html.contains("<a href=\"/models/model-1\">"));
    }

    #[test]
    fn render_user_models_breadcrumbs() {
        let html = render_user_models("/", "30d", 1, "2024-01", "alice@example.com", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("2024-01"));
        assert!(html.contains("By User"));
        assert!(html.contains("alice@example.com"));
    }

    #[test]
    fn render_model_users_empty() {
        let html = render_model_users("/", "30d", 1, "2024-01", "claude-3", &[]);
        assert!(html.contains("No cost data found."));
    }

    #[test]
    fn render_model_users_with_data() {
        let costs = vec![CostByUser {
            user_id: "user-1".to_string(),
            user_email: Some("alice@example.com".to_string()),
            amount: 25.0,
            currency: "USD".to_string(),
        }];
        let html = render_model_users("/", "30d", 1, "2024-01", "claude-3", &costs);
        assert!(html.contains("alice@example.com"));
        assert!(html.contains("25.00 USD"));
        // Leaf page: user emails are plain text, not links
        assert!(!html.contains("<a href=\"/users/user-1\">"));
    }

    #[test]
    fn render_model_users_breadcrumbs() {
        let html = render_model_users("/", "30d", 1, "2024-01", "claude-3", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("2024-01"));
        assert!(html.contains("By Model"));
        assert!(html.contains("claude-3"));
    }
}
