use crate::{
    app::App, cmd, cmd::key::read_secret_key, settings::Settings, store::MemoryBlockstore,
};
use anyhow::{Context, anyhow};
use metis_app_options::run::RunArgs;
use metis_chain::tm_abci::abci_app::ApplicationService;

pub use tendermint_rpc::HttpClient;
use tower_abci::v038::{Server, split};

// use metis_vm::interpreter::{
// evm::EvmMessageInterpreter,
// bytes::{BytesMessageInterpreter, ProposalPrepareMode},
// chain::{ChainMessageInterpreter, CheckpointPool},
// signed::SignedMessageInterpreter,
// };

// use std::sync::Arc;
// use tracing::info;

cmd! {
  RunArgs(self, settings : Settings) {
    run(settings).await
  }
}

/// Run the Fendermint ABCI Application.
///
/// This method acts as our composition root.
async fn run(settings: Settings) -> anyhow::Result<()> {
    let _client = HttpClient::new(settings.tendermint_rpc_url()?)
        .context("failed to create Tendermint client")?;

    let _validator_key = {
        let sk = settings.validator_key();
        if sk.exists() && sk.is_file() {
            Some(read_secret_key(&sk).context("failed to read validator key")?)
        } else {
            tracing::debug!("validator key not configured");
            None
        }
    };

    // let _interpreter = EvmMessageInterpreter::<NamespaceBlockstore, _>::new(
    //     client,
    //     // validator_key,
    //     // settings.contracts_dir(),
    //     // settings.fvm.gas_overestimation_rate,
    //     // settings.fvm.gas_search_step,
    //     // settings.fvm.exec_in_check,
    // );

    // todo add SignedMsg/ChainMsg/BytesMsg if we need
    // let interpreter = SignedMessageInterpreter::new(interpreter);
    // let interpreter = ChainMessageInterpreter::new(interpreter);
    // let interpreter =
    //     BytesMessageInterpreter::new(interpreter, ProposalPrepareMode::AppendOnly, false);

    // let ns = Namespaces::default();
    // let _db = open_db(&settings, &ns).context("error opening DB")?;

    // Blockstore for actors.
    let state_store = MemoryBlockstore::default();

    let app: App<MemoryBlockstore> = App::new(
        // AppConfig {
        //     app_namespace: ns.app,
        //     state_hist_namespace: ns.state_hist,
        //     state_hist_size: settings.db.state_hist_size,
        //     builtin_actors_bundle: settings.builtin_actors_bundle(),
        // },
        // db,
        state_store,
        // interpreter,
        // resolve_pool,
        // parent_finality_provider.clone(),
    )?;

    let service = ApplicationService(app);

    // Split it into components.
    let (consensus, mempool, snapshot, info) = split::service(service, settings.abci.bound);

    // Hand those components to the ABCI server. This is where tower layers could be added.
    let server = Server::builder()
        .consensus(consensus)
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info)
        .finish()
        .context("error creating ABCI server")?;

    // Run the ABCI server.
    server
        .listen_tcp(settings.abci.listen.addr())
        .await
        .map_err(|e| anyhow!("error listening: {e}"))?;

    Ok(())
}
//
// namespaces! {
//     Namespaces {
//         app,
//         state_hist,
//         state_store,
//         bit_store
//     }
// }
//
// /// Open database with all
// fn open_db(settings: &Settings, ns: &Namespaces) -> anyhow::Result<RocksDb> {
//     let path = settings.data_dir().join("rocksdb");
//     info!(
//         path = path.to_string_lossy().into_owned(),
//         "opening database"
//     );
//     let db = RocksDb::open_cf(path, &RocksDbConfig::default(), ns.values().iter())?;
//     Ok(db)
// }
