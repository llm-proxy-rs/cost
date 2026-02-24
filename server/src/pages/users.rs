use super::{make_path, paginate, with_period, PAGE_SIZE};
use common::{CostByUser, CostRecord, UserInfo};
use leptos::either::Either;
use leptos::prelude::*;
use templates::{pagination_nav, period_links, Breadcrumb, InfoRow, NavLink, Page, Subpage};

pub fn render_index(
    base: &str,
    period: &str,
    page: usize,
    users: &[UserInfo],
    costs: &[CostByUser],
) -> String {
    let users = users.to_vec();
    let costs = costs.to_vec();
    let empty = users.is_empty() && costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();

    // Build a cost lookup by user_id
    let cost_map: std::collections::HashMap<String, &CostByUser> =
        costs.iter().map(|c| (c.user_id.clone(), c)).collect();

    // Merge users with costs: show all users, lookup cost by user_id
    struct Row {
        user_id: String,
        display: String,
        cost: f64,
        currency: String,
        api_keys: String,
        profiles: i64,
    }

    let mut rows: Vec<Row> = users
        .iter()
        .map(|u| {
            let cost_entry = cost_map.get(&u.user_id);
            Row {
                user_id: u.user_id.clone(),
                display: u.user_email.clone(),
                cost: cost_entry.map(|c| c.amount).unwrap_or(0.0),
                currency: cost_entry
                    .map(|c| c.currency.clone())
                    .unwrap_or_else(|| currency.clone()),
                api_keys: format!("{}/{}", u.active_api_key_count, u.api_key_count),
                profiles: u.inference_profile_count,
            }
        })
        .collect();

    // Also add any cost entries for users not in the enriched list
    let user_ids: std::collections::HashSet<String> =
        users.iter().map(|u| u.user_id.clone()).collect();
    for c in &costs {
        if !user_ids.contains(&c.user_id) {
            rows.push(Row {
                user_id: c.user_id.clone(),
                display: c.user_email.clone().unwrap_or_else(|| c.user_id.clone()),
                cost: c.amount,
                currency: c.currency.clone(),
                api_keys: "-".to_string(),
                profiles: 0,
            });
        }
    }

    let total_rows = rows.len();
    let total_pages = if total_rows == 0 {
        1
    } else {
        total_rows.div_ceil(PAGE_SIZE)
    };
    let page = page.clamp(1, total_pages);
    let skip = (page - 1) * PAGE_SIZE;
    let self_path = with_period(&make_path(base, "/users"), period);
    let pagination_html = pagination_nav(&self_path, page, total_rows, PAGE_SIZE);

    let content = view! {
        <h2>"Users"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No users found."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_user">
                    <tr>
                        <th>"Email"</th>
                        <th>"Cost"</th>
                        <th>"API Keys"</th>
                        <th>"Profiles"</th>
                    </tr>
                    {rows.into_iter().skip(skip).take(PAGE_SIZE).map(|r| {
                        let href = with_period(&make_path(&base_owned, &format!("/users/{}", r.user_id)), period);
                        let cost_str = format!("{:.2} {}", r.cost, r.currency);
                        let profiles_str = r.profiles.to_string();
                        view! {
                            <tr>
                                <td><a href={href}>{r.display}</a></td>
                                <td>{cost_str}</td>
                                <td>{r.api_keys}</td>
                                <td>{profiles_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: "Cost Explorer - Users".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::current("Users"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw("Period", period_links(&make_path(base, "/users"), period)),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_hub(base: &str, period: &str, user: &UserInfo) -> String {
    Page {
        title: format!("Cost Explorer - {}", user.user_email),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link("Users", with_period(&make_path(base, "/users"), period)),
            Breadcrumb::current(&user.user_email),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("User ID", &user.user_id),
            InfoRow::new("Email", &user.user_email),
            InfoRow::new("Created", &user.created_at),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "Daily Cost",
                with_period(
                    &make_path(base, &format!("/users/{}/daily", user.user_id)),
                    period,
                ),
                "-",
            ),
            Subpage::new(
                "Monthly Cost",
                with_period(
                    &make_path(base, &format!("/users/{}/monthly", user.user_id)),
                    period,
                ),
                "-",
            ),
        ],
    }
    .render()
}

pub fn render_daily_costs(
    base: &str,
    period: &str,
    page: usize,
    user_id: &str,
    user_email: &str,
    costs: &[CostRecord],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let (page_items, page) = paginate(&costs, page);
    let self_path = with_period(
        &make_path(base, &format!("/users/{}/daily", user_id)),
        period,
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);
    let base_owned = base.to_string();

    let content = view! {
        <h2>"Daily Cost"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this user in this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="daily_cost">
                    <tr>
                        <th>"Date"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let href = with_period(&make_path(&base_owned, &format!("/costs/daily/{}/users/{}", c.date, user_id)), period);
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        let date = c.date.clone();
                        view! {
                            <tr>
                                <td><a href={href}>{date}</a></td>
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
        title: format!("Cost Explorer - {} - Daily Cost", user_email),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link("Users", with_period(&make_path(base, "/users"), period)),
            Breadcrumb::link(
                user_email,
                with_period(&make_path(base, &format!("/users/{}", user_id)), period),
            ),
            Breadcrumb::current("Daily Cost"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(
                    &make_path(base, &format!("/users/{}/daily", user_id)),
                    period,
                ),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_monthly_costs(
    base: &str,
    period: &str,
    page: usize,
    user_id: &str,
    user_email: &str,
    costs: &[CostRecord],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let (page_items, page) = paginate(&costs, page);
    let self_path = with_period(
        &make_path(base, &format!("/users/{}/monthly", user_id)),
        period,
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);
    let base_owned = base.to_string();

    let content = view! {
        <h2>"Monthly Cost"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this user in this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="monthly_cost">
                    <tr>
                        <th>"Month"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let month = if c.date.len() >= 7 { &c.date[..7] } else { &c.date };
                        let href = with_period(&make_path(&base_owned, &format!("/costs/monthly/{}/users/{}", month, user_id)), period);
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        let month_display = month.to_string();
                        view! {
                            <tr>
                                <td><a href={href}>{month_display}</a></td>
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
        title: format!("Cost Explorer - {} - Monthly Cost", user_email),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link("Users", with_period(&make_path(base, "/users"), period)),
            Breadcrumb::link(
                user_email,
                with_period(&make_path(base, &format!("/users/{}", user_id)), period),
            ),
            Breadcrumb::current("Monthly Cost"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(
                    &make_path(base, &format!("/users/{}/monthly", user_id)),
                    period,
                ),
            ),
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
        let html = render_index("/", "30d", 1, &[], &[]);
        assert!(html.contains("No users found."));
        assert!(html.contains("Cost Explorer - Users"));
    }

    #[test]
    fn render_index_with_data() {
        let users = vec![UserInfo {
            user_id: "abc-123".to_string(),
            user_email: "alice@example.com".to_string(),
            created_at: "2024-01-01".to_string(),
            api_key_count: 3,
            active_api_key_count: 2,
            inference_profile_count: 1,
        }];
        let costs = vec![CostByUser {
            user_id: "abc-123".to_string(),
            user_email: Some("alice@example.com".to_string()),
            amount: 50.0,
            currency: "USD".to_string(),
        }];
        let html = render_index("/", "30d", 1, &users, &costs);
        assert!(html.contains("alice@example.com"));
        assert!(html.contains("50.00 USD"));
        assert!(html.contains("2/3")); // active/total api keys
        assert!(html.contains("/users/abc-123"));
    }

    #[test]
    fn render_index_period_links() {
        let html = render_index("/", "30d", 1, &[], &[]);
        assert!(html.contains("<b>Past 30 Days</b>"));
        assert!(html.contains("?period=7d"));
    }

    #[test]
    fn render_index_custom_base() {
        let users = vec![UserInfo {
            user_id: "abc-123".to_string(),
            user_email: "alice@example.com".to_string(),
            created_at: "2024-01-01".to_string(),
            api_key_count: 1,
            active_api_key_count: 1,
            inference_profile_count: 0,
        }];
        let html = render_index("/_dashboard", "30d", 1, &users, &[]);
        assert!(html.contains("/_dashboard/users/abc-123"));
    }

    #[test]
    fn render_hub_contains_info() {
        let user = UserInfo {
            user_id: "abc-123".to_string(),
            user_email: "alice@example.com".to_string(),
            created_at: "2024-01-01".to_string(),
            api_key_count: 3,
            active_api_key_count: 2,
            inference_profile_count: 5,
        };
        let html = render_hub("/", "30d", &user);
        assert!(html.contains("alice@example.com"));
        assert!(html.contains("abc-123"));
        assert!(html.contains("2024-01-01"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("Monthly Cost"));
    }

    #[test]
    fn render_daily_costs_empty() {
        let html = render_daily_costs("/", "30d", 1, "abc-123", "alice@example.com", &[]);
        assert!(html.contains("No cost data found for this user"));
    }

    #[test]
    fn render_daily_costs_with_data() {
        let costs = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 42.0,
            currency: "USD".to_string(),
        }];
        let html = render_daily_costs("/", "30d", 1, "abc-123", "alice@example.com", &costs);
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("42.00 USD"));
        assert!(html.contains("/costs/daily/2024-01-15/users/abc-123"));
    }

    #[test]
    fn render_monthly_costs_empty() {
        let html = render_monthly_costs("/", "30d", 1, "abc-123", "alice@example.com", &[]);
        assert!(html.contains("No cost data found for this user"));
    }

    #[test]
    fn render_monthly_costs_with_data() {
        let costs = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 500.0,
            currency: "USD".to_string(),
        }];
        let html = render_monthly_costs("/", "30d", 1, "abc-123", "alice@example.com", &costs);
        assert!(html.contains("2024-01"));
        assert!(html.contains("500.00 USD"));
        assert!(html.contains("/costs/monthly/2024-01/users/abc-123"));
    }
}
