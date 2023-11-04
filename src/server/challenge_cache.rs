use std::net::SocketAddr;

use crate::{client::packets::SourceChallenge, timed_hashmap::TimedHashMap};

#[derive(Debug)]
pub struct ChallengeCache {
    inner: TimedHashMap<SocketAddr, SourceChallenge>,
}

impl ChallengeCache {
    pub async fn new() -> Self {
        Self {
            inner: TimedHashMap::new(false, None).await,
        }
    }

    fn generate_random_challenge() -> SourceChallenge {
        rand::random::<SourceChallenge>()
    }

    pub async fn get_challenge(&self, addr: &SocketAddr) -> SourceChallenge {
        if let Some(challenge) = self.inner.get(addr).await {
            challenge
        } else {
            let challenge = Self::generate_random_challenge();
            self.inner
                .insert(addr.clone(), challenge, std::time::Duration::from_secs(30))
                .await;
            challenge
        }
    }
}
