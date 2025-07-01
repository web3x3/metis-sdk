use crate::cmd;
use async_trait::async_trait;
use metis_app_options::rpc::{
    BroadcastMode, EvmArgs, RpcArgs, RpcCommands, RpcEvmCommands, RpcQueryCommands, TransArgs,
};
use metis_rpc::tx::{
    AsyncResponse, CommitResponse, GasParams, SignedMessage, SyncResponse, TxAsync, TxClient,
    TxCommit, TxSync,
};
use serde::Serialize;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use tendermint::abci::types::ExecTxResult;
use tendermint::block::Height;
use tendermint_rpc::HttpClient;

use metis_primitives::{Address, U256};
use metis_rpc::client::ClientApi;

cmd! {
  RpcArgs(self) {
    let client = ClientApi::new_http(self.url.clone(), self.proxy_url.clone())?;
    match self.command.clone() {
      RpcCommands::Query { height, command } => {
        let height = Height::try_from(height)?;
        query(client, height, command).await
      },
      RpcCommands::Transfer { args, to } => {
        transfer(client, args, to).await
      },
      RpcCommands::Evm { args:_, command } => match command {
        RpcEvmCommands::Create { contract:_, constructor_args:_ } => {
            // evm_create(client, args, contract, constructor_args).await
            todo!()
        }
        RpcEvmCommands::Invoke { args: EvmArgs { contract:_ }} => {
            // evm_invoke(client, args, contract, method, method_args).await
            todo!()
        }
        RpcEvmCommands::Call { args: EvmArgs { contract:_ }, height} => {
            let _height = Height::try_from(height)?;
            // evm_call(client, args, contract, method, method_args, height).await
            todo!()
        }
        RpcEvmCommands::EstimateGas { args: EvmArgs { contract:_ }, height} => {
            let _height = Height::try_from(height)?;
            // evm_estimate_gas(client, args, contract, method, method_args, height).await
            todo!()
        }
      }
    }
  }
}

// todo put query api into query.rs
// should include all the interface of tendermint-rpc::client::Client
// The implementation of the interface is in rpc/src/client_api.rs
/// Run an ABCI query and print the results on STDOUT.
async fn query(
    _client: ClientApi,
    _height: Height,
    _command: RpcQueryCommands,
) -> anyhow::Result<()> {
    Ok(())
}

/// Create a client, make a call to Tendermint with a closure, then maybe extract some JSON
/// depending on the return value, finally print the result in JSON.
async fn broadcast_and_print<F, T, G>(
    client: ClientApi,
    args: TransArgs,
    f: F,
    g: G,
) -> anyhow::Result<()>
where
    F: FnOnce(
        ClientApi,
        U256,
        GasParams,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<BroadcastResponse<T>>> + Send>>,
    G: FnOnce(T) -> serde_json::Value,
    T: Sync + Send,
{
    let gas_params = gas_params(&args);
    let res = f(client, args.value, gas_params).await?;
    let json = match res {
        BroadcastResponse::Async(res) => json!({"response": res.response}),
        BroadcastResponse::Sync(res) => json!({"response": res.response}),
        BroadcastResponse::Commit(res) => {
            let return_data = res.return_data.map(g).unwrap_or(serde_json::Value::Null);
            json!({"response": res.response, "return_data": return_data})
        }
    };
    print_json(&json)
}

/// Execute token transfer through RPC and print the response to STDOUT as JSON.
async fn transfer(client: ClientApi, args: TransArgs, to: Address) -> anyhow::Result<()> {
    broadcast_and_print(
        client,
        args.clone(),
        |client, value, gas_params| {
            let mut client = TransClient::new(client, &args).unwrap();
            Box::pin(async move { client.transfer(to, value, gas_params).await })
        },
        |_| serde_json::Value::Null,
    )
    .await
}

/// Print out pretty-printed JSON.
///
/// People can use `jq` to turn it into compact form if they want to save the results to a `.jsonline`
/// file, but the default of having human readable output seems more useful.
fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(&value)?;
    println!("{}", json);
    Ok(())
}

#[derive(Debug)]
pub enum BroadcastResponse<T> {
    Async(AsyncResponse<T>),
    Sync(SyncResponse<T>),
    Commit(Box<CommitResponse<T>>),
}

#[allow(dead_code)]
struct BroadcastModeWrapper(BroadcastMode);

impl metis_rpc::tx::BroadcastMode for BroadcastModeWrapper {
    type Response<T> = BroadcastResponse<T>;
}

const fn gas_params(args: &TransArgs) -> GasParams {
    GasParams {
        gas_limit: args.gas_limit,
        gas_fee_cap: args.gas_fee_cap,
        gas_premium: args.gas_premium,
    }
}

struct TransClient {
    inner: ClientApi<HttpClient>,
    broadcast_mode: BroadcastMode,
}

impl TransClient {
    const fn new(client: ClientApi, args: &TransArgs) -> anyhow::Result<Self> {
        let client = Self {
            inner: client,
            broadcast_mode: args.broadcast_mode,
        };
        Ok(client)
    }
}

#[async_trait]
impl TxClient<BroadcastModeWrapper> for TransClient {
    async fn perform<F, T>(&self, msg: SignedMessage, f: F) -> anyhow::Result<BroadcastResponse<T>>
    where
        F: FnOnce(&ExecTxResult) -> anyhow::Result<T> + Sync + Send,
        T: Sync + Send,
    {
        match self.broadcast_mode {
            BroadcastMode::Async => {
                let res = TxClient::<TxAsync>::perform(&self.inner, msg, f).await?;
                Ok(BroadcastResponse::Async(res))
            }
            BroadcastMode::Sync => {
                let res = TxClient::<TxSync>::perform(&self.inner, msg, f).await?;
                Ok(BroadcastResponse::Sync(res))
            }
            BroadcastMode::Commit => {
                let res = TxClient::<TxCommit>::perform(&self.inner, msg, f).await?;
                Ok(BroadcastResponse::Commit(Box::new(res)))
            }
        }
    }
}
