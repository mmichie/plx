//! Pure rendering — no I/O.
//!
//! Output shape:
//!
//! ```text
//! {city}, {country} {icon} {temp}{symbol}
//! ```
//!
//! Each piece is toggleable. The separator between city and icon is a single
//! space; the separator between icon and temperature is also a single space.

use super::Options;
use super::providers::{Condition, WeatherData};

/// Render the final one-line weather string.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn render_line(
    data: &WeatherData,
    city: Option<&str>,
    country: Option<&str>,
    opts: &Options,
) -> String {
    let mut out = String::with_capacity(48);

    if opts.show_city
        && let Some(c) = city.filter(|s| !s.is_empty())
    {
        out.push_str(c);
        if let Some(cc) = country.filter(|s| !s.is_empty()) {
            out.push_str(", ");
            out.push_str(cc);
        }
        out.push(' ');
    }

    if opts.show_icon {
        out.push_str(icon_for(data.condition, opts.use_nerd_font));
        out.push(' ');
    }

    // Round to nearest whole degree. Clamp first so the cast can't overflow
    // i64 on a broken provider response.
    let rounded = data.temp.round().clamp(i64::MIN as f64, i64::MAX as f64) as i64;
    out.push_str(&rounded.to_string());
    out.push_str(temp_symbol(&opts.units));

    // Trim a trailing whitespace defensively — nothing below adds one, but
    // tmux is picky so be paranoid.
    out.trim_end().to_string()
}

/// Temperature unit symbol — F for imperial, C for metric.
#[must_use]
pub fn temp_symbol(units: &str) -> &'static str {
    if units == "imperial" {
        "\u{00B0}F"
    } else {
        "\u{00B0}C"
    }
}

/// Icon glyph for a [`Condition`].
///
/// Plain variants use common emoji / symbols that are present in most fonts.
/// The Nerd Font variants use the `weather-*` block in the Nerd Font Symbols
/// Only range (Private Use Area), which is ubiquitous in terminal fonts like
/// `JetBrainsMono` Nerd Font.
#[must_use]
pub fn icon_for(cond: Condition, use_nerd_font: bool) -> &'static str {
    if use_nerd_font {
        // Nerd Font / Weather Icons glyphs — private use area.
        match cond {
            Condition::Clear => "\u{e30d}",        // nf-weather-day_sunny
            Condition::PartlyCloudy => "\u{e302}", // nf-weather-day_cloudy
            Condition::Cloudy => "\u{e312}",       // nf-weather-cloudy
            Condition::Rainy => "\u{e318}",        // nf-weather-rain
            Condition::Snow => "\u{e31a}",         // nf-weather-snow
            Condition::Storm => "\u{e31d}",        // nf-weather-thunderstorm
            Condition::Fog => "\u{e313}",          // nf-weather-fog
            Condition::Unknown => "\u{e374}",      // nf-weather-na
        }
    } else {
        match cond {
            Condition::Clear => "\u{2600}",        // ☀
            Condition::PartlyCloudy => "\u{26C5}", // ⛅
            Condition::Cloudy => "\u{2601}",       // ☁
            Condition::Rainy => "\u{1F327}",       // 🌧
            Condition::Snow => "\u{2744}",         // ❄
            Condition::Storm => "\u{26C8}",        // ⛈
            Condition::Fog => "\u{1F32B}",         // 🌫
            Condition::Unknown => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(temp: f64, cond: Condition) -> WeatherData {
        WeatherData {
            city: None,
            country: None,
            temp,
            feels_like: None,
            condition: cond,
            description: String::new(),
        }
    }

    #[test]
    fn temp_symbol_metric() {
        assert_eq!(temp_symbol("metric"), "\u{00B0}C");
    }

    #[test]
    fn temp_symbol_imperial() {
        assert_eq!(temp_symbol("imperial"), "\u{00B0}F");
    }

    #[test]
    fn full_output_imperial() {
        let d = data(54.0, Condition::Rainy);
        let opts = Options {
            units: "imperial".into(),
            ..Options::default()
        };
        let s = render_line(&d, Some("Tacoma"), Some("US"), &opts);
        // Should include city, country, icon, and temperature with F.
        assert!(s.starts_with("Tacoma, US "));
        assert!(s.contains("\u{1F327}"));
        assert!(s.ends_with("54\u{00B0}F"));
    }

    #[test]
    fn no_city_no_icon() {
        let d = data(12.4, Condition::Clear);
        let opts = Options {
            show_city: false,
            show_city_was_defaulted: false,
            show_icon: false,
            show_icon_was_defaulted: false,
            ..Options::default()
        };
        let s = render_line(&d, Some("Tacoma"), Some("US"), &opts);
        // Rounds to 12.
        assert_eq!(s, "12\u{00B0}C");
    }

    #[test]
    fn icon_only() {
        let d = data(0.49, Condition::Snow); // rounds to 0
        let opts = Options {
            show_city: false,
            show_city_was_defaulted: false,
            ..Options::default()
        };
        let s = render_line(&d, None, None, &opts);
        assert_eq!(s, "\u{2744} 0\u{00B0}C");
    }

    #[test]
    fn city_no_country() {
        let d = data(20.0, Condition::Clear);
        let opts = Options {
            show_icon: false,
            show_icon_was_defaulted: false,
            ..Options::default()
        };
        let s = render_line(&d, Some("Tacoma"), None, &opts);
        assert_eq!(s, "Tacoma 20\u{00B0}C");
    }

    #[test]
    fn no_city_shows_icon_and_temp_only() {
        // show_city=true but no city data: still no prefix.
        let d = data(20.0, Condition::Clear);
        let opts = Options::default();
        let s = render_line(&d, None, None, &opts);
        assert_eq!(s, "\u{2600} 20\u{00B0}C");
    }

    #[test]
    fn nerd_font_variants_differ() {
        for cond in [
            Condition::Clear,
            Condition::PartlyCloudy,
            Condition::Cloudy,
            Condition::Rainy,
            Condition::Snow,
            Condition::Storm,
            Condition::Fog,
        ] {
            let plain = icon_for(cond, false);
            let nerd = icon_for(cond, true);
            assert_ne!(plain, nerd, "nerd-font icon must differ for {cond:?}");
        }
    }

    #[test]
    fn rounding_is_nearest() {
        let d1 = data(54.49, Condition::Clear);
        let d2 = data(54.5, Condition::Clear);
        let opts = Options {
            show_city: false,
            show_city_was_defaulted: false,
            show_icon: false,
            show_icon_was_defaulted: false,
            ..Options::default()
        };
        assert_eq!(render_line(&d1, None, None, &opts), "54\u{00B0}C");
        assert_eq!(render_line(&d2, None, None, &opts), "55\u{00B0}C");
    }

    #[test]
    fn negative_temperature() {
        let d = data(-5.0, Condition::Snow);
        let opts = Options {
            show_city: false,
            show_city_was_defaulted: false,
            show_icon: false,
            show_icon_was_defaulted: false,
            ..Options::default()
        };
        let s = render_line(&d, None, None, &opts);
        assert_eq!(s, "-5\u{00B0}C");
    }
}
