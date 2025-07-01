use std::marker::PhantomData;

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use tendermint::abci::types::ExecTxResult;
use tendermint_rpc::{Client, HttpClient, Scheme, Url};
use tendermint_rpc::{WebSocketClient, WebSocketClientDriver};

use crate::tx::{
    AsyncResponse, CommitResponse, SignedMessage, SyncResponse, TxAsync, TxClient, TxCommit, TxSync,
};

// Retrieve the proxy URL with precedence:
// 1. If supplied, that's the proxy URL used.
// 2. If not supplied, but environment variable HTTP_PROXY or HTTPS_PROXY are
//    supplied, then use the appropriate variable for the URL in question.
//
// Copied from `tendermint_rpc`.
fn get_http_proxy_url(url_scheme: Scheme, proxy_url: Option<Url>) -> anyhow::Result<Option<Url>> {
    match proxy_url {
        Some(u) => Ok(Some(u)),
        None => match url_scheme {
            Scheme::Http => std::env::var("HTTP_PROXY").ok(),
            Scheme::Https => std::env::var("HTTPS_PROXY")
                .ok()
                .or_else(|| std::env::var("HTTP_PROXY").ok()),
            _ => {
                if std::env::var("HTTP_PROXY").is_ok() || std::env::var("HTTPS_PROXY").is_ok() {
                    tracing::warn!(
                        "Ignoring HTTP proxy environment variables for non-HTTP client connection"
                    );
                }
                None
            }
        }
        .map(|u| u.parse::<Url>().map_err(|e| anyhow!(e)))
        .transpose(),
    }
}

/// Create a Tendermint HTTP client.
pub fn http_client(url: Url, proxy_url: Option<Url>) -> anyhow::Result<HttpClient> {
    let proxy_url = get_http_proxy_url(url.scheme(), proxy_url)?;
    let client = match proxy_url {
        Some(proxy_url) => {
            tracing::debug!(
                "Using HTTP client with proxy {} to submit request to {}",
                proxy_url,
                url
            );
            HttpClient::new_with_proxy(url, proxy_url)?
        }
        None => {
            tracing::debug!("Using HTTP client to submit request to: {}", url);
            HttpClient::new(url)?
        }
    };
    Ok(client)
}

/// Create a Tendermint WebSocket client.
///
/// The caller must start the driver in a background task.
pub async fn ws_client(url: Url) -> anyhow::Result<(WebSocketClient, WebSocketClientDriver)> {
    // TODO: Doesn't handle proxy.
    tracing::debug!("Using WS client to submit request to: {}", url);
    let (client, driver) = WebSocketClient::new(url).await?;
    Ok((client, driver))
}

/// Unauthenticated Tendermint client.
#[derive(Clone, Debug)]
pub struct ClientApi<C = HttpClient> {
    inner: C,
}

impl ClientApi<HttpClient> {
    /// new client with http
    pub fn new_http(url: Url, proxy_url: Option<Url>) -> anyhow::Result<Self> {
        let inner = http_client(url, proxy_url)?;
        Ok(Self { inner })
    }
}

#[async_trait]
impl<C> TxClient<TxAsync> for ClientApi<C>
where
    C: Client + Sync + Send,
{
    async fn perform<F, T>(&self, msg: SignedMessage, _f: F) -> anyhow::Result<AsyncResponse<T>>
    where
        F: FnOnce(&ExecTxResult) -> anyhow::Result<T> + Sync + Send,
    {
        let data = msg.serialize()?;
        let response = self.inner.broadcast_tx_async(data).await?;
        let response = AsyncResponse {
            response,
            return_data: PhantomData,
        };
        Ok(response)
    }
}

#[async_trait]
impl<C> TxClient<TxSync> for ClientApi<C>
where
    C: Client + Sync + Send,
{
    async fn perform<F, T>(
        &self,
        msg: SignedMessage,
        _f: F,
    ) -> anyhow::Result<crate::tx::SyncResponse<T>>
    where
        F: FnOnce(&ExecTxResult) -> anyhow::Result<T> + Sync + Send,
    {
        let data = msg.serialize()?;
        let response = self.inner.broadcast_tx_sync(data).await?;
        let response = SyncResponse {
            response,
            return_data: PhantomData,
        };
        Ok(response)
    }
}

#[async_trait]
impl<C> TxClient<TxCommit> for ClientApi<C>
where
    C: Client + Sync + Send,
{
    async fn perform<F, T>(
        &self,
        msg: SignedMessage,
        f: F,
    ) -> anyhow::Result<crate::tx::CommitResponse<T>>
    where
        F: FnOnce(&ExecTxResult) -> anyhow::Result<T> + Sync + Send,
    {
        let data = msg.serialize()?;
        let response = self.inner.broadcast_tx_commit(data).await?;
        // We have a fully `DeliverTx` with default fields even if `CheckTx` indicates failure.
        let return_data = if response.check_tx.code.is_err() || response.tx_result.code.is_err() {
            None
        } else {
            let return_data =
                f(&response.tx_result).context("error decoding data from deliver_tx in commit")?;
            Some(return_data)
        };
        let response = CommitResponse {
            response,
            return_data,
        };
        Ok(response)
    }
}
