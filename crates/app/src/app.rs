use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

use metis_chain::tm_abci::abci_app::{AbciResult, Application};
// use metis_storage::{Codec, Encode, KVReadable, KVStore, KVWritable};
use metis_primitives::B256;
use metis_storage::KVStore;
use tendermint::abci::request::FinalizeBlock;
use tendermint::abci::{request, response};

#[derive(Serialize)]
#[repr(u8)]
#[allow(dead_code, unreachable_pub)]
pub enum AppStoreKey {
    State,
}

#[derive(Debug)]
#[repr(u32)]
enum _AppError {
    /// Failed to deserialize the transaction.
    InvalidEncoding = 51,
    /// Failed to validate the user signature.
    InvalidSignature = 52,
    /// User sent a message they should not construct.
    IllegalMessage = 53,
    /// The genesis block hasn't been initialized yet.
    NotInitialized = 54,
}

/// The application state record we keep a history of in the database.
#[allow(unreachable_pub, dead_code)]
pub struct AppState {
    /// Last committed block height.
    block_height: u64,
    /// Oldest state hash height.
    oldest_state_height: u64,
}

impl AppState {
    // TODO: calculate the state root hash from the reth state provider.
    fn app_hash(&self) -> tendermint::hash::AppHash {
        tendermint::hash::AppHash::try_from(B256::ZERO.to_vec()).expect("hash can be wrapped")
    }
}

#[allow(dead_code, unreachable_pub)]
pub struct AppConfig<S: KVStore> {
    /// Namespace to store the current app state.
    pub app_namespace: S::Namespace,
    /// Namespace to store the app state history.
    pub state_hist_namespace: S::Namespace,
    /// Size of state history to keep; 0 means unlimited.
    pub state_hist_size: u64,
    /// Path to the Wasm bundle.
    ///
    /// Only loaded once during genesis; later comes from the [`StateTree`].
    pub builtin_actors_bundle: PathBuf,
}

///// The type alias for the actual parent finality provider
// type ParentFinalityProvider = Toggle<CachedFinalityProvider>;

/// Handle ABCI requests.
#[derive(Clone)]
#[allow(dead_code, unreachable_pub)]
pub struct App {
    // TODO: use the reth state provider
    app_state: Arc<AppState>,
    // /// Namespace to store app state.
    // namespace: S::Namespace,
    // /// Collection of past state parameters.
    // ///
    // /// We store the state hash for the height of the block where it was committed,
    // /// which is different from how Tendermint Core will refer to it in queries,
    // /// shifted by one, because Tendermint Core will use the height where the hash
    // /// *appeared*, which is in the block *after* the one which was committed.
    // ///
    // /// The state also contains things like timestamp and the network version,
    // /// so that we can retrospectively execute EVM messages at past block heights
    // /// in read-only mode.
    // state_hist: KVCollection<S, BlockHeight, EvmStateParams>,
    // /// Interpreter for block lifecycle events.
    // interpreter: Arc<I>,
    // /// CID resolution pool.
    // resolve_pool: CheckpointPool,
    // /// The parent finality provider for top down checkpoint
    // parent_finality_provider: Arc<ParentFinalityProvider>,
    // /// State accumulating changes during block execution.
    // exec_state: Arc<Mutex<Option<EvmExecState<SS>>>>,
    // /// Projected (partial) state accumulating during transaction checks.
    // check_state: CheckStateRef<SS>,
    // /// How much history to keep.
    // // /// Zero means unlimited.
    // state_hist_size: u64,
}

#[allow(dead_code, unreachable_pub)]
impl App {
    pub fn new(// config: AppConfig<S>,
        // db: DB,
        // state_store: SS,
        // interpreter: I,
        // resolve_pool: CheckpointPool,
        // parent_finality_provider: Arc<ParentFinalityProvider>,
    ) -> Result<Self> {
        let app = Self {
            app_state: Arc::new(AppState {
                block_height: 0,
                oldest_state_height: 0,
            }), // multi_engine: Arc::new(MultiEngine::new(1)),
                // builtin_actors_bundle: config.builtin_actors_bundle,
                // namespace: config.app_namespace,
                // state_hist: KVCollection::new(config.state_hist_namespace),
                // state_hist_size: config.state_hist_size,
                // interpreter: Arc::new(interpreter),
                // resolve_pool,
                // parent_finality_provider,
                // exec_state: Arc::new(Mutex::new(None)),
                // check_state: Arc::new(tokio::sync::Mutex::new(None)),
        };
        // app.init_committed_state()?;
        Ok(app)
    }
}

