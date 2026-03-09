use crate::color::fg;

#[must_use]
pub fn render_with(job_count: u32, from_bg: Option<u8>) -> (String, Option<u8>) {
    if job_count == 0 {
        return (String::new(), from_bg);
    }

    (format!("{} {job_count}", fg(3)), from_bg)
}

#[cfg(test)]
mod tests {
    use super::render_with;

    #[test]
    fn zero_jobs_is_empty() {
        let (out, _) = render_with(0, Some(236));
        assert!(out.is_empty());
    }

    #[test]
    fn nonzero_jobs() {
        let (out, bg) = render_with(3, Some(236));
        assert!(out.contains("3"), "expected count in: {out}");
        assert_eq!(bg, Some(236));
    }
}
