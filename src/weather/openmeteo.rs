//! Open-Meteo provider — no API key required.
//!
//! Endpoint: <https://api.open-meteo.com/v1/forecast>. Open-Meteo doesn't
//! return city names, so [`WeatherData::city`] stays `None` here; the caller
//! fills it from the IP geolocation lookup when available.

use std::time::Duration;

use super::providers::{Condition, Provider, WeatherData};

pub struct OpenMeteo;

const HTTP_TIMEOUT: Duration = Duration::from_secs(3);

impl Provider for OpenMeteo {
    fn fetch(&self, lat: f64, lon: f64, units: &str) -> Result<WeatherData, String> {
        let (temp_unit, wind_unit) = if units == "imperial" {
            ("fahrenheit", "mph")
        } else {
            ("celsius", "kmh")
        };

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={lat:.4}&longitude={lon:.4}\
             &current=temperature_2m,apparent_temperature,weather_code\
             &temperature_unit={temp_unit}&wind_speed_unit={wind_unit}"
        );

        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(HTTP_TIMEOUT))
            .build()
            .new_agent();

        let body: serde_json::Value = agent
            .get(&url)
            .call()
            .map_err(|e| format!("http: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("json: {e}"))?;

        let current = body
            .get("current")
            .ok_or_else(|| String::from("missing 'current'"))?;

        let temp = current
            .get("temperature_2m")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| String::from("missing temperature_2m"))?;

        let feels_like = current
            .get("apparent_temperature")
            .and_then(serde_json::Value::as_f64);

        let code = current
            .get("weather_code")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let (condition, description) = wmo_to_condition(code);

        Ok(WeatherData {
            city: None,
            country: None,
            temp,
            feels_like,
            condition,
            description: description.to_string(),
        })
    }
}

/// Map a WMO weather code (the code Open-Meteo returns) to a normalized
/// [`Condition`] and short description.
///
/// Reference: <https://open-meteo.com/en/docs> — section "WMO Weather
/// interpretation codes".
#[must_use]
pub fn wmo_to_condition(code: u64) -> (Condition, &'static str) {
    match code {
        0 => (Condition::Clear, "clear"),
        1 => (Condition::Clear, "mainly clear"),
        2 => (Condition::PartlyCloudy, "partly cloudy"),
        3 => (Condition::Cloudy, "overcast"),
        45 | 48 => (Condition::Fog, "fog"),
        51 | 53 | 55 => (Condition::Rainy, "drizzle"),
        56 | 57 => (Condition::Rainy, "freezing drizzle"),
        61 => (Condition::Rainy, "light rain"),
        63 => (Condition::Rainy, "rain"),
        65 => (Condition::Rainy, "heavy rain"),
        66 | 67 => (Condition::Rainy, "freezing rain"),
        71 => (Condition::Snow, "light snow"),
        73 => (Condition::Snow, "snow"),
        75 => (Condition::Snow, "heavy snow"),
        77 => (Condition::Snow, "snow grains"),
        80..=82 => (Condition::Rainy, "rain showers"),
        85 | 86 => (Condition::Snow, "snow showers"),
        95 => (Condition::Storm, "thunderstorm"),
        96 | 99 => (Condition::Storm, "thunderstorm with hail"),
        _ => (Condition::Unknown, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wmo_clear() {
        let (c, _) = wmo_to_condition(0);
        assert_eq!(c, Condition::Clear);
    }

    #[test]
    fn wmo_rain_codes() {
        for code in [51, 61, 63, 65, 80] {
            let (c, _) = wmo_to_condition(code);
            assert_eq!(c, Condition::Rainy, "code {code} should be Rainy");
        }
    }

    #[test]
    fn wmo_snow_codes() {
        for code in [71, 73, 75, 77, 85] {
            let (c, _) = wmo_to_condition(code);
            assert_eq!(c, Condition::Snow, "code {code} should be Snow");
        }
    }

    #[test]
    fn wmo_storm() {
        let (c, _) = wmo_to_condition(95);
        assert_eq!(c, Condition::Storm);
    }

    #[test]
    fn wmo_fog() {
        let (c, _) = wmo_to_condition(45);
        assert_eq!(c, Condition::Fog);
    }

    #[test]
    fn wmo_unknown_code_is_unknown() {
        let (c, _) = wmo_to_condition(1234);
        assert_eq!(c, Condition::Unknown);
    }
}