// todo add exec state dependencies
/*
impl<DB, SS, S, I> App<DB, SS, S, I>
where
    S: KVStore
    + Codec<AppState>
    + Encode<AppStoreKey>
    + Encode<BlockHeight>
    + Codec<EvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + 'static + Clone,
{
    /// Get an owned clone of the state store.
    fn state_store_clone(&self) -> SS {
        self.state_store.as_ref().clone()
    }

    /// Ensure the store has some initial state.
    fn init_committed_state(&self) -> Result<()> {
        if self.get_committed_state()?.is_none() {
            // We need to be careful never to run a query on this.
            let mut state_tree = empty_state_tree(self.state_store_clone())
                .context("failed to create empty state tree")?;
            let state_root = state_tree.flush()?;
            let state = AppState {
                block_height: 0,
                oldest_state_height: 0,
                state_params: EvmStateParams {
                    timestamp: Timestamp(0),
                    state_root,
                    network_version: NetworkVersion::MAX,
                    base_fee: TokenAmount::zero(),
                    circ_supply: TokenAmount::zero(),
                    chain_id: 0,
                },
            };
            self.set_committed_state(state)?;
        }
        Ok(())
    }

    /// Get the last committed state, if exists.
    fn get_committed_state(&self) -> Result<Option<AppState>> {
        let tx = self.db.read();
        tx.get(&self.namespace, &AppStoreKey::State)
            .context("get failed")
    }

    /// Get the last committed state; return error if it doesn't exist.
    fn committed_state(&self) -> Result<AppState> {
        match self.get_committed_state()? {
            Some(state) => Ok(state),
            None => Err(anyhow!("app state not found")),
        }
    }

    /// Set the last committed state.
    fn set_committed_state(&self, mut state: AppState) -> Result<()> {
        self.db
            .with_write(|tx| {
                // Insert latest state history point.
                self.state_hist
                    .put(tx, &state.block_height, &state.state_params)?;

                // Prune state history.
                if self.state_hist_size > 0 && state.block_height >= self.state_hist_size {
                    let prune_height = state.block_height.saturating_sub(self.state_hist_size);
                    while state.oldest_state_height <= prune_height {
                        self.state_hist.delete(tx, &state.oldest_state_height)?;
                        state.oldest_state_height += 1;
                    }
                }

                // Update the application state.
                tx.put(&self.namespace, &AppStoreKey::State, &state)?;

                Ok(())
            })
            .context("commit failed")
    }

    /// Put the execution state during block execution. Has to be empty.
    fn put_exec_state(&self, state: EvmExecState<SS>) {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        assert!(guard.is_none(), "exec state not empty");
        *guard = Some(state);
    }

    /// Take the execution state during block execution. Has to be non-empty.
    fn take_exec_state(&self) -> EvmExecState<SS> {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        guard.take().expect("exec state empty")
    }

    /// Take the execution state, update it, put it back, return the output.
    async fn modify_exec_state<T, F, R>(&self, f: F) -> Result<T>
    where
        F: FnOnce(
            (
                CheckpointPool,
                Arc<ParentFinalityProvider>,
                EvmExecState<SS>,
            ),
        ) -> R,
        R: Future<
            Output = Result<(
                (
                    CheckpointPool,
                    Arc<ParentFinalityProvider>,
                    EvmExecState<SS>,
                ),
                T,
            )>,
        >,
    {
        let state = self.take_exec_state();
        let ((_pool, _provider, state), ret) = f((
            self.resolve_pool.clone(),
            self.parent_finality_provider.clone(),
            state,
        ))
            .await?;
        self.put_exec_state(state);
        Ok(ret)
    }

    /// Get a read only Evm execution state. This is useful to perform query commands targeting
    /// the latest state.
    pub fn new_read_only_exec_state(
        &self,
    ) -> Result<Option<EvmExecState<ReadOnlyBlockstore<Arc<SS>>>>> {
        let maybe_app_state = self.get_committed_state()?;

        Ok(if let Some(app_state) = maybe_app_state {
            let block_height = app_state.block_height;
            let state_params = app_state.state_params;

            // wait for block production
            if block_height == 0 {
                return Ok(None);
            }

            let exec_state = EvmExecState::new(
                ReadOnlyBlockstore::new(self.state_store.clone()),
                self.multi_engine.as_ref(),
                block_height as ChainEpoch,
                state_params,
            )
                .context("error creating execution state")?;

            Some(exec_state)
        } else {
            None
        })
    }

    /// Look up a past state at a particular height Tendermint Core is looking for.
    ///
    /// A height of zero means we are looking for the latest state.
    /// The genesis block state is saved under height 1.
    /// Under height 0 we saved the empty state, which we must not query,
    /// because it doesn't contain any initialized state for the actors.
    ///
    /// Returns the state params and the height of the block which committed it.
    fn state_params_at_height(
        &self,
        height: EvmQueryHeight,
    ) -> Result<(EvmStateParams, BlockHeight)> {
        if let EvmQueryHeight::Height(h) = height {
            let tx = self.db.read();
            let sh = self
                .state_hist
                .get(&tx, &h)
                .context("error looking up history")?;

            if let Some(p) = sh {
                return Ok((p, h));
            }
        }
        let state = self.committed_state()?;
        Ok((state.state_params, state.block_height))
    }
}
*/

