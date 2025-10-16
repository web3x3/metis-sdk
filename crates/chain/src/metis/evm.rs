// Copyright 2025 Metis SDK Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Metis EVM - Type alias for EthEvm with MetisPrecompiles

use metis_vm::MetisPrecompiles;

/// Metis EVM - type alias for EthEvm with MetisPrecompiles.
/// MetisPrecompiles implements PrecompileProvider and supports stateful precompiles
/// including the Native Balance Precompile that can modify account balances.
pub type MetisEvm<DB, I> = alloy_evm::EthEvm<DB, I, MetisPrecompiles>;
