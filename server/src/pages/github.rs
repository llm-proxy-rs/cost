use super::{make_path, paginate, with_period, PageContext, TableSort, PAGE_SIZE};
use common::{CostByGithub, CostRecord, GithubOrgCost};
use leptos::either::Either;
use leptos::prelude::*;
use templates::{pagination_nav, period_links, Breadcrumb, InfoRow, NavLink, Page, Subpage};

/// GitHub landing hub: total cost + the two tag dimensions as subpages
/// (GithubOrgName -> By Org, GithubRepoName -> By Repo), plus the GitHub-wide
/// Daily / Monthly cost views, with their counts.
#[allow(clippy::too_many_arguments)]
pub fn render_hub(
    base: &str,
    period: &str,
    total_cost: f64,
    currency: &str,
    daily_count: usize,
    monthly_count: usize,
    org_count: usize,
    repo_count: usize,
    csv_export: templates::CsvExportLimit,
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
                "Daily Cost",
                with_period(&make_path(base, "/costs/github/daily"), period),
                daily_count,
            ),
            Subpage::new(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/github/monthly"), period),
                monthly_count,
            ),
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
    .render(csv_export)
}

/// By Org: cost grouped by GithubOrgName, each org clickable.
pub fn render_orgs(
    base: &str,
    period: &str,
    page: usize,
    orgs: &[GithubOrgCost],
    sort: Option<usize>,
    order: &str,
    csv_export: templates::CsvExportLimit,
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
    .render(csv_export)
}

/// By Repo: flat cost grouped by GithubOrgName + GithubRepoName, each repo clickable.
pub fn render_repos(
    base: &str,
    period: &str,
    page: usize,
    costs: &[CostByGithub],
    sort: Option<usize>,
    order: &str,
    csv_export: templates::CsvExportLimit,
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
    .render(csv_export)
}

