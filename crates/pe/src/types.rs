//! Reference: https://github.com/risechain/pevm

use metis_primitives::{Address, B256, BuildIdentityHasher, HashMap, U256};
use revm::state::AccountInfo;
use smallvec::SmallVec;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct AtomicWrapper<T: Into<usize> + From<usize>> {
    inner: AtomicUsize,
    _marker: PhantomData<T>,
}

impl<T: Into<usize> + From<usize>> AtomicWrapper<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: AtomicUsize::new(value.into()),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn load(&self, ordering: Ordering) -> T {
        T::from(self.inner.load(ordering))
    }

    #[inline]
    pub fn store(&self, value: T, ordering: Ordering) {
        self.inner.store(value.into(), ordering);
    }

    #[inline]
    pub fn swap(&self, value: T, ordering: Ordering) -> T {
        T::from(self.inner.swap(value.into(), ordering))
    }

    #[inline]
    pub fn compare_exchange(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.inner
            .compare_exchange(current.into(), new.into(), success, failure)
            .map(T::from)
            .map_err(T::from)
    }
}

impl<T: Into<usize> + From<usize> + fmt::Debug> fmt::Debug for AtomicWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.load(Ordering::SeqCst).fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountMeta {
    CA(U256),
    EOA(U256),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Location {
    Basic(Address),
    CodeHash(Address),
    Storage(Address, U256),
}

/// We only need the full location to read from storage.
/// We then identify the locations with its hash in the multi-version
/// data, write and read sets, which is much faster than rehashing
/// on every single lookup & validation.
pub type LocationHash = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocationValue {
    Basic(AccountInfo),
    CodeHash(B256),
    Storage(U256),
    // We lazily update the beneficiary balance to avoid continuous
    // dependencies as all transactions read and write to it. We also
    // lazy update the senders & recipients of raw transfers, which are
    // also common (popular CEX addresses, airdrops, etc).
    // We fully evaluate these account states at the end of the block or
    // when there is an explicit read.
    // Explicit balance addition.
    LazyRecipient(U256),
    // Explicit balance subtraction & implicit nonce increment.
    LazySender(U256),
    // The account was self-destructed.
    SelfDestructed,
}

#[derive(Debug)]
pub enum Entry {
    Data(TxIncarnation, LocationValue),
    // When an incarnation is aborted due to a validation failure, the
    // entries in the multi-version data structure corresponding to its
    // write set are replaced with this special ESTIMATE marker.
    // This signifies that the next incarnation is estimated to write to
    // the same locations. An incarnation stops and is immediately
    // aborted whenever it reads a value marked as an ESTIMATE written by
    // a lower transaction, instead of potentially wasting a full execution
    // and aborting during validation.
    // The ESTIMATE markers that are not overwritten are removed by the next
    // incarnation.
    Estimate,
}

/// The index of the transaction in the block.
pub type TxIdx = usize;

/// The i-th time a transaction is re-executed, counting from 0.
pub type TxIncarnation = usize;

/// Execution:
/// - ReadyToExecute(i) --try_incarnate--> Executing(i)
///
/// Non-blocked execution:
/// - Executing(i) --finish_execution--> Executed(i)
/// - Executed(i) --finish_validation--> Validated(i)
/// - Executed/Validated(i) --try_validation_abort--> Aborting(i)
/// - Aborted(i) --finish_validation(w.aborted=true)--> ReadyToExecute(i+1)
///
/// Blocked execution:
/// - Executing(i) --add_dependency--> Aborting(i)
/// - Aborted(i) --resume--> ReadyToExecute(i+1)
#[derive(PartialEq, Clone, Debug)]
pub enum IncarnationStatus {
    ReadyToExecute = 0,
    Executing,
    Executed,
    Validated,
    Aborted,
}

#[derive(PartialEq, Clone, Debug)]
pub struct TxStatus {
    pub incarnation: TxIncarnation,
    pub status: IncarnationStatus,
}

impl From<usize> for TxStatus {
    fn from(value: usize) -> Self {
        let incarnation = value >> 8;
        let status = match value & 0xFF {
            0 => IncarnationStatus::ReadyToExecute,
            1 => IncarnationStatus::Executing,
            2 => IncarnationStatus::Executed,
            3 => IncarnationStatus::Validated,
            _ => IncarnationStatus::Aborted,
        };
        Self {
            incarnation,
            status,
        }
    }
}

impl From<TxStatus> for usize {
    fn from(val: TxStatus) -> Self {
        (val.incarnation << 8) | val.status as usize
    }
}

/// We maintain an in-memory multi-version data structure that stores for
/// each location the latest value written per transaction, along
/// with the associated transaction incarnation. When a transaction reads
/// a location, it obtains from the multi-version data structure the
/// value written to this location by the highest transaction that appears
/// before it in the block, along with the associated version. If no previous
/// transactions have written to a location, the value would be read from the
/// storage state before block execution.
#[derive(Clone, Debug, PartialEq)]
pub struct TxVersion {
    pub tx_idx: TxIdx,
    pub tx_incarnation: TxIncarnation,
}

/// The origin of a read. It could be from the live multi-version
/// data structure or from storage (chain state before block execution).
#[derive(Debug, PartialEq)]
pub enum ReadOrigin {
    MvMemory(TxVersion),
    Storage,
}

/// A scheduled worker task
#[derive(Debug)]
pub enum Task {
    Execution(TxVersion),
    Validation(TxVersion),
}

/// Most locations only have one read origin. Lazy updated ones like
/// the beneficiary balance, raw transfer senders & recipients, etc. have a
/// list of lazy updates all the way to the first strict/absolute value.
pub type ReadOrigins = SmallVec<[ReadOrigin; 1]>;

/// For validation: a list of read origins (previous transaction versions)
/// for each read location.
pub type ReadSet = HashMap<LocationHash, ReadOrigins, BuildIdentityHasher>;

/// The updates made by this transaction incarnation, which is applied
/// to the multi-version data structure at the end of execution.
pub type WriteSet = HashMap<LocationHash, LocationValue, BuildIdentityHasher>;

/// Read and write set for each transaction.
#[derive(Default, Debug)]
pub struct ReadWriteSet {
    /// Read locations in the transaction.
    pub read: ReadSet,
    /// Write locations in the transaction.
    pub write: SmallVec<[LocationHash; 2]>,
}
