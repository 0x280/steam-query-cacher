#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigServer {
    pub name: String,
    pub host: String,
    pub bind: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub servers: Vec<ConfigServer>,
    pub log_level: Option<String>,
}

impl Config {
    pub async fn load(file: String) -> Result<Self, Box<dyn std::error::Error>> {
        let file = tokio::fs::read_to_string(file).await?;
        let config: Config = serde_json::from_str(&file)?;
        Ok(config)
    }
}