/// Repos for one org, each repo clickable.
pub fn render_org(
    ctx: &PageContext<'_>,
    page: usize,
    org: &str,
    repos: &[CostByGithub],
    sort: TableSort<'_>,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
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

    if let Some(col) = sort.column {
        let desc = sort.order == "desc";
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
    .render(ctx.csv_export)
}

/// Repo hub: info + Daily / Monthly subpages.
pub fn render_repo_hub(
    base: &str,
    period: &str,
    org: &str,
    repo: &str,
    total_cost: f64,
    currency: &str,
    csv_export: templates::CsvExportLimit,
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
    .render(csv_export)
}

/// Daily (or monthly) cost breakdown for one org/repo. `monthly` selects the label/paths.
fn render_repo_costs(
    ctx: &PageContext<'_>,
    page: usize,
    org: &str,
    repo: &str,
    costs: &[CostRecord],
    monthly: bool,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
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
    .render(ctx.csv_export)
}

pub fn render_repo_daily(
    ctx: &PageContext<'_>,
    page: usize,
    org: &str,
    repo: &str,
    costs: &[CostRecord],
) -> String {
    render_repo_costs(ctx, page, org, repo, costs, false)
}

pub fn render_repo_monthly(
    ctx: &PageContext<'_>,
    page: usize,
    org: &str,
    repo: &str,
    costs: &[CostRecord],
) -> String {
    render_repo_costs(ctx, page, org, repo, costs, true)
}

// ---------------------------------------------------------------------------
// GitHub-wide Daily / Monthly views — parity with normal/legacy mode. `monthly`
// selects the label, the path leaf ("daily"/"monthly") and the key column.
// Each date/month key drills into a hub presenting By Org / By Repo scoped to
// that single day/month; By Org drills once more into its repos, while By Repo
// is a flat terminal list (a repo on one key is a leaf).
// ---------------------------------------------------------------------------

/// Top-level GitHub Daily/Monthly cost: one row per date (or month), each key
/// clickable into its period hub.
fn render_period_list(
    ctx: &PageContext<'_>,
    page: usize,
    costs: &[CostRecord],
    monthly: bool,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
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
        "github_monthly"
    } else {
        "github_daily"
    };
    let base_owned = base.to_string();
    let leaf_owned = leaf.to_string();
    let self_path = with_period(&make_path(base, &format!("/costs/github/{}", leaf)), period);
    let pagination_html = pagination_nav(&self_path, page, costs.len(), PAGE_SIZE);

    let content = view! {
        <h2>{heading.to_string()}</h2>
        {if empty {
            Either::Left(view! {
                <p>"No GitHub cost data found for this period."</p>
            })
        } else {
            Either::Right(view! {
                <table class="data-table" data-export-name={export}>
                    <tr>
                        <th>{col.to_string()}</th>
                        <th>"Cost"</th>
                    </tr>
                    {page_items.iter().map(|c| {
                        // Monthly rows arrive as "YYYY-MM-01"; show/route on "YYYY-MM".
                        let key = if monthly {
                            c.date.strip_suffix("-01").unwrap_or(&c.date).to_string()
                        } else {
                            c.date.clone()
                        };
                        let href = make_path(&base_owned, &format!("/costs/github/{}/{}", leaf_owned, key));
                        let label = key.clone();
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td><a href={href}>{label}</a></td>
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
        title: format!("Cost Explorer - GitHub - {}", heading),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::current(heading),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::raw(
                "Period",
                period_links(&make_path(base, &format!("/costs/github/{}", leaf)), period),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(ctx.csv_export)
}

pub fn render_daily(ctx: &PageContext<'_>, page: usize, costs: &[CostRecord]) -> String {
    render_period_list(ctx, page, costs, false)
}

pub fn render_monthly(ctx: &PageContext<'_>, page: usize, costs: &[CostRecord]) -> String {
    render_period_list(ctx, page, costs, true)
}

/// Date/month hub scoped to one key: By Org / By Repo subpages.
#[allow(clippy::too_many_arguments)]
pub fn render_period_hub(
    ctx: &PageContext<'_>,
    key: &str,
    monthly: bool,
    total_cost: f64,
    currency: &str,
    org_count: usize,
    repo_count: usize,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
    let leaf = if monthly { "monthly" } else { "daily" };
    let list_label = if monthly {
        "Monthly Cost"
    } else {
        "Daily Cost"
    };
    let key_label = if monthly { "Month" } else { "Date" };
    Page {
        title: format!("Cost Explorer - GitHub - {}", key),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                list_label,
                with_period(&make_path(base, &format!("/costs/github/{}", leaf)), period),
            ),
            Breadcrumb::current(key),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new(key_label, key),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "By Org",
                make_path(base, &format!("/costs/github/{}/{}/orgs", leaf, key)),
                org_count,
            ),
            Subpage::new(
                "By Repo",
                make_path(base, &format!("/costs/github/{}/{}/repos", leaf, key)),
                repo_count,
            ),
        ],
    }
    .render(ctx.csv_export)
}

/// By Org scoped to one key; each org drills into its repos for that key.
pub fn render_period_orgs(
    ctx: &PageContext<'_>,
    page: usize,
    key: &str,
    monthly: bool,
    orgs: &[GithubOrgCost],
    sort: TableSort<'_>,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
    let mut orgs = orgs.to_vec();
    let empty = orgs.is_empty();
    let total: f64 = orgs.iter().map(|o| o.amount).sum();
    let currency = orgs
        .first()
        .map(|o| o.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let leaf = if monthly { "monthly" } else { "daily" };
    let list_label = if monthly {
        "Monthly Cost"
    } else {
        "Daily Cost"
    };
    let key_label = if monthly { "Month" } else { "Date" };
    let base_owned = base.to_string();
    let leaf_owned = leaf.to_string();
    let key_owned = key.to_string();

    if let Some(col) = sort.column {
        let desc = sort.order == "desc";
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
    let self_path = make_path(base, &format!("/costs/github/{}/{}/orgs", leaf, key));
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
                        let href = make_path(&base_owned, &format!("/costs/github/{}/{}/orgs/{}", leaf_owned, key_owned, o.org_name));
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
        title: format!("Cost Explorer - GitHub - {} - By Org", key),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                list_label,
                with_period(&make_path(base, &format!("/costs/github/{}", leaf)), period),
            ),
            Breadcrumb::link(
                key,
                make_path(base, &format!("/costs/github/{}/{}", leaf, key)),
            ),
            Breadcrumb::current("By Org"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new(key_label, key),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(ctx.csv_export)
}

/// Repos for one org scoped to one key (terminal — repo is a leaf).
pub fn render_period_org(
    ctx: &PageContext<'_>,
    page: usize,
    key: &str,
    monthly: bool,
    org: &str,
    repos: &[CostByGithub],
    sort: TableSort<'_>,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
    let mut repos = repos.to_vec();
    let empty = repos.is_empty();
    let total: f64 = repos.iter().map(|r| r.amount).sum();
    let currency = repos
        .first()
        .map(|r| r.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let leaf = if monthly { "monthly" } else { "daily" };
    let list_label = if monthly {
        "Monthly Cost"
    } else {
        "Daily Cost"
    };
    let key_label = if monthly { "Month" } else { "Date" };

    if let Some(col) = sort.column {
        let desc = sort.order == "desc";
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
    let self_path = make_path(
        base,
        &format!("/costs/github/{}/{}/orgs/{}", leaf, key, org),
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
                        let repo = r.repo_name.clone();
                        let cost_str = format!("{:.2} {}", r.amount, r.currency);
                        view! {
                            <tr>
                                <td>{repo}</td>
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
        title: format!("Cost Explorer - GitHub - {} - {}", key, org),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                list_label,
                with_period(&make_path(base, &format!("/costs/github/{}", leaf)), period),
            ),
            Breadcrumb::link(
                key,
                make_path(base, &format!("/costs/github/{}/{}", leaf, key)),
            ),
            Breadcrumb::link(
                "By Org",
                make_path(base, &format!("/costs/github/{}/{}/orgs", leaf, key)),
            ),
            Breadcrumb::current(org),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new(key_label, key),
            InfoRow::new("Org", org),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(ctx.csv_export)
}

/// By Repo flat list (org + repo) scoped to one key (terminal).
pub fn render_period_repos(
    ctx: &PageContext<'_>,
    page: usize,
    key: &str,
    monthly: bool,
    costs: &[CostByGithub],
    sort: TableSort<'_>,
) -> String {
    let base = ctx.base;
    let period = ctx.period;
    let mut costs = costs.to_vec();
    let empty = costs.is_empty();
    let total: f64 = costs.iter().map(|c| c.amount).sum();
    let currency = costs
        .first()
        .map(|c| c.currency.clone())
        .unwrap_or_else(|| "USD".to_string()); // empty result set: no row to read a currency from
    let leaf = if monthly { "monthly" } else { "daily" };
    let list_label = if monthly {
        "Monthly Cost"
    } else {
        "Daily Cost"
    };
    let key_label = if monthly { "Month" } else { "Date" };

    if let Some(col) = sort.column {
        let desc = sort.order == "desc";
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
    let self_path = make_path(base, &format!("/costs/github/{}/{}/repos", leaf, key));
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
                        let org = c.org_name.clone();
                        let repo = c.repo_name.clone();
                        let cost_str = format!("{:.2} {}", c.amount, c.currency);
                        view! {
                            <tr>
                                <td>{org}</td>
                                <td>{repo}</td>
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
        title: format!("Cost Explorer - GitHub - {} - By Repo", key),
        breadcrumbs: vec![
            Breadcrumb::link("Cost Explorer", with_period(&make_path(base, ""), period)),
            Breadcrumb::link(
                "GitHub",
                with_period(&make_path(base, "/costs/github"), period),
            ),
            Breadcrumb::link(
                list_label,
                with_period(&make_path(base, &format!("/costs/github/{}", leaf)), period),
            ),
            Breadcrumb::link(
                key,
                make_path(base, &format!("/costs/github/{}/{}", leaf, key)),
            ),
            Breadcrumb::current("By Repo"),
        ],
        nav_links: vec![NavLink::back()],
        info_rows: vec![
            InfoRow::new(key_label, key),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total, currency)),
        ],
        content,
        subpages: vec![],
    }
    .render(ctx.csv_export)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx(base: &str) -> PageContext<'_> {
        PageContext::new(base, "30d", templates::CsvExportLimit::DEFAULT)
    }

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
            7,
            3,
            2,
            6,
            templates::CsvExportLimit::DEFAULT,
        );
        assert!(html.contains("<title>Cost Explorer - GitHub</title>"));
        assert!(html.contains(">Daily Cost</a>"));
        assert!(html.contains(">Monthly Cost</a>"));
        assert!(html.contains(">Orgs</a>"));
        assert!(html.contains(">Repos</a>"));
        assert!(html.contains("/costs/github/daily"));
        assert!(html.contains("/costs/github/monthly"));
        assert!(html.contains("/costs/github/orgs"));
        assert!(html.contains("/costs/github/repos"));
        assert!(html.contains("3521.44 USD"));
        // Counts are the daily/monthly/org/repo totals, shown in the Count column.
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
            templates::CsvExportLimit::DEFAULT,
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
            templates::CsvExportLimit::DEFAULT,
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
            &test_ctx("/"),
            1,
            "acme-corp",
            &repos(),
            TableSort::new(None, "asc"),
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
            templates::CsvExportLimit::DEFAULT,
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
        let html = render_repo_daily(&test_ctx("/"), 1, "acme-corp", "platform-api", &costs);
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
        let html = render_repo_monthly(&test_ctx("/"), 1, "acme-corp", "platform-api", &costs);
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
            0,
            0,
            templates::CsvExportLimit::DEFAULT,
        );
        assert!(html.contains("/_dashboard/costs/github/daily"));
        assert!(html.contains("/_dashboard/costs/github/monthly"));
        assert!(html.contains("/_dashboard/costs/github/orgs"));
        assert!(html.contains("/_dashboard/costs/github/repos"));
    }

    #[test]
    fn render_daily_links_dates_to_date_hub() {
        let costs = vec![CostRecord {
            date: "2024-01-15".to_string(),
            amount: 12.34,
            currency: "USD".to_string(),
        }];
        let html = render_daily(&test_ctx("/"), 1, &costs);
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("12.34 USD"));
        // The date is a link into its per-date hub.
        assert!(html.contains("/costs/github/daily/2024-01-15"));
    }

    #[test]
    fn render_monthly_links_months_to_month_hub() {
        let costs = vec![CostRecord {
            date: "2024-01-01".to_string(),
            amount: 69.12,
            currency: "USD".to_string(),
        }];
        let html = render_monthly(&test_ctx("/"), 1, &costs);
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("69.12 USD"));
        // Months are shown/routed on "YYYY-MM".
        assert!(html.contains("/costs/github/monthly/2024-01"));
        assert!(!html.contains("2024-01-01"));
    }

    #[test]
    fn render_period_hub_has_org_and_repo_subpages() {
        let html = render_period_hub(&test_ctx("/"), "2024-01-15", false, 99.0, "USD", 2, 6);
        assert!(html.contains(">By Org</a>"));
        assert!(html.contains(">By Repo</a>"));
        assert!(html.contains("/costs/github/daily/2024-01-15/orgs"));
        assert!(html.contains("/costs/github/daily/2024-01-15/repos"));
        assert!(html.contains("99.00 USD"));
    }

    #[test]
    fn render_period_orgs_links_into_date_org() {
        let html = render_period_orgs(
            &test_ctx("/"),
            1,
            "2024-01-15",
            false,
            &orgs(),
            TableSort::new(None, "asc"),
        );
        assert!(html.contains("Cost by GitHub Org"));
        assert!(html.contains("/costs/github/daily/2024-01-15/orgs/acme-corp"));
    }

    #[test]
    fn render_period_org_is_terminal() {
        let html = render_period_org(
            &test_ctx("/"),
            1,
            "2024-01-15",
            false,
            "acme-corp",
            &repos(),
            TableSort::new(None, "asc"),
        );
        assert!(html.contains("Repos for "));
        assert!(html.contains("acme-corp"));
        assert!(html.contains("platform-api"));
        // Terminal: the repo name is not a drill-down link.
        assert!(!html.contains("/orgs/acme-corp/platform-api"));
    }

    #[test]
    fn render_period_repos_is_terminal_flat_list() {
        let html = render_period_repos(
            &test_ctx("/"),
            1,
            "2024-01",
            true,
            &repos(),
            TableSort::new(None, "asc"),
        );
        assert!(html.contains("Cost by GitHub Repo"));
        assert!(html.contains("platform-api"));
        // Monthly key carried through to the self path.
        assert!(html.contains("Month"));
        // Terminal: no repo drill-down links.
        assert!(!html.contains("<a href=\"/costs/github/monthly/2024-01/repos/"));
    }
}
