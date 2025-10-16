// Copyright 2025 Metis SDK Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Metis custom EVM implementation
//
// This module provides a complete custom EVM implementation with MetisPrecompiles support.

pub mod config;
pub mod evm;
pub mod factory;

pub use config::{MetisBlockExecutorFactory, MetisEvmConfig};
pub use evm::MetisEvm;
pub use factory::MetisEvmFactory;
