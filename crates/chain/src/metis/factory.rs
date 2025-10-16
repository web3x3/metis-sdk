// Copyright 2025 Metis SDK Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Metis EVM Factory - Creates EthEvm instances with MetisPrecompiles

use crate::metis::evm::MetisEvm;
use alloy_evm::eth::EthEvmContext;
use metis_primitives::{SpecId, TxEnv};
use metis_vm::MetisPrecompiles;
use reth_evm::{Database, EvmEnv, EvmFactory};
use revm::{
    context::{BlockEnv, CfgEnv, Context},
    context_interface::result::{EVMError, HaltReason},
    inspector::NoOpInspector,
    Inspector, MainBuilder, MainContext,
};

/// Factory producing [`MetisEvm`] (EthEvm with MetisPrecompiles).
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct MetisEvmFactory;

impl EvmFactory for MetisEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>>> = MetisEvm<DB, I>;
    type Context<DB: Database> = Context<BlockEnv, TxEnv, CfgEnv<SpecId>, DB>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Spec = SpecId;
    type Precompiles = MetisPrecompiles;  // ✅ Use MetisPrecompiles (supports stateful precompiles)

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: EvmEnv<SpecId>,
    ) -> Self::Evm<DB, NoOpInspector> {
        let spec_id = input.cfg_env.spec;
        
        // ✅ Create MetisPrecompiles (supports stateful native balance precompile)
        let metis_precompiles = MetisPrecompiles::new_with_spec(spec_id);
        
        // Build RevmEvm with MetisPrecompiles
        let revm_evm = Context::mainnet()
            .with_block(input.block_env)
            .with_cfg(input.cfg_env)
            .with_db(db)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(metis_precompiles);  // ✅ Use MetisPrecompiles directly
        
        // Wrap in EthEvm
        alloy_evm::EthEvm::new(revm_evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: EvmEnv<SpecId>,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        let spec_id = input.cfg_env.spec;
        
        // ✅ Create MetisPrecompiles (supports stateful native balance precompile)
        let metis_precompiles = MetisPrecompiles::new_with_spec(spec_id);
        
        // Build RevmEvm with inspector and MetisPrecompiles
        let revm_evm = Context::mainnet()
            .with_block(input.block_env)
            .with_cfg(input.cfg_env)
            .with_db(db)
            .build_mainnet_with_inspector(inspector)
            .with_precompiles(metis_precompiles);  // ✅ Use MetisPrecompiles directly
        
        // Wrap in EthEvm
        alloy_evm::EthEvm::new(revm_evm, true)
    }
}
