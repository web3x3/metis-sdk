// Copyright 2025 Metis SDK Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Metis EVM Config - Complete custom EVM configuration with MetisPrecompiles

use super::MetisEvmFactory;
use alloy_consensus::{BlockHeader, Header, TxReceipt};
use alloy_evm::eth::EthBlockExecutor;
use alloy_hardforks::EthereumHardfork;
use alloy_primitives::{Log, U256};
use reth::revm::db::State;
use reth_chainspec::{ChainSpec, EthChainSpec, EthereumHardforks, Hardforks};
use reth_ethereum_primitives::{EthPrimitives, TransactionSigned};
use reth_evm::{
    block::{BlockExecutorFactory, BlockExecutorFor},
    eth::{receipt_builder::ReceiptBuilder, EthBlockExecutionCtx},
    ConfigureEvm, EvmEnv, EvmFactory, ExecutionCtxFor, FromRecoveredTx, FromTxWithEncoded,
    NextBlockEnvAttributes,
};
use reth_evm_ethereum::{EthBlockAssembler, RethReceiptBuilder};
use reth_primitives::{BlockTy, HeaderTy, SealedBlock, SealedHeader};
use revm::{
    context::{BlockEnv, CfgEnv},
    context_interface::block::BlobExcessGasAndPrice,
    primitives::hardfork::SpecId,
    Inspector,
};
use std::{borrow::Cow, convert::Infallible, sync::Arc};

/// Map the latest active hardfork at the given header to a revm [`SpecId`].
fn revm_spec(chain_spec: &ChainSpec, header: &Header) -> SpecId {
    revm_spec_by_timestamp_and_block_number(chain_spec, header.timestamp(), header.number())
}

/// Map the latest active hardfork at the given timestamp or block number to a revm [`SpecId`].
fn revm_spec_by_timestamp_and_block_number(
    chain_spec: &ChainSpec,
    timestamp: u64,
    block_number: u64,
) -> SpecId {
    if chain_spec
        .fork(EthereumHardfork::Osaka)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::OSAKA
    } else if chain_spec
        .fork(EthereumHardfork::Prague)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::PRAGUE
    } else if chain_spec
        .fork(EthereumHardfork::Cancun)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::CANCUN
    } else if chain_spec
        .fork(EthereumHardfork::Shanghai)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::SHANGHAI
    } else if chain_spec
        .fork(EthereumHardfork::Paris)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::MERGE
    } else if chain_spec
        .fork(EthereumHardfork::London)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::LONDON
    } else if chain_spec
        .fork(EthereumHardfork::Berlin)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::BERLIN
    } else if chain_spec
        .fork(EthereumHardfork::Istanbul)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::ISTANBUL
    } else if chain_spec
        .fork(EthereumHardfork::Petersburg)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::PETERSBURG
    } else if chain_spec
        .fork(EthereumHardfork::Constantinople)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::CONSTANTINOPLE
    } else if chain_spec
        .fork(EthereumHardfork::Byzantium)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::BYZANTIUM
    } else if chain_spec
        .fork(EthereumHardfork::SpuriousDragon)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::SPURIOUS_DRAGON
    } else if chain_spec
        .fork(EthereumHardfork::Tangerine)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::TANGERINE
    } else if chain_spec
        .fork(EthereumHardfork::Homestead)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::HOMESTEAD
    } else if chain_spec
        .fork(EthereumHardfork::Frontier)
        .active_at_timestamp_or_number(timestamp, block_number)
    {
        SpecId::FRONTIER
    } else {
        SpecId::FRONTIER
    }
}

/// Metis EVM configuration
#[derive(Debug, Clone)]
pub struct MetisEvmConfig {
    /// Inner block executor factory
    pub executor_factory: MetisBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, MetisEvmFactory>,
    /// Block assembler
    pub block_assembler: EthBlockAssembler<ChainSpec>,
}

