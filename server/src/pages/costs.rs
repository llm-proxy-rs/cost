use super::{make_path, paginate, with_period, PAGE_SIZE};
use common::{CostByModel, CostByUser, CostRecord};
use leptos::either::Either;
use leptos::prelude::*;
use templates::{pagination_nav, period_links, Breadcrumb, InfoRow, NavLink, Page, Subpage};

pub fn render(base: &str, period: &str, page: usize, daily_cost: &[CostRecord]) -> String {
    let daily_cost = daily_cost.to_vec();
    let total: f64 = daily_cost.iter().map(|r| r.amount).sum();
    let currency = daily_cost
        .first()
        .map(|r| r.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let empty = daily_cost.is_empty();
    let start = daily_cost.first().map(|r| r.date.as_str()).unwrap_or("");
    let end = daily_cost.last().map(|r| r.date.as_str()).unwrap_or("");
    let start_owned = start.to_string();
    let end_owned = end.to_string();
    let base_owned = base.to_string();
    let (page_items, page) = paginate(&daily_cost, page);
    let self_path = with_period(&make_path(base, "/costs/daily"), period);
    let pagination_html = pagination_nav(&self_path, page, daily_cost.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Daily Cost Breakdown"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="daily_cost" data-start={start_owned} data-end={end_owned}>
                    <tr>
                        <th>"Date"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|r| {
                        let date_href = make_path(&base_owned, &format!("/costs/daily/{}", r.date));
                        let cost_str = format!("{:.2} {}", r.amount, r.currency);
                        let date = r.date.clone();
                        view! {
                            <tr>
                                <td><a href={date_href}>{date}</a></td>
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
        title: "Cost Explorer - Daily Cost".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::current("Daily Cost"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(&make_path(base, "/costs/daily"), period),
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
    date: &str,
    total_cost: f64,
    currency: &str,
    user_count: usize,
    model_count: usize,
) -> String {
    Page {
        title: format!("Cost Explorer - {}", date),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Daily Cost",
                with_period(&make_path(base, "/costs/daily"), period),
            ),
            Breadcrumb::current(date),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date", date),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "By User",
                make_path(base, &format!("/costs/daily/{}/users", date)),
                user_count,
            ),
            Subpage::new(
                "By Model",
                make_path(base, &format!("/costs/daily/{}/models", date)),
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
    date: &str,
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
    let date_owned = date.to_string();
    let (page_items, page) = paginate(&costs, page);
    let self_path = make_path(base, &format!("/costs/daily/{}/users", date));
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Cost by User"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this date."</p>
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
                        let href = make_path(&base_owned, &format!("/costs/daily/{}/users/{}", date_owned, c.user_id));
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
        title: format!("Cost Explorer - {} - By User", date),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Daily Cost",
                with_period(&make_path(base, "/costs/daily"), period),
            ),
            Breadcrumb::link(date, make_path(base, &format!("/costs/daily/{}", date))),
            Breadcrumb::current("By User"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date", date),
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
    date: &str,
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
    let date_owned = date.to_string();
    let (page_items, page) = paginate(&costs, page);
    let self_path = make_path(base, &format!("/costs/daily/{}/models", date));
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Cost by Model"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this date."</p>
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
                        let href = make_path(&base_owned, &format!("/costs/daily/{}/models/{}", date_owned, c.model_id));
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
        title: format!("Cost Explorer - {} - By Model", date),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Daily Cost",
                with_period(&make_path(base, "/costs/daily"), period),
            ),
            Breadcrumb::link(date, make_path(base, &format!("/costs/daily/{}", date))),
            Breadcrumb::current("By Model"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date", date),
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
    date: &str,
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
    let self_path = make_path(base, &format!("/costs/daily/{}/users/{}", date, user_email));
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
        title: format!("Cost Explorer - {} - {} - Models", date, user_email),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Daily Cost",
                with_period(&make_path(base, "/costs/daily"), period),
            ),
            Breadcrumb::link(date, make_path(base, &format!("/costs/daily/{}", date))),
            Breadcrumb::link(
                "By User",
                make_path(base, &format!("/costs/daily/{}/users", date)),
            ),
            Breadcrumb::current(user_email),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date", date),
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
    date: &str,
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
        &format!("/costs/daily/{}/models/{}", date, model_name),
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
        title: format!("Cost Explorer - {} - {} - Users", date, model_name),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "Daily Cost",
                with_period(&make_path(base, "/costs/daily"), period),
            ),
            Breadcrumb::link(date, make_path(base, &format!("/costs/daily/{}", date))),
            Breadcrumb::link(
                "By Model",
                make_path(base, &format!("/costs/daily/{}/models", date)),
            ),
            Breadcrumb::current(model_name),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Date", date),
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
        let daily = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 123.45,
            currency: "USD".to_string(),
        }];
        let html = render("/", "30d", 1, &daily);
        assert!(html.contains("<title>Cost Explorer - Daily Cost</title>"));
    }

    #[test]
    fn render_contains_breadcrumbs() {
        let html = render("/", "30d", 1, &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Daily Cost"));
    }

    #[test]
    fn render_contains_period_links() {
        let html = render("/", "30d", 1, &[]);
        assert!(html.contains("<b>Past 30 Days</b>"));
        assert!(html.contains("?period=7d"));
    }

    #[test]
    fn render_contains_total_cost() {
        let daily = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 99.99,
            currency: "USD".to_string(),
        }];
        let html = render("/", "30d", 1, &daily);
        assert!(html.contains("99.99 USD"));
    }

    #[test]
    fn render_contains_daily_table() {
        let daily = vec![
            CostRecord {
                date: "2024-01-15".to_string(),
                amount: 50.0,
                currency: "USD".to_string(),
            },
            CostRecord {
                date: "2024-01-16".to_string(),
                amount: 75.0,
                currency: "USD".to_string(),
            },
        ];
        let html = render("/", "30d", 1, &daily);
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("2024-01-16"));
        assert!(html.contains("50.00 USD"));
        assert!(html.contains("75.00 USD"));
    }

    #[test]
    fn render_empty_daily_cost() {
        let html = render("/", "30d", 1, &[]);
        assert!(html.contains("No cost data found for this period."));
    }

    #[test]
    fn render_uses_custom_base_path() {
        let html = render("/_dashboard", "30d", 1, &[]);
        assert!(html.contains("/_dashboard/costs/daily"));
    }

    #[test]
    fn render_dates_are_links() {
        let daily = vec![
            CostRecord {
                date: "2024-01-15".to_string(),
                amount: 50.0,
                currency: "USD".to_string(),
            },
            CostRecord {
                date: "2024-01-16".to_string(),
                amount: 75.0,
                currency: "USD".to_string(),
            },
        ];
        let html = render("/", "30d", 1, &daily);
        assert!(html.contains("/costs/daily/2024-01-15"));
        assert!(html.contains("/costs/daily/2024-01-16"));
        assert!(html.contains("<a href=\"/costs/daily/2024-01-15\">"));
        assert!(html.contains("<a href=\"/costs/daily/2024-01-16\">"));
    }

    #[test]
    fn render_dates_are_links_custom_base() {
        let daily = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 50.0,
            currency: "USD".to_string(),
        }];
        let html = render("/_dashboard", "30d", 1, &daily);
        assert!(html.contains("/_dashboard/costs/daily/2024-01-15"));
    }

    #[test]
    fn render_hub_contains_title() {
        let html = render_hub("/", "30d", "2024-01-15", 123.45, "USD", 3, 2);
        assert!(html.contains("<title>Cost Explorer - 2024-01-15</title>"));
    }

    #[test]
    fn render_hub_contains_breadcrumbs() {
        let html = render_hub("/", "30d", "2024-01-15", 123.45, "USD", 3, 2);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("2024-01-15"));
    }

    #[test]
    fn render_hub_contains_info_rows() {
        let html = render_hub("/", "30d", "2024-01-15", 123.45, "USD", 3, 2);
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("123.45 USD"));
    }

    #[test]
    fn render_hub_contains_subpage_links() {
        let html = render_hub("/", "30d", "2024-01-15", 123.45, "USD", 3, 2);
        assert!(html.contains("By User"));
        assert!(html.contains("By Model"));
        assert!(html.contains("/costs/daily/2024-01-15/users"));
        assert!(html.contains("/costs/daily/2024-01-15/models"));
    }

    #[test]
    fn render_hub_custom_base() {
        let html = render_hub("/_dashboard", "30d", "2024-01-15", 50.0, "USD", 1, 1);
        assert!(html.contains("/_dashboard/costs/daily/2024-01-15/users"));
        assert!(html.contains("/_dashboard/costs/daily/2024-01-15/models"));
    }

    #[test]
    fn render_users_empty() {
        let html = render_users("/", "30d", 1, "2024-01-15", &[]);
        assert!(html.contains("No cost data found for this date."));
    }

    #[test]
    fn render_users_with_data() {
        let costs = vec![CostByUser {
            user_id: "user-1".to_string(),
            user_email: Some("alice@example.com".to_string()),
            amount: 42.0,
            currency: "USD".to_string(),
        }];
        let html = render_users("/", "30d", 1, "2024-01-15", &costs);
        assert!(html.contains("alice@example.com"));
        assert!(html.contains("42.00 USD"));
        assert!(html.contains("/costs/daily/2024-01-15/users/user-1"));
    }

    #[test]
    fn render_users_breadcrumbs() {
        let html = render_users("/", "30d", 1, "2024-01-15", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("By User"));
    }

    #[test]
    fn render_users_links_to_user_drill_down() {
        let costs = vec![CostByUser {
            user_id: "user-1".to_string(),
            user_email: Some("alice@example.com".to_string()),
            amount: 10.0,
            currency: "USD".to_string(),
        }];
        let html = render_users("/", "30d", 1, "2024-01-15", &costs);
        assert!(html.contains("<a href=\"/costs/daily/2024-01-15/users/user-1\">"));
    }

    #[test]
    fn render_models_empty() {
        let html = render_models("/", "30d", 1, "2024-01-15", &[]);
        assert!(html.contains("No cost data found for this date."));
    }

    #[test]
    fn render_models_with_data() {
        let costs = vec![CostByModel {
            model_id: "model-1".to_string(),
            model_name: Some("claude-3".to_string()),
            amount: 55.0,
            currency: "USD".to_string(),
        }];
        let html = render_models("/", "30d", 1, "2024-01-15", &costs);
        assert!(html.contains("claude-3"));
        assert!(html.contains("55.00 USD"));
        assert!(html.contains("/costs/daily/2024-01-15/models/model-1"));
    }

    #[test]
    fn render_models_breadcrumbs() {
        let html = render_models("/", "30d", 1, "2024-01-15", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("By Model"));
    }

    #[test]
    fn render_models_links_to_model_drill_down() {
        let costs = vec![CostByModel {
            model_id: "model-1".to_string(),
            model_name: Some("claude-3".to_string()),
            amount: 10.0,
            currency: "USD".to_string(),
        }];
        let html = render_models("/", "30d", 1, "2024-01-15", &costs);
        assert!(html.contains("<a href=\"/costs/daily/2024-01-15/models/model-1\">"));
    }

    #[test]
    fn render_user_models_empty() {
        let html = render_user_models("/", "30d", 1, "2024-01-15", "alice@example.com", &[]);
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
        let html = render_user_models("/", "30d", 1, "2024-01-15", "alice@example.com", &costs);
        assert!(html.contains("claude-3"));
        assert!(html.contains("30.00 USD"));
        // Leaf page: model names are plain text, not links
        assert!(!html.contains("<a href=\"/models/model-1\">"));
    }

    #[test]
    fn render_user_models_breadcrumbs() {
        let html = render_user_models("/", "30d", 1, "2024-01-15", "alice@example.com", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("By User"));
        assert!(html.contains("alice@example.com"));
    }

    #[test]
    fn render_model_users_empty() {
        let html = render_model_users("/", "30d", 1, "2024-01-15", "claude-3", &[]);
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
        let html = render_model_users("/", "30d", 1, "2024-01-15", "claude-3", &costs);
        assert!(html.contains("alice@example.com"));
        assert!(html.contains("25.00 USD"));
        // Leaf page: user emails are plain text, not links
        assert!(!html.contains("<a href=\"/users/user-1\">"));
    }

    #[test]
    fn render_model_users_breadcrumbs() {
        let html = render_model_users("/", "30d", 1, "2024-01-15", "claude-3", &[]);
        assert!(html.contains("Cost Explorer"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("By Model"));
        assert!(html.contains("claude-3"));
    }
}
