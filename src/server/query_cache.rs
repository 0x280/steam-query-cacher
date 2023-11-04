use std::{sync::Arc, time};

use tokio::sync::RwLock;

use crate::client::{
    packets::{SourceQueryRequest, SourceQueryResponse, a2s_info::A2SInfo, a2s_info_reply::A2SInfoReply},
    SteamQueryClient,
};

pub const DEFAULT_REFRESH_INTERVAL: time::Duration = time::Duration::from_secs(5);

#[derive(Debug)]
pub struct QueryCache<Request: SourceQueryRequest, Response: SourceQueryResponse> {
    val: RwLock<Option<(Response, time::Instant)>>,
    refresh_interval: time::Duration,
    client: Arc<SteamQueryClient>,
    _phantom: std::marker::PhantomData<Request>,
}

impl<Request: SourceQueryRequest, Response: SourceQueryResponse> QueryCache<Request, Response>
where
    for<'a> <Response as TryFrom<&'a [u8]>>::Error: std::fmt::Display,
{
    pub fn new(client: Arc<SteamQueryClient>, refresh_interval: Option<time::Duration>) -> Self {
        Self {
            val: RwLock::new(None),
            refresh_interval: refresh_interval.unwrap_or(DEFAULT_REFRESH_INTERVAL),
            client,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn query_cached(&self) -> Result<Response, std::io::Error> {
        {
            let val = self.val.read().await;
            if let Some((val, expiration)) = val.as_ref() {
                if time::Instant::now() < *expiration {
                    log::info!("Using cached value");
                    return Ok(val.clone());
                }
            }
        }

        let val = self
            .client
            .query::<Request, Response>(Request::new())
            .await?;
        self.val.write().await.replace((val.clone(), time::Instant::now() + self.refresh_interval));

        Ok(val)
    }
}

#[derive(Debug)]
pub struct QueryCacheManager {
    a2s_info: QueryCache<A2SInfo, A2SInfoReply>,
}

impl QueryCacheManager {
    pub fn new(client: Arc<SteamQueryClient>) -> Self {
        Self {
            a2s_info: QueryCache::<A2SInfo, A2SInfoReply>::new(client, None),
        }
    }

    pub async fn a2s_info(&self) -> Result<A2SInfoReply, std::io::Error> {
        self.a2s_info.query_cached().await
    }
}
