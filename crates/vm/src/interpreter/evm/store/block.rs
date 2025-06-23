use cid::Cid;

#[derive(Copy, Clone)]
pub struct Block<D>
where
    D: AsRef<[u8]> + ?Sized,
{
    pub codec: u64,
    pub data: D,
}

pub trait Blockstore {
    /// Gets the block from the blockstore.
    fn get(&self, k: &Cid) -> anyhow::Result<Option<Vec<u8>>>;

    /// Put a block with a pre-computed cid.
    ///
    /// If you don't yet know the CID, use put. Some blockstores will re-compute the CID internally
    /// even if you provide it.
    ///
    /// If you _do_ already know the CID, use this method as some blockstores _won't_ recompute it.
    fn put_keyed(&self, k: &Cid, block: &[u8]) -> anyhow::Result<()>;

    /// Checks if the blockstore has the specified block.
    fn has(&self, k: &Cid) -> anyhow::Result<bool> {
        Ok(self.get(k)?.is_some())
    }

    /// Puts the block into the blockstore, computing the hash with the specified multicodec.
    ///
    /// By default, this defers to put.
    fn put<D>(&self, _block: &Block<D>) -> anyhow::Result<Cid>
    where
        Self: Sized,
        D: AsRef<[u8]>,
    {
        Ok(Cid::default())
    }

    fn put_many<D, I>(&self, _blocks: I) -> anyhow::Result<()>
    where
        Self: Sized,
        D: AsRef<[u8]>,
        I: IntoIterator<Item = Block<D>>,
    {
        Ok(())
    }

    /// Bulk-put pre-keyed blocks into the blockstore.
    ///
    /// By default, this defers to put_keyed.
    fn put_many_keyed<D, I>(&self, blocks: I) -> anyhow::Result<()>
    where
        Self: Sized,
        D: AsRef<[u8]>,
        I: IntoIterator<Item = (Cid, D)>,
    {
        for (c, b) in blocks {
            self.put_keyed(&c, b.as_ref())?
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ReadOnlyBlockstore<DB>(DB);

impl<DB> ReadOnlyBlockstore<DB> {
    pub fn new(store: DB) -> Self {
        Self(store)
    }
}

impl<DB> Blockstore for ReadOnlyBlockstore<DB>
where
    DB: Blockstore,
{
    fn get(&self, k: &Cid) -> anyhow::Result<Option<Vec<u8>>> {
        self.0.get(k)
    }

    fn put_keyed(&self, k: &Cid, block: &[u8]) -> anyhow::Result<()> {
        self.0.put_keyed(k, block)
    }
}
