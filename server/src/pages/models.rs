use super::{make_path, paginate, with_period, PAGE_SIZE};
use common::{CostByModel, CostRecord, ModelInfo};
use leptos::either::Either;
use leptos::prelude::*;
use templates::{pagination_nav, period_links, Breadcrumb, InfoRow, NavLink, Page, Subpage};

pub fn render_index(
    base: &str,
    period: &str,
    page: usize,
    models: &[ModelInfo],
    costs: &[CostByModel],
) -> String {
    let models = models.to_vec();
    let costs = costs.to_vec();
    let empty = models.is_empty() && costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();

    // Build a cost lookup by model_id
    let cost_map: std::collections::HashMap<String, &CostByModel> =
        costs.iter().map(|c| (c.model_id.clone(), c)).collect();

    struct Row {
        model_id: String,
        display: String,
        cost: f64,
        currency: String,
        status: String,
        protected: bool,
        user_count: i64,
    }

    let mut rows: Vec<Row> = models
        .iter()
        .map(|m| {
            let cost_entry = cost_map.get(&m.model_id);
            Row {
                model_id: m.model_id.clone(),
                display: m.model_name.clone(),
                cost: cost_entry.map(|c| c.amount).unwrap_or(0.0),
                currency: cost_entry
                    .map(|c| c.currency.clone())
                    .unwrap_or_else(|| currency.clone()),
                status: if m.is_disabled {
                    "Disabled".to_string()
                } else {
                    "Active".to_string()
                },
                protected: m.protected,
                user_count: m.user_count,
            }
        })
        .collect();

    // Also add any cost entries for models not in the enriched list
    let model_ids: std::collections::HashSet<String> =
        models.iter().map(|m| m.model_id.clone()).collect();
    for c in &costs {
        if !model_ids.contains(&c.model_id) {
            rows.push(Row {
                model_id: c.model_id.clone(),
                display: c.model_name.clone().unwrap_or_else(|| c.model_id.clone()),
                cost: c.amount,
                currency: c.currency.clone(),
                status: "-".to_string(),
                protected: false,
                user_count: 0,
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
    let self_path = with_period(&make_path(base, "/models"), period);
    let pagination_html = pagination_nav(&self_path, page, total_rows, PAGE_SIZE);

    let content = view! {
        <h2>"Models"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No models found."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_model">
                    <tr>
                        <th>"Name"</th>
                        <th>"Cost"</th>
                        <th>"Status"</th>
                        <th>"Protected"</th>
                        <th>"Users"</th>
                    </tr>
                    {rows.into_iter().skip(skip).take(PAGE_SIZE).map(|r| {
                        let href = with_period(&make_path(&base_owned, &format!("/models/{}", r.model_id)), period);
                        let cost_str = format!("{:.2} {}", r.cost, r.currency);
                        let protected_str = if r.protected { "Yes" } else { "No" };
                        let user_count_str = r.user_count.to_string();
                        view! {
                            <tr>
                                <td><a href={href}>{r.display}</a></td>
                                <td>{cost_str}</td>
                                <td>{r.status}</td>
                                <td>{protected_str}</td>
                                <td>{user_count_str}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </table>
                <div inner_html={pagination_html}></div>
            })
        }}
    };

    Page {
        title: "Cost Explorer - Models".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::current("Models"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw("Period", period_links(&make_path(base, "/models"), period)),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render()
}

pub fn render_hub(base: &str, period: &str, model: &ModelInfo) -> String {
    let status = if model.is_disabled {
        "Disabled"
    } else {
        "Active"
    };
    let protected = if model.protected { "Yes" } else { "No" };

    Page {
        title: format!("Cost Explorer - {}", model.model_name),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link("Models", with_period(&make_path(base, "/models"), period)),
            Breadcrumb::current(&model.model_name),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Model ID", &model.model_id),
            InfoRow::new("Model Name", &model.model_name),
            InfoRow::new("Status", status),
            InfoRow::new("Protected", protected),
            InfoRow::new("Users with Access", &model.user_count.to_string()),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "Daily Cost",
                with_period(
                    &make_path(base, &format!("/models/{}/daily", model.model_id)),
                    period,
                ),
                "-",
            ),
            Subpage::new(
                "Monthly Cost",
                with_period(
                    &make_path(base, &format!("/models/{}/monthly", model.model_id)),
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
    model_id: &str,
    model_name: &str,
    costs: &[CostRecord],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();

    let (page_items, page) = paginate(&costs, page);
    let self_path = with_period(
        &make_path(base, &format!("/models/{}/daily", model_id)),
        period,
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Daily Cost"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this model in this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="daily_cost">
                    <tr>
                        <th>"Date"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let href = with_period(&make_path(&base_owned, &format!("/costs/daily/{}/models/{}", c.date, model_id)), period);
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
        title: format!("Cost Explorer - {} - Daily Cost", model_name),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link("Models", with_period(&make_path(base, "/models"), period)),
            Breadcrumb::link(
                model_name,
                with_period(&make_path(base, &format!("/models/{}", model_id)), period),
            ),
            Breadcrumb::current("Daily Cost"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(
                    &make_path(base, &format!("/models/{}/daily", model_id)),
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
    model_id: &str,
    model_name: &str,
    costs: &[CostRecord],
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string());
    let base_owned = base.to_string();

    let (page_items, page) = paginate(&costs, page);
    let self_path = with_period(
        &make_path(base, &format!("/models/{}/monthly", model_id)),
        period,
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Monthly Cost"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this model in this period."</p>
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
                        let href = with_period(&make_path(&base_owned, &format!("/costs/monthly/{}/models/{}", month, model_id)), period);
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
        title: format!("Cost Explorer - {} - Monthly Cost", model_name),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link("Models", with_period(&make_path(base, "/models"), period)),
            Breadcrumb::link(
                model_name,
                with_period(&make_path(base, &format!("/models/{}", model_id)), period),
            ),
            Breadcrumb::current("Monthly Cost"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(
                    &make_path(base, &format!("/models/{}/monthly", model_id)),
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
        assert!(html.contains("No models found."));
        assert!(html.contains("Cost Explorer - Models"));
    }

    #[test]
    fn render_index_with_data() {
        let models = vec![ModelInfo {
            model_id: "model-1".to_string(),
            model_name: "claude-3".to_string(),
            is_disabled: false,
            protected: true,
            user_count: 5,
        }];
        let costs = vec![CostByModel {
            model_id: "model-1".to_string(),
            model_name: Some("claude-3".to_string()),
            amount: 100.0,
            currency: "USD".to_string(),
        }];
        let html = render_index("/", "30d", 1, &models, &costs);
        assert!(html.contains("claude-3"));
        assert!(html.contains("100.00 USD"));
        assert!(html.contains("Active"));
        assert!(html.contains("Yes")); // protected
        assert!(html.contains("/models/model-1"));
    }

    #[test]
    fn render_index_period_links() {
        let html = render_index("/", "30d", 1, &[], &[]);
        assert!(html.contains("<b>Past 30 Days</b>"));
        assert!(html.contains("?period=7d"));
    }

    #[test]
    fn render_index_custom_base() {
        let models = vec![ModelInfo {
            model_id: "model-1".to_string(),
            model_name: "claude-3".to_string(),
            is_disabled: false,
            protected: false,
            user_count: 1,
        }];
        let html = render_index("/_dashboard", "30d", 1, &models, &[]);
        assert!(html.contains("/_dashboard/models/model-1"));
    }

    #[test]
    fn render_hub_contains_info() {
        let model = ModelInfo {
            model_id: "model-1".to_string(),
            model_name: "claude-3".to_string(),
            is_disabled: false,
            protected: true,
            user_count: 5,
        };
        let html = render_hub("/", "30d", &model);
        assert!(html.contains("claude-3"));
        assert!(html.contains("model-1"));
        assert!(html.contains("Active"));
        assert!(html.contains("Yes")); // protected
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("Monthly Cost"));
    }

    #[test]
    fn render_daily_costs_empty() {
        let html = render_daily_costs("/", "30d", 1, "model-1", "claude-3", &[]);
        assert!(html.contains("No cost data found for this model"));
    }

    #[test]
    fn render_daily_costs_with_data() {
        let costs = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 75.0,
            currency: "USD".to_string(),
        }];
        let html = render_daily_costs("/", "30d", 1, "model-1", "claude-3", &costs);
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("75.00 USD"));
        assert!(html.contains("/costs/daily/2024-01-15/models/model-1"));
    }

    #[test]
    fn render_monthly_costs_empty() {
        let html = render_monthly_costs("/", "30d", 1, "model-1", "claude-3", &[]);
        assert!(html.contains("No cost data found for this model"));
    }

    #[test]
    fn render_monthly_costs_with_data() {
        let costs = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 500.0,
            currency: "USD".to_string(),
        }];
        let html = render_monthly_costs("/", "30d", 1, "model-1", "claude-3", &costs);
        assert!(html.contains("2024-01"));
        assert!(html.contains("500.00 USD"));
        assert!(html.contains("/costs/monthly/2024-01/models/model-1"));
    }
}
