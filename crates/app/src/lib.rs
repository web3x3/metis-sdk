#![allow(clippy::type_repetition_in_bounds)]
#![allow(missing_docs)]

mod app;

// Different type from `ChainEpoch` just because we might use epoch in a more traditional sense for checkpointing.
pub type BlockHeight = u64;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_VERSION: u64 = 0;