impl MetisEvmConfig {
    /// Creates a new Metis EVM configuration with the given chain spec
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self::new_with_evm_factory(chain_spec, MetisEvmFactory::default())
    }

    /// Creates a new Metis EVM configuration with custom EVM factory
    pub fn new_with_evm_factory(chain_spec: Arc<ChainSpec>, evm_factory: MetisEvmFactory) -> Self {
        Self {
            block_assembler: EthBlockAssembler::new(chain_spec.clone()),
            executor_factory: MetisBlockExecutorFactory::new(
                RethReceiptBuilder::default(),
                chain_spec,
                evm_factory,
            ),
        }
    }

    /// Returns the chain spec
    pub const fn chain_spec(&self) -> &Arc<ChainSpec> {
        self.executor_factory.spec()
    }
}

/// Metis block executor factory
#[derive(Debug, Clone, Default, Copy)]
pub struct MetisBlockExecutorFactory<
    R = RethReceiptBuilder,
    Spec = Arc<ChainSpec>,
    EvmFactory = MetisEvmFactory,
> {
    /// Receipt builder
    receipt_builder: R,
    /// Chain specification
    spec: Spec,
    /// EVM factory
    evm_factory: EvmFactory,
}

impl<R, Spec, EvmFactory> MetisBlockExecutorFactory<R, Spec, EvmFactory> {
    /// Creates a new MetisBlockExecutorFactory
    pub const fn new(receipt_builder: R, spec: Spec, evm_factory: EvmFactory) -> Self {
        Self { receipt_builder, spec, evm_factory }
    }

    /// Exposes the receipt builder
    pub const fn receipt_builder(&self) -> &R {
        &self.receipt_builder
    }

    /// Exposes the chain specification
    pub const fn spec(&self) -> &Spec {
        &self.spec
    }
}

impl<R, Spec, EvmF> BlockExecutorFactory for MetisBlockExecutorFactory<R, Spec, EvmF>
where
    R: ReceiptBuilder<Transaction = TransactionSigned, Receipt: TxReceipt<Log = Log>>,
    Spec: EthereumHardforks + EthChainSpec + Hardforks + Clone + alloy_evm::eth::spec::EthExecutorSpec,
    EvmF: EvmFactory<Tx: FromRecoveredTx<TransactionSigned> + FromTxWithEncoded<TransactionSigned>>,
    R::Transaction: From<TransactionSigned> + Clone,
    Self: 'static,
{
    type EvmFactory = EvmF;
    type ExecutionCtx<'a> = EthBlockExecutionCtx<'a>;
    type Transaction = TransactionSigned;
    type Receipt = R::Receipt;

    fn evm_factory(&self) -> &Self::EvmFactory {
        &self.evm_factory
    }

    fn create_executor<'a, DB, I>(
        &'a self,
        evm: <Self::EvmFactory as EvmFactory>::Evm<&'a mut State<DB>, I>,
        ctx: Self::ExecutionCtx<'a>,
    ) -> impl BlockExecutorFor<'a, Self, DB, I>
    where
        DB: alloy_evm::Database + 'a,
        I: Inspector<<Self::EvmFactory as EvmFactory>::Context<&'a mut State<DB>>> + 'a,
    {
        EthBlockExecutor::new(
            evm,
            ctx,
            self.spec().clone(),
            self.receipt_builder(),
        )
    }
}

const EIP1559_INITIAL_BASE_FEE: u64 = 1_000_000_000;

