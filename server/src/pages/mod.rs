pub mod home;
pub mod models;
pub mod users;

pub fn make_path(base: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        return base.to_string();
    }
    let base = base.trim_end_matches('/');
    format!("{}{}", base, suffix)
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
}
