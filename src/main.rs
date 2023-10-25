use clap::Parser;

use steam_query_cacher::{Config, SteamQueryCacheServer};
use tokio::task::JoinSet;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value = "config.json")]
    config: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

    log::info!("Loaded config: {:?}", config);

    // create a tokio join set
    let mut set = JoinSet::new();

    for server in config.servers {
        let server = SteamQueryCacheServer::new(server).await?;

        set.spawn(async move {
            server.listen().await;
        });
    }

    let join_all = async {
        while let Some(result) = set.join_next().await {
            result?;
        }
        Ok::<(), tokio::task::JoinError>(())
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            log::info!("Ctrl-C received, shutting down");
        }
        _ = join_all => {
            log::info!("All servers shut down");
        }
    };

    Ok(())
}
