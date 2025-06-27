use metis_primitives::{B256, HashMap};
use metis_vm::interpreter::evm::store::block::Blockstore;
use std::sync::{Arc, RwLock};

// todo should be replaced by db/reth-db
#[allow(dead_code, unreachable_pub)]
#[derive(Clone, Default)]
pub struct MemoryBlockstore {
    data: Arc<RwLock<HashMap<B256, Vec<u8>>>>,
}

impl Blockstore for MemoryBlockstore {
    fn get(&self, k: &B256) -> anyhow::Result<Option<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.get(k).cloned())
    }

    fn put_keyed(&self, k: &B256, block: &[u8]) -> anyhow::Result<()> {
        let mut data = self.data.write().unwrap();
        data.insert(*k, block.to_vec());
        Ok(())
    }
}