// NOTE: The `Application` interface doesn't allow failures at the moment. The protobuf
// of `Response` actually has an `Exception` type, so in theory we could use that, and
// Tendermint would break up the connection. However, before the response could reach it,
// the `tower-abci` library would throw an exception when it tried to convert a
// `Response::Exception` into a `ConsensusResponse` for example.
#[async_trait]
// impl<DB, SS, S, I> Application for App<DB, SS, S, I>
impl Application for App {
    /// Provide information about the ABCI application.
    async fn info(&self, _request: request::Info) -> AbciResult<response::Info> {
        let height = tendermint::block::Height::try_from(self.app_state.block_height)?;
        let app_hash = self.app_state.app_hash();

        let info = response::Info {
            data: "metis".to_string(),
            version: "metis-v1".to_string(),
            app_version: 1,
            last_block_height: height,
            last_block_app_hash: app_hash,
        };
        Ok(info)
    }

    /// Called once upon genesis.
    async fn init_chain(&self, request: request::InitChain) -> AbciResult<response::InitChain> {
        // let state =
        //     EvmGenesisState::new(self.state_store_clone())
        //         .await
        //         .context("failed to create genesis state")?;

        // let (state, out) = self
        //     .interpreter
        //     .init(state, request.app_state_bytes.to_vec())
        //     .await
        //     .context("failed to init from genesis")?;

        // todo cal state_root hash with genesis
        // let state_root = state.commit().context("failed to commit genesis state")?;
        let height = request.initial_height.into();
        // let validators =
        //     to_validator_updates(out.validators).context("failed to convert validators")?;

        let app_state = AppState {
            block_height: height,
            oldest_state_height: height,
        };

        let response = response::InitChain {
            consensus_params: None,
            validators: vec![],
            app_hash: app_state.app_hash(),
        };

        tracing::info!(app_hash = format!("{}", app_state.app_hash()), "init chain");

        // todo set state
        // self.set_committed_state(app_state)?;

        Ok(response)
    }

    /// Query the application for data at the current or past height.
    /// // todo now only latest state
    async fn query(&self, _request: request::Query) -> AbciResult<response::Query> {
        Ok(response::Query {
            code: Default::default(),
            log: "".to_string(),
            info: "".to_string(),
            index: 0,
            key: Default::default(),
            value: Default::default(),
            proof: None,
            height: Default::default(),
            codespace: "".to_string(),
        })
    }

    /// Check the given transaction before putting it into the local mempool.
    async fn check_tx(&self, request: request::CheckTx) -> AbciResult<response::CheckTx> {
        /*
        // Keep the guard through the check, so there can be only one at a time.
        let mut guard = self.check_state.lock().await;

        let state = match guard.take() {
            Some(state) => state,
            None => {
                // let db = self.state_store_clone();
                // let state = self.committed_state()?;

                // This would create a partial state, but some client scenarios need the full one.
                // EvmCheckState::new(db, state.state_root(), state.chain_id())
                //     .context("error creating check state")?

                EvmExecState::new(
                    // ReadOnlyBlockstore::new(db),
                    // self.multi_engine.as_ref(),
                    // state.block_height.try_into()?,
                    // state.state_params,
                )
                    .context("error creating check state")?
            }
        };

        let (state, result) = self
            .interpreter
            .check(
                state,
                request.tx.to_vec(),
                request.kind == request::CheckTxKind::Recheck,
            )
            .await
            .context("error running check")?;

        // Update the check state.
        *guard = Some(state);

        let response = match result {
            Err(e) => invalid_check_tx(AppError::InvalidEncoding, e.description),
            Ok(result) => match result {
                Err(IllegalMessage) => invalid_check_tx(AppError::IllegalMessage, "".to_owned()),
                Ok(Err(InvalidSignature(d))) => invalid_check_tx(AppError::InvalidSignature, d),
                Ok(Ok(ret)) => to_check_tx(ret),
            },
        };
        */
        let _tx_bytes = request.tx.to_vec();
        // todo use evm Deserialize tx_bytes to evm_tx
        // let tx = Deserialize(tx_bytes)
        // check validate tx

        Ok(response::CheckTx {
            code: Default::default(),
            data: Default::default(),
            log: "".to_string(),
            info: "".to_string(),
            gas_wanted: 0,
            gas_used: 0,
            events: vec![],
            codespace: "".to_string(),
            sender: "".to_string(),
            priority: 0,
            mempool_error: "".to_string(),
        })
    }

