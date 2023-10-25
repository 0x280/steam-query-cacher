use clap::Parser;
use steam_query_cacher::{Config, server::SteamQueryServer};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = "config.json")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    let config: Config = match Config::load(args.config).await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            return Ok(());
        }
    };

    let default_log_level: String = match config.log_level.clone() {
        Some(log_level) => log_level,
        None => "info".to_string(),
    };

    env_logger::init_from_env(env_logger::Env::new().default_filter_or(default_log_level));

    let server = SteamQueryServer::new(&config).await?;
    server.listen().await?;

    Ok(())
}
