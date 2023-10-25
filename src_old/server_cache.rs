use std::sync::Arc;

use super::{config::ConfigServer, client::{SteamQueryClient, packets}};

use tokio::sync::RwLock;

#[derive(Debug)]
struct Cache<T> {
    last_update: Option<std::time::Instant>,
    data: Option<Arc<T>>,
}

impl<T> Cache<T> {
    fn new() -> Self {
        Self {
            last_update: None,
            data: None,
        }
    }
}

#[derive(Debug)]
pub struct ServerCache {
    pub server: ConfigServer,
    info: RwLock<Cache<packets::a2s_info::A2SInfoReply>>,
    player: RwLock<Cache<packets::a2s_player::A2SPlayerReply>>,
    rules: RwLock<Cache<packets::a2s_rules::A2SRulesReply>>,
}

impl ServerCache {
    pub fn new(server: &ConfigServer) -> Self {
        Self {
            server: server.clone(),
            info: RwLock::new(Cache::new()),
            player: RwLock::new(Cache::new()),
            rules: RwLock::new(Cache::new()),
        }
    }

    pub async fn a2s_info(&self) -> Result<Arc<packets::a2s_info::A2SInfoReply>, Box<dyn std::error::Error + Send + Sync>> {
        let info = self.info.read().await;

        if info.last_update.is_none() || info.last_update.unwrap().elapsed().as_secs() > 5 || info.data.is_none() {
            drop(info);

            let client = SteamQueryClient::new(&self.server.host).await?;
            let server_info = client.a2s_info().await?;

            let mut info = self.info.write().await;
            info.last_update = Some(std::time::Instant::now());
            info.data = Some(Arc::new(server_info));
        }

        let info = self.info.read().await;
        match info.data.as_ref() {
            Some(info) => Ok(info.clone()),
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get server info"))),
        }
    }

    pub async fn a2s_player(&self) -> Result<Arc<packets::a2s_player::A2SPlayerReply>, Box<dyn std::error::Error + Send + Sync>> {
        let player = self.player.read().await;

        if player.last_update.is_none() || player.last_update.unwrap().elapsed().as_secs() > 5 || player.data.is_none() {
            drop(player);

            let client = SteamQueryClient::new(&self.server.host).await?;
            let player_info = client.a2s_player().await?;

            let mut player = self.player.write().await;
            player.last_update = Some(std::time::Instant::now());
            player.data = Some(Arc::new(player_info));
        }

        let player = self.player.read().await;
        match player.data.as_ref() {
            Some(player) => Ok(player.clone()),
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get player info"))),
        }
    }

    pub async fn a2s_rules(&self) -> Result<Arc<packets::a2s_rules::A2SRulesReply>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;

        if rules.last_update.is_none() || rules.last_update.unwrap().elapsed().as_secs() > 60*5 || rules.data.is_none() {
            drop(rules);

            let client = SteamQueryClient::new(&self.server.host).await?;
            let rules_packet = client.a2s_rules().await?;

            let mut rules = self.rules.write().await;
            rules.last_update = Some(std::time::Instant::now());
            rules.data = Some(Arc::new(rules_packet));
        }

        let rules = self.rules.read().await;
        match rules.data.as_ref() {
            Some(rules) => Ok(rules.clone()),
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to get rules info"))),
        }
    }
}