impl ConfigureEvm for MetisEvmConfig
where
    Self: Send + Sync + Unpin + Clone + 'static,
{
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = MetisBlockExecutorFactory;
    type BlockAssembler = EthBlockAssembler<ChainSpec>;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        &self.executor_factory
    }

    fn block_assembler(&self) -> &Self::BlockAssembler {
        &self.block_assembler
    }

    fn evm_env(&self, header: &Header) -> EvmEnv<SpecId> {
        let spec = revm_spec(self.chain_spec(), header);

        let mut cfg_env = CfgEnv::new()
            .with_chain_id(self.chain_spec().chain().id())
            .with_spec(spec);

        // Handle blob params if Cancun is active
        if self.chain_spec().is_cancun_active_at_timestamp(header.timestamp) {
            if let Some(blob_params) = self.chain_spec().blob_params_at_timestamp(header.timestamp) {
                cfg_env.set_max_blobs_per_tx(blob_params.max_blobs_per_tx);
            }
        }

        // Derive EIP-4844 blob fees
        let blob_excess_gas_and_price = if self.chain_spec().is_cancun_active_at_timestamp(header.timestamp) {
            header.excess_blob_gas.zip(self.chain_spec().blob_params_at_timestamp(header.timestamp))
                .map(|(excess_blob_gas, params)| {
                    let blob_gasprice = params.calc_blob_fee(excess_blob_gas);
                    BlobExcessGasAndPrice { excess_blob_gas, blob_gasprice }
                })
        } else {
            None
        };

        let block_env = BlockEnv {
            number: U256::from(header.number()),
            beneficiary: header.beneficiary(),
            timestamp: U256::from(header.timestamp()),
            difficulty: if spec >= SpecId::MERGE { U256::ZERO } else { header.difficulty() },
            prevrandao: if spec >= SpecId::MERGE { header.mix_hash() } else { None },
            gas_limit: header.gas_limit(),
            basefee: header.base_fee_per_gas().unwrap_or_default(),
            blob_excess_gas_and_price,
        };

        EvmEnv { cfg_env, block_env }
    }

    fn next_evm_env(
        &self,
        parent: &Header,
        attributes: &Self::NextBlockEnvCtx,
    ) -> Result<EvmEnv<SpecId>, Self::Error> {
        let spec_id = revm_spec_by_timestamp_and_block_number(
            self.chain_spec(),
            attributes.timestamp,
            parent.number() + 1,
        );

        let cfg_env = CfgEnv::new()
            .with_chain_id(self.chain_spec().chain().id())
            .with_spec(spec_id);

        let blob_params = self.chain_spec().blob_params_at_timestamp(attributes.timestamp);

        // Handle blob excess gas
        let blob_excess_gas_and_price = parent
            .maybe_next_block_excess_blob_gas(blob_params)
            .or_else(|| (spec_id >= SpecId::CANCUN).then_some(0))
            .map(|excess_blob_gas| {
                let blob_gasprice = blob_params
                    .unwrap_or_else(|| alloy_eips::eip7840::BlobParams::cancun())
                    .calc_blob_fee(excess_blob_gas);
                BlobExcessGasAndPrice { excess_blob_gas, blob_gasprice }
            });

        let mut basefee = parent.next_block_base_fee(
            self.chain_spec().base_fee_params_at_timestamp(attributes.timestamp),
        );

        let mut gas_limit = U256::from(parent.gas_limit);

        // Handle London fork boundary
        if self.chain_spec()
            .fork(EthereumHardfork::London)
            .transitions_at_block(parent.number + 1)
        {
            let elasticity_multiplier = self
                .chain_spec()
                .base_fee_params_at_timestamp(attributes.timestamp)
                .elasticity_multiplier;

            gas_limit *= U256::from(elasticity_multiplier);
            basefee = Some(EIP1559_INITIAL_BASE_FEE);
        }

        let block_env = BlockEnv {
            number: U256::from(parent.number() + 1),
            beneficiary: attributes.suggested_fee_recipient,
            timestamp: U256::from(attributes.timestamp),
            difficulty: U256::ZERO,
            prevrandao: Some(attributes.prev_randao),
            gas_limit: attributes.gas_limit,
            basefee: basefee.unwrap_or_default(),
            blob_excess_gas_and_price,
        };

        Ok(EvmEnv { cfg_env, block_env })
    }

    fn context_for_block<'a>(
        &self,
        block: &'a SealedBlock<BlockTy<Self::Primitives>>,
    ) -> ExecutionCtxFor<'a, Self> {
        EthBlockExecutionCtx {
            parent_hash: block.header().parent_hash,
            parent_beacon_block_root: block.header().parent_beacon_block_root,
            ommers: &block.body().ommers,
            withdrawals: block.body().withdrawals.as_ref().map(Cow::Borrowed),
        }
    }

    fn context_for_next_block(
        &self,
        parent: &SealedHeader<HeaderTy<Self::Primitives>>,
        attributes: Self::NextBlockEnvCtx,
    ) -> ExecutionCtxFor<'_, Self> {
        EthBlockExecutionCtx {
            parent_hash: parent.hash(),
            parent_beacon_block_root: attributes.parent_beacon_block_root,
            ommers: &[],
            withdrawals: attributes.withdrawals.map(Cow::Owned),
        }
    }
}
