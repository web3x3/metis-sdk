// Copyright 2025 Metis SDK Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Unified Metis Precompiles Collection
//
// Contains all Metis-specific custom precompile contracts:
// - Native Balance Precompile (0x0100) - Native balance management
// - Inference Precompile (0x0999) - AI inference (optional, controlled by feature)

use metis_primitives::{Address, SpecId};
use revm::{
    context::{Cfg, ContextTr},
    context_interface::JournalTr,
    handler::{EthPrecompiles, PrecompileProvider},
    interpreter::{CallInput, Gas, InputsImpl, InstructionResult, InterpreterResult},
    precompile::{Precompiles, PrecompileError, PrecompileResult},
};
use std::sync::OnceLock;

// Import stateful Native Balance Precompile (can actually modify balances)
use crate::native_balance_precompile_stateful::{
    native_balance_precompile_stateful, INativeBalancePrecompile, NATIVE_BALANCE_PRECOMPILE_ADDRESS,
};
use alloy_sol_types::SolCall;

// Conditionally import Inference Precompile
#[cfg(feature = "inference")]
use crate::inference::{get_or_init_engine, GAS_PER_INFERENCE_TOKEN, INFERENCE_PRECOMPILE_ADDRESS};

#[cfg(feature = "inference")]
use crate::runtime::get_runtime;

#[cfg(feature = "inference")]
use alith::{Request, ResponseContent, ResponseTokenUsage};

/// Get the collection containing all Metis custom precompiles
///
/// Contains:
/// - Standard Ethereum precompiles (Prague)
/// - Native Balance Precompile (0x0100) - Registered as placeholder, actual execution in MetisPrecompiles::run()
/// - Inference Precompile (0x0999) - Only included when inference feature is enabled
///
/// Note: Native Balance Precompile (0x0100) is registered as a placeholder so revm knows it's a precompile.
/// The actual stateful execution happens in MetisPrecompiles::run() with full EVM context access.
pub fn metis_precompiles() -> &'static Precompiles {
    static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        #[allow(unused_mut)]
        let mut precompiles = Precompiles::prague().clone();
        
        // ✅ Register Native Balance Precompile (0x0100) as a placeholder
        //    The placeholder tells revm this is a precompile address
        //    The actual execution happens in MetisPrecompiles::run() with full EVM context access
        fn native_balance_placeholder(_bytes: &[u8], _gas_limit: u64) -> PrecompileResult {
            // This should never be called - MetisPrecompiles::run() handles this
            Err(PrecompileError::Fatal("Use MetisPrecompiles::run() instead".to_string()))
        }
        
        precompiles.extend([(
            NATIVE_BALANCE_PRECOMPILE_ADDRESS,
            native_balance_placeholder as fn(&[u8], u64) -> PrecompileResult
        ).into()]);
        
        // Add Inference Precompile (only when feature is enabled)
        #[cfg(feature = "inference")]
        {
            precompiles.extend([(INFERENCE_PRECOMPILE_ADDRESS, |bytes,
                                                                gas_limit|
             -> PrecompileResult {
                let prompt = String::from_utf8(bytes.to_vec())
                    .map_err(|_| PrecompileError::Fatal("Invalid UTF-8 input".to_string()))?;
                let request = Request::new(prompt, "".to_string());
                let result = get_runtime()
                    .block_on(async {
                        let mut engine = get_or_init_engine().write().await;
                        engine.completion(request).await
                    })
                    .map_err(|err| PrecompileError::Fatal(err.to_string()))?;
                let output = result.content();
                let gas_used = GAS_PER_INFERENCE_TOKEN * result.token_usage().total_tokens as u64;
                if gas_used > gas_limit {
                    Err(PrecompileError::OutOfGas)
                } else {
                    Ok(PrecompileOutput::new(
                        gas_used,
                        output.as_bytes().to_vec().into(),
                    ))
                }
            } as PrecompileFn)
                .into()]);
        }
        
        precompiles
    })
}

/// Metis Precompiles Provider
///
/// Unified management of all Metis-specific precompile contracts.
///
/// # Included Precompiles
///
/// - **Native Balance Precompile** (0x0100): Native balance management
/// - **Inference Precompile** (0x0999): AI inference (requires `inference` feature)
///
/// # Usage Example
///
/// ```rust,ignore
/// use metis_vm::MetisPrecompiles;
///
/// // Create precompiles provider
/// let precompiles = MetisPrecompiles::new();
///
/// // Use in EVM configuration
/// let evm_config = ParallelEthEvmConfig::with_precompiles(precompiles);
/// ```
#[derive(Clone)]
pub struct MetisPrecompiles {
    pub precompiles: EthPrecompiles,
}

impl MetisPrecompiles {
    /// Create a new Metis precompiles provider with the given spec
    pub fn new_with_spec(spec: SpecId) -> Self {
        // Initialize inference engine if the feature is enabled
        #[cfg(feature = "inference")]
        {
            get_or_init_engine();
        }

        let precompiles_map = metis_precompiles();

        Self {
            precompiles: EthPrecompiles {
                precompiles: precompiles_map,
                spec: spec.into(),
            },
        }
    }

    /// Create a new Metis precompiles provider with default spec (Prague)
    pub fn new() -> Self {
        Self::new_with_spec(SpecId::PRAGUE)
    }

