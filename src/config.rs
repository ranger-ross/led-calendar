use anyhow::bail;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub calendar_ids: Vec<String>,
    pub days_in_advance: u64,
    pub translate_japanese_to_english: bool,
}

impl Config {
    pub fn try_from_env() -> anyhow::Result<Self> {
        let config = std::fs::read_to_string("calendar.toml")?;
        let config: Config = toml::from_str(&config)?;

        if config.calendar_ids.len() == 0 {
            bail!("No calendar-ids were specified")
        }

        Ok(config)
    }
}
