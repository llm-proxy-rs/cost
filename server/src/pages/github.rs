use super::{make_path, paginate, with_period, PAGE_SIZE};
use common::{CostByGithub, CostRecord, GithubOrgCost};
use leptos::either::Either;
use leptos::prelude::*;
use templates::{pagination_nav, period_links, Breadcrumb, InfoRow, NavLink, Page, Subpage};

/// GitHub landing hub: total cost + the two tag dimensions as subpages
/// (GithubOrgName -> By Org, GithubRepoName -> By Repo), with their counts.
pub fn render_hub(
    base: &str,
    period: &str,
    total_cost: f64,
    currency: &str,
    org_count: usize,
    repo_count: usize,
    export_row_cap: usize,
) -> String {
    Page {
        title: "Cost Explorer - GitHub".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::current("GitHub"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(&make_path(base, "/costs/github"), period),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "Orgs",
                with_period(&make_path(base, "/costs/github/orgs"), period),
                org_count,
            ),
            Subpage::new(
                "Repos",
                with_period(&make_path(base, "/costs/github/repos"), period),
                repo_count,
            ),
        ],
    }
    .render(export_row_cap)
}

/// By Org: cost grouped by GithubOrgName, each org clickable.
pub fn render_orgs(
    base: &str,
    period: &str,
    page: usize,
    orgs: &[GithubOrgCost],
    sort: Option<usize>,
    order: &str,
    export_row_cap: usize,
) -> String {
    let mut orgs = orgs.to_vec();
    let empty = orgs.is_empty();
    let total: f64 = orgs.iter().map(|o| o.amount).sum();
    let currency = orgs
        .first()
        .map(|o| o.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let base_owned = base.to_string();
    let period_owned = period.to_string();

    if let Some(col) = sort {
        let desc = order == "desc";
        orgs.sort_by(|a, b| {
            let cmp = match col {
                0 => a.org_name.cmp(&b.org_name),
                1 => a
                    .amount
                    .partial_cmp(&b.amount)
                    // f64 has no total order; treat incomparable (NaN) amounts as equal.
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            };
            if desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    let (page_items, page) = paginate(&orgs, page);
    let self_path = with_period(&make_path(base, "/costs/github/orgs"), period);
    let pagination_html = pagination_nav(&self_path, page, orgs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Cost by GitHub Org"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No GitHub cost data found for this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_github_org">
                    <tr>
                        <th>"Org"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|o| {
                        let href = with_period(&make_path(&base_owned, &format!("/costs/github/orgs/{}", o.org_name)), &period_owned);
                        let org = o.org_name.clone();
                        let cost_str = format!("{:.2} {}", o.amount, o.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{org}</a></td>
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
        title: "Cost Explorer - GitHub - Orgs".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::current("Orgs"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(&make_path(base, "/costs/github/orgs"), period),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(export_row_cap)
}

/// By Repo: flat cost grouped by GithubOrgName + GithubRepoName, each repo clickable.
pub fn render_repos(
    base: &str,
    period: &str,
    page: usize,
    costs: &[CostByGithub],
    sort: Option<usize>,
    order: &str,
    export_row_cap: usize,
) -> String {
    let mut costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let base_owned = base.to_string();
    let period_owned = period.to_string();

    if let Some(col) = sort {
        let desc = order == "desc";
        costs.sort_by(|a, b| {
            let cmp = match col {
                0 => a.org_name.cmp(&b.org_name),
                1 => a.repo_name.cmp(&b.repo_name),
                2 => a
                    .amount
                    .partial_cmp(&b.amount)
                    // f64 has no total order; treat incomparable (NaN) amounts as equal.
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            };
            if desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    let (page_items, page) = paginate(&costs, page);
    let self_path = with_period(&make_path(base, "/costs/github/repos"), period);
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Cost by GitHub Repo"</h2>
        {if empty {
            Either::Left(view! {
                <p>"No GitHub cost data found for this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_github_repo">
                    <tr>
                        <th>"Org"</th>
                        <th>"Repo"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let href = with_period(&make_path(&base_owned, &format!("/costs/github/orgs/{}/{}", c.org_name, c.repo_name)), &period_owned);
                        let org = c.org_name.clone();
                        let repo = c.repo_name.clone();
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td>{org}</td>
                                <td><a href={href}>{repo}</a></td>
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
        title: "Cost Explorer - GitHub - Repos".to_string(),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::current("Repos"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(&make_path(base, "/costs/github/repos"), period),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(export_row_cap)
}

/// Repos for one org, each repo clickable.
pub fn render_org(
    base: &str,
    period: &str,
    page: usize,
    org: &str,
    repos: &[CostByGithub],
    sort: Option<usize>,
    order: &str,
    export_row_cap: usize,
) -> String {
    let mut repos = repos.to_vec();
    let empty = repos.is_empty();
    let total: f64 = repos.iter().map(|r| r.amount).sum();
    let currency = repos
        .first()
        .map(|r| r.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let base_owned = base.to_string();
    let period_owned = period.to_string();
    let org_owned = org.to_string();

    if let Some(col) = sort {
        let desc = order == "desc";
        repos.sort_by(|a, b| {
            let cmp = match col {
                0 => a.repo_name.cmp(&b.repo_name),
                1 => a
                    .amount
                    .partial_cmp(&b.amount)
                    // f64 has no total order; treat incomparable (NaN) amounts as equal.
                    .unwrap_or(std::cmp::Ordering::Equal),
                _ => std::cmp::Ordering::Equal,
            };
            if desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    let (page_items, page) = paginate(&repos, page);
    let self_path = with_period(
        &make_path(base, &format!("/costs/github/orgs/{}", org)),
        period,
    );
    let pagination_html = pagination_nav(&self_path, page, repos.len(), PAGE_SIZE);

    let content = view! {
        <h2>"Repos for "{org.to_string()}</h2>
        {if empty {
            Either::Left(view! {
                <p>"No GitHub cost data found for this org."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name="cost_by_github_repo">
                    <tr>
                        <th>"Repo"</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|r| {
                        let href = with_period(&make_path(&base_owned, &format!("/costs/github/orgs/{}/{}", org_owned, r.repo_name)), &period_owned);
                        let repo = r.repo_name.clone();
                        let cost_str = format!("{:.2} {}", r.amount, r.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{repo}</a></td>
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
        title: format!("Cost Explorer - GitHub - {}", org),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                "Orgs",
                with_period(&make_path(base, "/costs/github/orgs"), period),
            ),
            Breadcrumb::current(org),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(
                    &make_path(base, &format!("/costs/github/orgs/{}", org)),
                    period,
                ),
            ),
            InfoRow::new("Org", org),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(export_row_cap)
}

/// Repo hub: info + Daily / Monthly subpages.
pub fn render_repo_hub(
    base: &str,
    period: &str,
    org: &str,
    repo: &str,
    total_cost: f64,
    currency: &str,
    export_row_cap: usize,
) -> String {
    Page {
        title: format!("Cost Explorer - GitHub - {}/{}", org, repo),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                org,
                with_period(
                    &make_path(base, &format!("/costs/github/orgs/{}", org)),
                    period,
                ),
            ),
            Breadcrumb::current(repo),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new("Org", org),
            InfoRow::new("Repo", repo),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "Daily Cost",
                with_period(
                    &make_path(base, &format!("/costs/github/orgs/{}/{}/daily", org, repo)),
                    period,
                ),
                "-",
            ),
            Subpage::new(
                "Monthly Cost",
                with_period(
                    &make_path(
                        base,
                        &format!("/costs/github/orgs/{}/{}/monthly", org, repo),
                    ),
                    period,
                ),
                "-",
            ),
        ],
    }
    .render(export_row_cap)
}

/// Daily (or monthly) cost breakdown for one org/repo. `monthly` selects the label/paths.
fn render_repo_costs(
    base: &str,
    period: &str,
    page: usize,
    org: &str,
    repo: &str,
    costs: &[CostRecord],
    monthly: bool,
    export_row_cap: usize,
) -> String {
    let costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let (page_items, page) = paginate(&costs, page);
    let leaf = if monthly { "monthly" } else { "daily" };
    let heading = if monthly {
        "Monthly Cost"
    } else {
        "Daily Cost"
    };
    let col = if monthly { "Month" } else { "Date" };
    let export = if monthly {
        "github_repo_monthly"
    } else {
        "github_repo_daily"
    };
    let self_path = with_period(
        &make_path(
            base,
            &format!("/costs/github/orgs/{}/{}/{}", org, repo, leaf),
        ),
        period,
    );
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>{heading.to_string()}</h2>
        {if empty {
            Either::Left(view! {
                <p>"No cost data found for this repo in this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name={export}>
                    <tr>
                        <th>{col.to_string()}</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        let label = if monthly && c.date.len() >= 7 { c.date[..7].to_string() } else { c.date.clone() };
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td>{label}</td>
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
        title: format!("Cost Explorer - GitHub - {}/{} - {}", org, repo, heading),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                org,
                with_period(
                    &make_path(base, &format!("/costs/github/orgs/{}", org)),
                    period,
                ),
            ),
            Breadcrumb::link(
                repo,
                with_period(
                    &make_path(base, &format!("/costs/github/orgs/{}/{}", org, repo)),
                    period,
                ),
            ),
            Breadcrumb::current(heading),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(
                    &make_path(
                        base,
                        &format!("/costs/github/orgs/{}/{}/{}", org, repo, leaf),
                    ),
                    period,
                ),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(export_row_cap)
}

pub fn render_repo_daily(
    base: &str,
    period: &str,
    page: usize,
    org: &str,
    repo: &str,
    costs: &[CostRecord],
    export_row_cap: usize,
) -> String {
    render_repo_costs(base, period, page, org, repo, costs, false, export_row_cap)
}

pub fn render_repo_monthly(
    base: &str,
    period: &str,
    page: usize,
    org: &str,
    repo: &str,
    costs: &[CostRecord],
    export_row_cap: usize,
) -> String {
    render_repo_costs(base, period, page, org, repo, costs, true, export_row_cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orgs() -> Vec<GithubOrgCost> {
        vec![
            GithubOrgCost {
                org_name: "acme-corp".to_string(),
                amount: 2387.21,
                currency: "USD".to_string(),
            },
            GithubOrgCost {
                org_name: "acme-labs".to_string(),
                amount: 1134.23,
                currency: "USD".to_string(),
            },
        ]
    }

    fn repos() -> Vec<CostByGithub> {
        vec![
            CostByGithub {
                org_name: "acme-corp".to_string(),
                repo_name: "platform-api".to_string(),
                amount: 1284.57,
                currency: "USD".to_string(),
            },
            CostByGithub {
                org_name: "acme-labs".to_string(),
                repo_name: "ml-training".to_string(),
                amount: 977.33,
                currency: "USD".to_string(),
            },
        ]
    }

    #[test]
    fn render_hub_has_two_dimension_subpages() {
        let html = render_hub(
            "/",
            "30d",
            3521.44,
            "USD",
            2,
            6,
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("<title>Cost Explorer - GitHub</title>"));
        assert!(html.contains(">Orgs</a>"));
        assert!(html.contains(">Repos</a>"));
        assert!(html.contains("/costs/github/orgs"));
        assert!(html.contains("/costs/github/repos"));
        assert!(html.contains("3521.44 USD"));
        // Counts are the org/repo totals, shown in the Count column.
        assert!(html.contains(">2<"));
        assert!(html.contains(">6<"));
    }

    #[test]
    fn render_orgs_lists_and_links() {
        let html = render_orgs(
            "/",
            "30d",
            1,
            &orgs(),
            None,
            "asc",
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("Cost by GitHub Org"));
        assert!(html.contains("acme-corp"));
        assert!(html.contains("2387.21 USD"));
        assert!(html.contains("/costs/github/orgs/acme-corp"));
    }

    #[test]
    fn render_repos_flat_lists_and_links() {
        let html = render_repos(
            "/",
            "30d",
            1,
            &repos(),
            None,
            "asc",
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("Cost by GitHub Repo"));
        assert!(html.contains("acme-corp"));
        assert!(html.contains("platform-api"));
        assert!(html.contains("1284.57 USD"));
        assert!(html.contains("/costs/github/orgs/acme-corp/platform-api"));
    }

    #[test]
    fn render_org_lists_repos() {
        let html = render_org(
            "/",
            "30d",
            1,
            "acme-corp",
            &repos(),
            None,
            "asc",
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("Repos for "));
        assert!(html.contains("platform-api"));
        assert!(html.contains("/costs/github/orgs/acme-corp/platform-api"));
    }

    #[test]
    fn render_repo_hub_has_subpages() {
        let html = render_repo_hub(
            "/",
            "30d",
            "acme-corp",
            "platform-api",
            1284.57,
            "USD",
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("/costs/github/orgs/acme-corp/platform-api/daily"));
        assert!(html.contains("/costs/github/orgs/acme-corp/platform-api/monthly"));
    }

    #[test]
    fn render_repo_daily_table() {
        let costs = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 12.34,
            currency: "USD".to_string(),
        }];
        let html = render_repo_daily(
            "/",
            "30d",
            1,
            "acme-corp",
            "platform-api",
            &costs,
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("2024-01-15"));
        assert!(html.contains("12.34 USD"));
    }

    #[test]
    fn render_repo_monthly_table() {
        let costs = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 69.12,
            currency: "USD".to_string(),
        }];
        let html = render_repo_monthly(
            "/",
            "30d",
            1,
            "acme-corp",
            "platform-api",
            &costs,
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("2024-01"));
        assert!(html.contains("69.12 USD"));
    }

    #[test]
    fn render_uses_custom_base_path() {
        let html = render_hub(
            "/_dashboard",
            "30d",
            0.0,
            "USD",
            0,
            0,
            templates::DEFAULT_EXPORT_ROW_CAP,
        );
        assert!(html.contains("/_dashboard/costs/github/orgs"));
        assert!(html.contains("/_dashboard/costs/github/repos"));
    }
}
