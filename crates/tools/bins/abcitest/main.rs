use async_stm::{TVar, atomically};
use async_trait::async_trait;
use metis_chain::tm_abci::abci_app::{
    AbciResult, Application, ApplicationService, take_until_max_size,
};
use std::collections::HashMap;
use tendermint::abci::{request, response};
use tendermint::validator;
use tower_abci::v038::{Server, split};

#[derive(Clone)]
pub struct KVStoreApp {
    store: TVar<HashMap<String, Vec<u8>>>,
    height: TVar<u32>,
    app_hash: TVar<[u8; 8]>,
}

impl Default for KVStoreApp {
    fn default() -> Self {
        Self::new()
    }
}

impl KVStoreApp {
    pub fn new() -> Self {
        Self {
            store: TVar::new(std::collections::HashMap::new()),
            height: TVar::new(Default::default()),
            app_hash: TVar::new(Default::default()),
        }
    }
}

#[async_trait]
impl Application for KVStoreApp {
    async fn init_chain(&self, request: request::InitChain) -> AbciResult<response::InitChain> {
        println!("init chain req, height: {}", request.initial_height);
        println!("validator num: {}", request.validators.len());

        let mut validators = Vec::new();
        for validator in request.validators {
            println!(
                "pubkey: {:?}, power: {}",
                validator.pub_key, validator.power
            );

            validators.push(validator::Update {
                pub_key: validator.pub_key,
                power: validator.power,
            });
        }

        Ok(response::InitChain {
            consensus_params: None,
            validators,
            app_hash: Default::default(),
        })
    }

    async fn finalize_block(
        &self,
        request: request::FinalizeBlock,
    ) -> AbciResult<response::FinalizeBlock> {
        println!("test finalize_block, height: {:?}", request.height);

        Ok(response::FinalizeBlock {
            events: vec![],
            tx_results: vec![],
            validator_updates: vec![],
            consensus_param_updates: None,
            app_hash: Default::default(),
        })
    }

    async fn query(&self, request: request::Query) -> AbciResult<response::Query> {
        println!("test query. req: {:?}", request);

        Ok(Default::default())
    }

    async fn info(&self, request: request::Info) -> AbciResult<response::Info> {
        println!("test info. req: {:?}", request);

        let (height, app_hash) = atomically(|| {
            let height = self.height.read_clone()?.into();
            let app_hash = self.app_hash.read()?.to_vec().try_into().unwrap();
            Ok((height, app_hash))
        })
        .await;

        Ok(response::Info {
            data: "kvstore-example".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: height,
            last_block_app_hash: app_hash,
        })
    }

    async fn commit(&self) -> AbciResult<response::Commit> {
        let (retain_height, app_hash) = atomically(|| {
            // As in the other kvstore examples, just use store.len() as the "hash"
            let app_hash = (self.store.read()?.len() as u64).to_be_bytes();
            self.app_hash.replace(app_hash)?;
            let retain_height = self.height.modify(|h| (h + 1, h))?;
            Ok((retain_height.into(), app_hash.to_vec().into()))
        })
        .await;

        println!("test commit. height: {}", retain_height);

        Ok(response::Commit {
            data: app_hash,
            retain_height,
        })
    }

    /// Echo back the same message as provided in the request.
    async fn echo(&self, request: request::Echo) -> AbciResult<response::Echo> {
        println!("test echo");

        Ok(response::Echo {
            message: request.message,
        })
    }

    /// Check the given transaction before putting it into the local mempool.
    async fn check_tx(&self, request: request::CheckTx) -> AbciResult<response::CheckTx> {
        println!("test check_tx. req: {:?}", request);

        Ok(Default::default())
    }

    /// Opportunity for the application to modify the proposed transactions.
    ///
    /// The application must copy the transactions it wants to propose into the response and respect the size restrictions.
    ///
    /// See the [spec](https://github.com/tendermint/tendermint/tree/v0.37.0-rc2/spec/abci#prepareproposal).
    async fn prepare_proposal(
        &self,
        request: request::PrepareProposal,
    ) -> AbciResult<response::PrepareProposal> {
        println!("test prepare_proposal. height: {:?}", request.height);

        let txs = take_until_max_size(request.txs, request.max_tx_bytes.try_into().unwrap());

        Ok(response::PrepareProposal { txs })
    }

    /// Opportunity for the application to inspect the proposal before voting on it.
    ///
    /// The application should accept the proposal unless there's something wrong with it.
    ///
    /// See the [spec](https://github.com/tendermint/tendermint/tree/v0.37.0-rc2/spec/abci#processproposal).
    async fn process_proposal(
        &self,
        request: request::ProcessProposal,
    ) -> AbciResult<response::ProcessProposal> {
        println!("test process_proposal. height: {:?}", request.height);

        Ok(response::ProcessProposal::Accept)
    }

    /// Used during state sync to discover available snapshots on peers.
    async fn list_snapshots(&self) -> AbciResult<response::ListSnapshots> {
        println!("test list_snapshots");

        Ok(Default::default())
    }

    /// Called when bootstrapping the node using state sync.
    async fn offer_snapshot(
        &self,
        request: request::OfferSnapshot,
    ) -> AbciResult<response::OfferSnapshot> {
        println!("test offer_snapshot. req: {:?}", request);

        Ok(Default::default())
    }

    /// Used during state sync to retrieve chunks of snapshots from peers.
    async fn load_snapshot_chunk(
        &self,
        request: request::LoadSnapshotChunk,
    ) -> AbciResult<response::LoadSnapshotChunk> {
        println!("test load_snapshot_chunk. req: {:?}", request);

        Ok(Default::default())
    }

    /// Apply the given snapshot chunk to the application's state.
    async fn apply_snapshot_chunk(
        &self,
        request: request::ApplySnapshotChunk,
    ) -> AbciResult<response::ApplySnapshotChunk> {
        println!("test apply_snapshot_chunk. req: {:?}", request);

        Ok(Default::default())
    }
}

#[tokio::main]
async fn main() {
    let service = ApplicationService(KVStoreApp::new());
    let (consensus, mempool, snapshot, info) = split::service(service, 1);

    println!("Starting ABCI server on port 26658...");
    let server = Server::builder()
        .consensus(consensus)
        .snapshot(snapshot)
        .mempool(mempool)
        .info(info)
        .finish()
        .unwrap();

    server
        .listen_tcp("0.0.0.0:26658")
        .await
        .expect("Server failed to start");
}
