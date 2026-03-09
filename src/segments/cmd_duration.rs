use crate::color::fg;

#[must_use]
pub fn render_with(duration_ms: u64, from_bg: Option<u8>) -> (String, Option<u8>) {
    if duration_ms <= 2000 {
        return (String::new(), from_bg);
    }

    let total_secs = duration_ms / 1000;
    let formatted = if total_secs >= 3600 {
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        format!("{h}h{m}m")
    } else if total_secs >= 60 {
        let m = total_secs / 60;
        let s = total_secs % 60;
        format!("{m}m{s}s")
    } else {
        format!("{total_secs}s")
    };

    (format!("{} {formatted}", fg(3)), from_bg)
}

#[cfg(test)]
mod tests {
    use super::render_with;

    #[test]
    fn short_duration_is_empty() {
        let (out, _) = render_with(2000, Some(236));
        assert!(out.is_empty());
    }

    #[test]
    fn seconds_format() {
        let (out, bg) = render_with(5000, Some(236));
        assert!(out.contains("5s"), "expected 5s in: {out}");
        assert_eq!(bg, Some(236));
    }

    #[test]
    fn minutes_format() {
        let (out, _) = render_with(90_000, Some(236));
        assert!(out.contains("1m30s"), "expected 1m30s in: {out}");
    }

    #[test]
    fn hours_format() {
        let (out, _) = render_with(3_720_000, Some(236));
        assert!(out.contains("1h2m"), "expected 1h2m in: {out}");
    }
}
