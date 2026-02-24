pub mod costs;
pub mod home;
pub mod models;
pub mod monthly;
pub mod users;

pub const PAGE_SIZE: usize = 50;

pub fn with_period(path: &str, period: &str) -> String {
    if period == "30d" {
        path.to_string()
    } else {
        format!("{}?period={}", path, period)
    }
}

pub fn make_path(base: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return base.to_string();
    }
    let base = base.trim_end_matches('/');
    format!("{}{}", base, suffix)
}

pub fn paginate<T>(items: &[T], page: usize) -> (&[T], usize) {
    let total = items.len();
    if total == 0 {
        return (items, 1);
    }
    let total_pages = total.div_ceil(PAGE_SIZE);
    let page = page.clamp(1, total_pages);
    let start = (page - 1) * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(total);
    (&items[start..end], page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_path_root_base() {
        assert_eq!(make_path("/", ""), "/");
        assert_eq!(make_path("/", "/users"), "/users");
        assert_eq!(make_path("/", "/models/abc"), "/models/abc");
    }

    #[test]
    fn make_path_nested_base() {
        assert_eq!(make_path("/_dashboard", ""), "/_dashboard");
        assert_eq!(make_path("/_dashboard", "/users"), "/_dashboard/users");
        assert_eq!(
            make_path("/_dashboard", "/models/abc"),
            "/_dashboard/models/abc"
        );
    }

    #[test]
    fn make_path_trailing_slash_base() {
        assert_eq!(make_path("/_dashboard/", "/users"), "/_dashboard/users");
    }

    #[test]
    fn with_period_default() {
        assert_eq!(with_period("/users", "30d"), "/users");
    }

    #[test]
    fn with_period_non_default() {
        assert_eq!(with_period("/users", "7d"), "/users?period=7d");
        assert_eq!(with_period("/models", "3m"), "/models?period=3m");
    }
}
