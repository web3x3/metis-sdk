use anyhow::{Context, anyhow};
use cid::Cid;
use std::cell::RefCell;

use crate::interpreter::evm::{
    state::{
        CheckStateRef,
        exec::{EvmExecState, EvmStateParams},
    },
    store::block::{Blockstore, ReadOnlyBlockstore},
};

/// The state over which we run queries. These can interrogate the IPLD block store or the state tree.
#[allow(unreachable_pub, dead_code)]
pub struct EvmQueryState<DB>
where
    DB: Blockstore + 'static,
{
    /// A read-only wrapper around the blockstore, to make sure we aren't
    /// accidentally committing any state. Any writes by the Evm will be
    /// buffered; as long as we don't call `flush()` we should be fine.
    store: ReadOnlyBlockstore<DB>,
    // /// Multi-engine for potential message execution.
    // multi_engine: Arc<MultiEngine>,
    /// Height of block at which we are executing the queries.
    block_height: i64,
    /// State at the height we want to query.
    state_params: EvmStateParams,
    /// Lazy loaded execution state.
    exec_state: RefCell<Option<EvmExecState<ReadOnlyBlockstore<DB>>>>,
    /// Lazy locked check state.
    check_state: CheckStateRef<DB>,
    /// Whether to try to use the check state or not.
    pending: bool,
}

impl<DB> EvmQueryState<DB>
where
    DB: Blockstore + Clone + 'static,
{
    pub fn new(
        blockstore: DB,
        // multi_engine: Arc<MultiEngine>,
        block_height: i64,
        state_params: EvmStateParams,
        check_state: CheckStateRef<DB>,
        pending: bool,
    ) -> anyhow::Result<Self> {
        // Sanity check that the blockstore contains the supplied state root.
        if !blockstore
            .has(&state_params.state_root)
            .context("failed to load state-root")?
        {
            return Err(anyhow!(
                "blockstore doesn't have the state-root {}",
                state_params.state_root
            ));
        }

        let state = Self {
            store: ReadOnlyBlockstore::new(blockstore),
            // multi_engine,
            block_height,
            state_params,
            exec_state: RefCell::new(None),
            check_state,
            pending,
        };

        Ok(state)
    }
    /*
    /// Do not make the changes in the call persistent. They should be run on top of
    /// transactions added to the mempool, but they can run independent of each other.
    ///
    /// There is no way to specify stacking in the API and only transactions should modify things.
    fn with_revert<T, F>(
        &self,
        exec_state: &mut EvmExecState<ReadOnlyBlockstore<DB>>,
        f: F,
    ) -> anyhow::Result<T>
    where
        F: FnOnce(&mut EvmExecState<ReadOnlyBlockstore<DB>>) -> anyhow::Result<T>,
    {
        exec_state.state_tree_mut().begin_transaction();

        let res = f(exec_state);

        exec_state
            .state_tree_mut()
            .end_transaction(true)
            .expect("we just started a transaction");
        res
    }

    /// If we know the query is over the state, cache the state tree.
    async fn with_exec_state<T, F>(self, f: F) -> anyhow::Result<(Self, T)>
    where
        F: FnOnce(&mut EvmExecState<ReadOnlyBlockstore<DB>>) -> anyhow::Result<T>,
    {
        if self.pending {
            // XXX: This will block all `check_tx` from going through and also all other queries.
            let mut guard = self.check_state.lock().await;

            if let Some(ref mut exec_state) = *guard {
                let res = self.with_revert(exec_state, f);
                drop(guard);
                return res.map(|r| (self, r));
            }
        }

        // Not using pending, or there is no pending state.
        let mut cache = self.exec_state.borrow_mut();

        if let Some(exec_state) = cache.as_mut() {
            let res = self.with_revert(exec_state, f);
            drop(cache);
            return res.map(|r| (self, r));
        }

        let mut exec_state = EvmExecState::new(
            // self.store.clone(),
            // self.multi_engine.as_ref(),
            // self.block_height,
            // self.state_params.clone(),
        )
        .context("error creating execution state")?;

        let res = self.with_revert(&mut exec_state, f);

        *cache = Some(exec_state);
        drop(cache);

        res.map(|r| (self, r))
    }
    */

    /// Read a CID from the underlying IPLD store.
    pub fn store_get(&self, key: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        self.store.get(key)
    }

    // /// Get the state of an actor, if it exists.
    // pub async fn actor_state(
    //     self,
    //     addr: &Address,
    // ) -> anyhow::Result<(Self, Option<(ActorID, ActorState)>)> {
    //     self.with_exec_state(|exec_state| {
    //         let state_tree = exec_state.state_tree_mut();
    //         get_actor_state(state_tree, addr)
    //     })
    //     .await
    // }

    // /// Run a "read-only" message.
    // ///
    // /// The results are never going to be flushed, so it's semantically read-only,
    // /// but it might write into the buffered block store the Evm creates. Running
    // /// multiple such messages results in their buffered effects stacking up,
    // /// unless it's called with `revert`.
    // pub async fn call(
    //     self,
    //     mut msg: EvmMessage,
    // ) -> anyhow::Result<(Self, (ApplyRet, HashMap<u64, Address>))> {
    //     self.with_exec_state(|s| {
    //         // If the sequence is zero, treat it as a signal to use whatever is in the state.
    //         if msg.sequence.is_zero() {
    //             let state_tree = s.state_tree_mut();
    //             if let Some(id) = state_tree.lookup_id(&msg.from)? {
    //                 state_tree.get_actor(id)?.map(|st| {
    //                     msg.sequence = st.sequence;
    //                     st
    //                 });
    //             }
    //         }
    //         if msg.from == SYSTEM_ACTOR_ADDR {
    //             // Explicit execution requires `from` to be an account kind.
    //             s.execute_implicit(msg)
    //         } else {
    //             s.execute_explicit(msg)
    //         }
    //     })
    //     .await
    // }

    pub fn state_params(&self) -> &EvmStateParams {
        &self.state_params
    }

    pub fn block_height(&self) -> i64 {
        self.block_height
    }

    pub fn pending(&self) -> bool {
        self.pending
    }
}
/*
impl<DB> HasChainID for EvmQueryState<DB>
where
    DB: Blockstore + 'static,
{
    fn chain_id(&self) -> ChainID {
        ChainID::from(self.state_params.chain_id)
    }
}

fn get_actor_state<DB>(
    state_tree: &StateTree<DB>,
    addr: &Address,
) -> anyhow::Result<Option<(ActorID, ActorState)>>
where
    DB: Blockstore,
{
    if let Some(id) = state_tree.lookup_id(addr)? {
        Ok(state_tree.get_actor(id)?.map(|st| {
            let st = ActorState {
                code: st.code,
                state: st.state,
                sequence: st.sequence,
                balance: st.balance,
                delegated_address: st.delegated_address,
            };
            (id, st)
        }))
    } else {
        Ok(None)
    }
}
*/