    /// Amend which transactions to put into the next block proposal.
    async fn prepare_proposal(
        &self,
        request: request::PrepareProposal,
    ) -> AbciResult<response::PrepareProposal> {
        let txs = request.txs;
        /*
                let txs = request.txs.into_iter().map(|tx| tx.to_vec()).collect();

                let txs = self
                    .interpreter
                    .prepare(
                        (
                            self.resolve_pool.clone(),
                            self.parent_finality_provider.clone(),
                        ),
                        txs,
                    )
                    .await
                    .context("failed to prepare proposal")?;

                let txs = txs.into_iter().map(bytes::Bytes::from).collect();
                let txs = take_until_max_size(txs, request.max_tx_bytes.try_into().unwrap());
        */
        Ok(response::PrepareProposal { txs })
    }

    /// Inspect a proposal and decide whether to vote on it.
    async fn process_proposal(
        &self,
        _request: request::ProcessProposal,
    ) -> AbciResult<response::ProcessProposal> {
        //todo based on exec state judge if tx is accept
        let accept = true;
        // let accept = self
        //     .interpreter
        //     .process(
        //         (
        //             self.resolve_pool.clone(),
        //             self.parent_finality_provider.clone(),
        //         ),
        //         txs,
        //     )
        //     .await
        //     .context("failed to process proposal")?;

        if accept {
            Ok(response::ProcessProposal::Accept)
        } else {
            Ok(response::ProcessProposal::Reject)
        }
    }

    async fn finalize_block(&self, request: FinalizeBlock) -> AbciResult<response::FinalizeBlock> {
        // todo exec txs
        let _txs = request.txs;
        //let tx_results = txs.into_iter().map(|tx| txRes = exec(tx)).collect();
        Ok(response::FinalizeBlock {
            events: vec![],
            tx_results: vec![],
            validator_updates: vec![],
            consensus_param_updates: None,
            app_hash: Default::default(),
        })
    }

    /// Commit the current state at the current height.
    async fn commit(&self) -> AbciResult<response::Commit> {
        // todo get state_root from exec state
        /*
                let exec_state = self.take_exec_state();

                // Commit the execution state to the datastore.
                let mut state = self.committed_state()?;
                state.block_height = exec_state.block_height().try_into()?;
                state.state_params.timestamp = exec_state.timestamp();
                state.state_params.state_root = exec_state.commit().context("failed to commit Evm")?;

                let state_root = state.state_root();

                tracing::debug!(
                    state_root = state_root.to_string(),
                    timestamp = state.state_params.timestamp.0,
                    "commit state"
                );

                // For example if a checkpoint is successfully executed, that's when we want to remove
                // that checkpoint from the pool, and not propose it to other validators again.
                // However, once Tendermint starts delivering the transactions, the commit will surely
                // follow at the end, so we can also remove these checkpoints from memory at the time
                // the transaction is delivered, rather than when the whole thing is committed.
                // It is only important to the persistent database changes as an atomic step in the
                // commit in case the block execution fails somewhere in the middle for unknown reasons.
                // But if that happened, we will have to restart the application again anyway, and
                // repopulate the in-memory checkpoints based on the last committed ledger.
                // So, while the pool is theoretically part of the evolving state and we can pass
                // it in and out, unless we want to defer commit to here (which the interpreters aren't
                // notified about), we could add it to the `ChainMessageInterpreter` as a constructor argument,
                // a sort of "ambient state", and not worry about in in the `App` at all.

                // Commit app state to the datastore.
                self.set_committed_state(state)?;

                // Reset check state.
                let mut guard = self.check_state.lock().await;
                *guard = None;

                tracing::debug!("committed state");
        */

        let response = response::Commit {
            data: self.app_state.app_hash().as_bytes().to_vec().into(),
            // We have to retain blocks until we can support Snapshots.
            retain_height: tendermint::block::Height::try_from(self.app_state.block_height)?,
        };
        Ok(response)
    }
}
