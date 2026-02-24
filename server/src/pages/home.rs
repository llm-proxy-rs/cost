use super::make_path;
use templates::{date_range_form, Breadcrumb, InfoRow, Page, Subpage};

pub fn render(
    base: &str,
    start: &str,
    end: &str,
    total_cost: f64,
    currency: &str,
    user_count: usize,
    model_count: usize,
) -> String {
    Page {
        title: "Cost Explorer - Home".to_string(),
        breadcrumbs: vec![Breadcrumb::current("Cost Explorer")],
        info_rows: vec![
            InfoRow::raw(
                "Date Range",
                date_range_form(&make_path(base, ""), start, end),
            ),
            InfoRow::new("Total Cost", &format!("{:.2} {}", total_cost, currency)),
        ],
        subpages: vec![
            Subpage::new("Users", make_path(base, "/users"), user_count),
            Subpage::new("Models", make_path(base, "/models"), model_count),
        ],
        ..Default::default()
    }
    .render()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_title() {
        let html = render("/", "2024-01-01", "2024-01-31", 123.45, "USD", 5, 3);
        assert!(html.contains("<title>Cost Explorer - Home</title>"));
    }

    #[test]
    fn render_contains_date_range() {
        let html = render("/", "2024-01-01", "2024-01-31", 0.0, "USD", 0, 0);
        assert!(html.contains(r#"value="2024-01-01"#));
        assert!(html.contains(r#"value="2024-01-31"#));
        assert!(html.contains(r#"type="date""#));
        assert!(html.contains("Apply"));
    }

    #[test]
    fn render_contains_total_cost() {
        let html = render("/", "2024-01-01", "2024-01-31", 99.99, "USD", 0, 0);
        assert!(html.contains("99.99 USD"));
    }

    #[test]
    fn render_contains_subpage_links() {
        let html = render("/", "2024-01-01", "2024-01-31", 0.0, "USD", 5, 3);
        assert!(html.contains("/users"));
        assert!(html.contains("/models"));
        assert!(html.contains("Users"));
        assert!(html.contains("Models"));
    }

    #[test]
    fn render_contains_counts() {
        let html = render("/", "2024-01-01", "2024-01-31", 0.0, "USD", 12, 7);
        assert!(html.contains("12"));
        assert!(html.contains("7"));
    }

    #[test]
    fn render_uses_custom_base_path() {
        let html = render("/_dashboard", "2024-01-01", "2024-01-31", 0.0, "USD", 1, 1);
        assert!(html.contains("/_dashboard/users"));
        assert!(html.contains("/_dashboard/models"));
    }
}
