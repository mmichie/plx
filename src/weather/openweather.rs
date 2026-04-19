//! `OpenWeather` provider (api.openweathermap.org).
//!
//! Requires a free API key. Uses the `/data/2.5/weather` current-conditions
//! endpoint. Maps the `weather[0].id` condition code onto [`Condition`].

use std::time::Duration;

use super::providers::{Condition, Provider, WeatherData};

pub struct OpenWeather {
    api_key: String,
}

const HTTP_TIMEOUT: Duration = Duration::from_secs(3);

impl OpenWeather {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl Provider for OpenWeather {
    fn fetch(&self, lat: f64, lon: f64, units: &str) -> Result<WeatherData, String> {
        // OpenWeather uses "metric" and "imperial" literally, so pass through.
        let ow_units = if units == "imperial" {
            "imperial"
        } else {
            "metric"
        };

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather\
             ?lat={lat:.4}&lon={lon:.4}&appid={key}&units={ow_units}",
            key = self.api_key,
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

        let main = body
            .get("main")
            .ok_or_else(|| String::from("missing 'main'"))?;
        let temp = main
            .get("temp")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| String::from("missing main.temp"))?;
        let feels_like = main.get("feels_like").and_then(serde_json::Value::as_f64);

        let first_weather = body
            .get("weather")
            .and_then(serde_json::Value::as_array)
            .and_then(|a| a.first());
        let code = first_weather
            .and_then(|w| w.get("id"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let description = first_weather
            .and_then(|w| w.get("description"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();

        let city = body
            .get("name")
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let country = body
            .get("sys")
            .and_then(|s| s.get("country"))
            .and_then(serde_json::Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);

        Ok(WeatherData {
            city,
            country,
            temp,
            feels_like,
            condition: ow_id_to_condition(code),
            description,
        })
    }
}

/// Map an `OpenWeather` condition `id` to a normalized [`Condition`].
///
/// Reference: <https://openweathermap.org/weather-conditions>. The 800s are
/// sky codes (800 = clear, 801 = few clouds, 802 = scattered, 803/804 = broken
/// / overcast). Everything else uses the 2xx/3xx/5xx/6xx/7xx buckets.
#[must_use]
pub fn ow_id_to_condition(id: u64) -> Condition {
    match id {
        200..=299 => Condition::Storm,
        300..=399 | 500..=599 => Condition::Rainy,
        600..=699 => Condition::Snow,
        700..=799 => Condition::Fog,
        800 => Condition::Clear,
        801 | 802 => Condition::PartlyCloudy,
        803 | 804 => Condition::Cloudy,
        _ => Condition::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ow_storm_ids() {
        assert_eq!(ow_id_to_condition(200), Condition::Storm);
        assert_eq!(ow_id_to_condition(232), Condition::Storm);
    }

    #[test]
    fn ow_rain_ids() {
        assert_eq!(ow_id_to_condition(300), Condition::Rainy);
        assert_eq!(ow_id_to_condition(500), Condition::Rainy);
        assert_eq!(ow_id_to_condition(531), Condition::Rainy);
    }

    #[test]
    fn ow_snow_ids() {
        assert_eq!(ow_id_to_condition(600), Condition::Snow);
        assert_eq!(ow_id_to_condition(622), Condition::Snow);
    }

    #[test]
    fn ow_fog_ids() {
        assert_eq!(ow_id_to_condition(701), Condition::Fog);
        assert_eq!(ow_id_to_condition(781), Condition::Fog);
    }

    #[test]
    fn ow_clear_vs_clouds() {
        assert_eq!(ow_id_to_condition(800), Condition::Clear);
        assert_eq!(ow_id_to_condition(801), Condition::PartlyCloudy);
        assert_eq!(ow_id_to_condition(802), Condition::PartlyCloudy);
        assert_eq!(ow_id_to_condition(803), Condition::Cloudy);
        assert_eq!(ow_id_to_condition(804), Condition::Cloudy);
    }

    #[test]
    fn ow_unknown_id() {
        assert_eq!(ow_id_to_condition(9999), Condition::Unknown);
    }
}
