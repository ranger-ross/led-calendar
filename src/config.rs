use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub calendar_ids: Vec<String>,
}

impl Config {
    pub fn try_from_env() -> anyhow::Result<Self> {
        let config = std::fs::read_to_string("calendar.toml")?;
        let config: Config = toml::from_str(&config)?;
        Ok(config)
    }
}
