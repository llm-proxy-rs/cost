use super::{make_path, with_period};
use templates::{period_links, Breadcrumb, InfoRow, Page, Subpage};

#[allow(clippy::too_many_arguments)]
pub fn render(
    base: &str,
    period: &str,
    total_cost: f64,
    currency: &str,
    cost_count: usize,
    monthly_count: usize,
    user_count: usize,
    model_count: usize,
) -> String {
    Page {
        title: "Cost Explorer - Home".to_string(),
        breadcrumbs: vec![Breadcrumb::current("Cost Explorer")],
        nav_links: vec![],
        info_rows: vec![
            InfoRow::raw("Period", period_links(&make_path(base, ""), period)),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        content: (),
        subpages: vec![
            Subpage::new(
                "Daily Cost",
                with_period(&make_path(base, "/costs/daily"), period),
                cost_count,
            ),
            Subpage::new(
                "Monthly Cost",
                with_period(&make_path(base, "/costs/monthly"), period),
                monthly_count,
            ),
            Subpage::new(
                "Users",
                with_period(&make_path(base, "/users"), period),
                user_count,
            ),
            Subpage::new(
                "Models",
                with_period(&make_path(base, "/models"), period),
                model_count,
            ),
        ],
    }
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_title() {
        let html = render("/", "30d", 123.45, "USD", 1, 6, 5, 3);
        assert!(html.contains("<title>Cost Explorer - Home</title>"));
    }

    #[test]
    fn render_contains_period_links() {
        let html = render("/", "30d", 0.0, "USD", 0, 0, 0, 0);
        assert!(html.contains("<b>Past 30 Days</b>"));
        assert!(html.contains("?period=7d"));
    }

    #[test]
    fn render_contains_total_cost() {
        let html = render("/", "30d", 99.99, "USD", 0, 0, 0, 0);
        assert!(html.contains("99.99 USD"));
    }

    #[test]
    fn render_contains_subpage_links() {
        let html = render("/", "30d", 0.0, "USD", 0, 0, 5, 3);
        assert!(html.contains("/costs/daily"));
        assert!(html.contains("/costs/monthly"));
        assert!(html.contains("/users"));
        assert!(html.contains("/models"));
        assert!(html.contains("Daily Cost"));
        assert!(html.contains("Monthly Cost"));
        assert!(html.contains("Users"));
        assert!(html.contains("Models"));
    }

    #[test]
    fn render_contains_counts() {
        let html = render("/", "30d", 0.0, "USD", 2, 6, 12, 7);
        assert!(html.contains("12"));
        assert!(html.contains("7"));
    }

    #[test]
    fn render_uses_custom_base_path() {
        let html = render("/_dashboard", "30d", 0.0, "USD", 0, 0, 1, 1);
        assert!(html.contains("/_dashboard/costs/daily"));
        assert!(html.contains("/_dashboard/costs/monthly"));
        assert!(html.contains("/_dashboard/users"));
        assert!(html.contains("/_dashboard/models"));
    }
}
