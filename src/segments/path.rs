use std::fmt::Write;

use crate::color::{bg, fg, ARROW, THIN};

#[must_use]
pub fn render(home: &str, pwd: &str) -> String {
    let path = if !home.is_empty() && pwd.starts_with(home) {
        format!("~{}", &pwd[home.len()..])
    } else {
        pwd.to_string()
    };

    let mut parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        parts = vec!["/"];
    }

    let n = parts.len();
    let truncated;
    let parts = if n > 5 {
        truncated = [&["…"][..], &parts[n - 4..]].concat();
        &truncated
    } else {
        &parts
    };
    let n = parts.len();

    let mut out = String::with_capacity(256);

    if n <= 1 {
        let _ = write!(
            out,
            "{}{}{} {}{} {}{}{}",
            fg(238), bg(31), ARROW,
            fg(15), parts.first().unwrap_or(&""),
            fg(31), bg(237), ARROW
        );
    } else {
        let _ = write!(
            out,
            "{}{}{} {}{} {}{}{}",
            fg(238), bg(31), ARROW,
            fg(15), parts[0],
            fg(31), bg(237), ARROW
        );

        let last = parts.len() - 1;
        for (i, part) in parts.iter().enumerate().skip(1) {
            if i > 1 {
                let _ = write!(out, " {}{THIN}", fg(244));
            }
            let color = if i == last { 254 } else { 250 };
            let _ = write!(out, " {}{part}", fg(color));
        }
        let _ = write!(out, " ");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn home_shows_tilde() {
        let out = render("/home/user", "/home/user");
        assert!(out.contains('~'), "expected ~ in: {out}");
        assert!(!out.contains("/home"), "should not contain raw home path");
    }

    #[test]
    fn root_shows_slash() {
        let out = render("/home/user", "/");
        assert!(out.contains('/'), "expected / in: {out}");
    }

    #[test]
    fn deep_truncation() {
        let out = render("", "/a/b/c/d/e/f/g");
        assert!(out.contains('…'), "expected ellipsis in: {out}");
        assert!(out.contains('g'), "expected last component");
    }

    #[test]
    fn five_components_no_truncation() {
        let out = render("", "/a/b/c/d/e");
        assert!(!out.contains('…'), "should not truncate 5 components");
        assert!(out.contains('a'));
        assert!(out.contains('e'));
    }

    #[test]
    fn non_home_no_tilde() {
        let out = render("/home/user", "/var/log");
        assert!(!out.contains('~'), "should not contain ~ for non-home path");
        assert!(out.contains("var"));
        assert!(out.contains("log"));
    }

    #[test]
    fn single_component() {
        let out = render("/home/user", "/tmp");
        assert!(out.contains("tmp"));
    }

    #[test]
    fn home_subdir_shows_tilde() {
        let out = render("/home/user", "/home/user/projects/plx");
        assert!(out.contains('~'), "expected ~ for home subdir");
        assert!(out.contains("plx"));
    }
}