    /// Get the precompiles collection (similar to reth-bsc's BscPrecompiles::precompiles)
    #[inline]
    pub fn precompiles(&self) -> &'static Precompiles {
        self.precompiles.precompiles
    }
}

impl Default for MetisPrecompiles {
    fn default() -> Self {
        Self::new()
    }
}

impl<CTX> PrecompileProvider<CTX> for MetisPrecompiles
where
    CTX: ContextTr,
    CTX::Journal: JournalTr, // ← Add constraint to support stateful precompiles
{
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: <CTX::Cfg as Cfg>::Spec) -> bool {
        // ✅ Always use Metis precompiles (including Native Balance Precompile)
        // regardless of spec version
        self.precompiles = EthPrecompiles {
            precompiles: metis_precompiles(), // Use unified precompile collection
            spec: spec.into(),
        };
        true
    }

    fn run(
        &mut self,
        context: &mut CTX,
        address: &Address,
        inputs: &InputsImpl,
        is_static: bool,
        gas_limit: u64,
    ) -> Result<Option<Self::Output>, String> {
        // ✅ Special handling: Native Balance Precompile uses stateful version
        if address == &NATIVE_BALANCE_PRECOMPILE_ADDRESS {
            // Get input data
            let input_bytes = match &inputs.input {
                CallInput::SharedBuffer(_) => {
                    // SharedBuffer is not supported for now (precompiles typically don't use SharedBuffer)
                    return Err(
                        "SharedBuffer not supported for Native Balance Precompile".to_string()
                    );
                }
                CallInput::Bytes(bytes) => bytes.as_ref(),
            };

            // ✅ Check static call: state modification not allowed in staticcall
            if is_static && input_bytes.len() >= 4 {
                let selector = &input_bytes[0..4];

                // Check if it's a state-modifying function
                if selector
                    == <INativeBalancePrecompile::addBalanceCall as SolCall>::SELECTOR.as_slice()
                    || selector
                        == <INativeBalancePrecompile::subtractBalanceCall as SolCall>::SELECTOR
                            .as_slice()
                {
                    return Err("State modification not allowed in static call. \
                         Only getBalance can be called via staticcall."
                        .to_string());
                }
            }

            // Call stateful precompile (can actually modify balances)
            let result = native_balance_precompile_stateful(input_bytes, gas_limit, context);

            // Convert PrecompileResult to InterpreterResult
            match result {
                Ok(output) => {
                    let gas_remaining = gas_limit.saturating_sub(output.gas_used);
                    return Ok(Some(InterpreterResult {
                        result: InstructionResult::Return,
                        output: output.bytes,
                        gas: Gas::new(gas_remaining),
                    }));
                }
                Err(e) => {
                    // Return detailed error
                    let error_msg = match &e {
                        PrecompileError::OutOfGas => {
                            "Precompile out of gas".to_string()
                        },
                        PrecompileError::Fatal(msg) => {
                            format!("Precompile fatal error: {}", msg)
                        },
                        _ => {
                            format!("Precompile error: {:?}", e)
                        }
                    };
                    return Err(error_msg);
                }
            }
        }

        // Other precompiles use standard flow
        self.precompiles
            .run(context, address, inputs, is_static, gas_limit)
    }

    fn warm_addresses(&self) -> Box<impl Iterator<Item = Address>> {
        // Add Native Balance Precompile address to warm address list
        let native_balance_addr = NATIVE_BALANCE_PRECOMPILE_ADDRESS;
        let standard_addrs: Vec<Address> = self.precompiles.warm_addresses().collect();

        Box::new(std::iter::once(native_balance_addr).chain(standard_addrs.into_iter()))
    }

    fn contains(&self, address: &Address) -> bool {
        // Special handling for Native Balance Precompile
        if address == &NATIVE_BALANCE_PRECOMPILE_ADDRESS {
            return true;
        }
        
        self.precompiles.contains(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metis_precompiles_provider_contains_native_balance() {
        let provider = MetisPrecompiles::new();
        let addr = NATIVE_BALANCE_PRECOMPILE_ADDRESS;

        // Native Balance Precompile is visible through both:
        // 1. Registered as placeholder in metis_precompiles() HashMap
        // 2. MetisPrecompiles::contains() explicitly checks for it
        assert!(
            provider.contains(&addr),
            "Native Balance Precompile should be accessible"
        );
    }

    #[cfg(feature = "inference")]
    #[test]
    fn test_metis_precompiles_contains_inference() {
        let precompiles = metis_precompiles();
        let addr = INFERENCE_PRECOMPILE_ADDRESS;

        // Inference Precompile should be present when feature is enabled
        assert!(
            precompiles.contains(&addr),
            "Inference Precompile should be present when feature is enabled"
        );
    }

    #[test]
    fn test_native_balance_in_warm_addresses() {
        let provider = MetisPrecompiles::new();
        let warm_addrs: Vec<Address> = provider.warm_addresses().collect();

        // Verify Native Balance Precompile is in warm address list
        assert!(
            warm_addrs.contains(&NATIVE_BALANCE_PRECOMPILE_ADDRESS),
            "Native Balance Precompile should be in warm addresses"
        );
    }
}
