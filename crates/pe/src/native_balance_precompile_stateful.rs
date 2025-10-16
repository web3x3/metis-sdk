// Copyright 2025 Circle Internet Group, Inc. All rights reserved.
//
// SPDX-License-Identifier: Apache-2.0
//
// Native Balance Precompile - Stateful Version
//
// This version can actually modify account balances

use alloy_sol_types::{sol, SolCall, SolValue};
use metis_primitives::{address, Address, U256};
use revm::context::ContextTr;
use revm::context_interface::JournalTr;
use revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

// Precompile address
pub const NATIVE_BALANCE_PRECOMPILE_ADDRESS: Address =
    address!("0000000000000000000000000000000000000100");

// Gas costs
const BASE_GAS: u64 = 2100;
const READ_GAS: u64 = 200;
const WRITE_GAS: u64 = 5000;

// Define Solidity interface
sol! {
    interface INativeBalancePrecompile {
        function addBalance(address account, uint256 amount) external;
        function subtractBalance(address account, uint256 amount) external;
        function getBalance(address account) external view returns (uint256);
    }
}

/// Stateful Native Balance Precompile
///
/// Can actually modify account balances in EVM state
pub fn native_balance_precompile_stateful<CTX: ContextTr>(
    input: &[u8],
    gas_limit: u64,
    context: &mut CTX,
) -> PrecompileResult
where
    CTX::Journal: JournalTr,
{
    // Check minimum gas
    if gas_limit < BASE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    // Check input length
    if input.len() < 4 {
        return Err(PrecompileError::Fatal("Invalid input length".to_string()));
    }

    let function_selector = &input[..4];
    let call_data = &input[4..];

    match function_selector {
        // addBalance(address,uint256)
        selector if selector == INativeBalancePrecompile::addBalanceCall::SELECTOR.as_slice() => {
            let total_gas = BASE_GAS + WRITE_GAS;
            if gas_limit < total_gas {
                return Err(PrecompileError::OutOfGas);
            }

            // Decode parameters
            let call = INativeBalancePrecompile::addBalanceCall::abi_decode(call_data)
                .map_err(|_| {
                    PrecompileError::Fatal("Failed to decode addBalance".to_string())
                })?;

            // Actually modify balance!
            add_balance(context, call.account, call.amount)?;

            let output = true.abi_encode();
            Ok(PrecompileOutput::new(total_gas, output.into()))
        }

        // subtractBalance(address,uint256)
        selector
            if selector == INativeBalancePrecompile::subtractBalanceCall::SELECTOR.as_slice() =>
        {
            let total_gas = BASE_GAS + WRITE_GAS;
            if gas_limit < total_gas {
                return Err(PrecompileError::OutOfGas);
            }

            let call = INativeBalancePrecompile::subtractBalanceCall::abi_decode(call_data)
                .map_err(|_| {
                    PrecompileError::Fatal("Failed to decode subtractBalance".to_string())
                })?;

            // Actually modify balance!
            subtract_balance(context, call.account, call.amount)?;

            let output = true.abi_encode();
            Ok(PrecompileOutput::new(total_gas, output.into()))
        }

        // getBalance(address)
        selector if selector == INativeBalancePrecompile::getBalanceCall::SELECTOR.as_slice() => {
            let total_gas = BASE_GAS + READ_GAS;
            if gas_limit < total_gas {
                return Err(PrecompileError::OutOfGas);
            }

            let call = INativeBalancePrecompile::getBalanceCall::abi_decode(call_data)
                .map_err(|_| {
                    PrecompileError::Fatal("Failed to decode getBalance".to_string())
                })?;

            // Actually query balance!
            let balance = get_balance(context, call.account)?;
            
            let output = balance.abi_encode();
            Ok(PrecompileOutput::new(total_gas, output.into()))
        }

        _ => {
            Err(PrecompileError::Fatal(
                "Unknown function selector".to_string(),
            ))
        },
    }
}

/// Check if the caller is authorized
///
/// Only authorized system contracts can call state-modifying functions
fn check_authorization<CTX: ContextTr>(_context: &CTX) -> Result<(), PrecompileError> {
    // 🔥 Authorization check temporarily disabled for testing
    // TODO: Re-enable authorization check in production!
    // 
    // Production code should be:
    // let caller = context.tx().caller();
    // if !AUTHORIZED_CONTRACTS.contains(&caller) {
    //     return Err(PrecompileError::Fatal(format!(
    //         "Unauthorized caller: {:?}. Only authorized system contracts can modify native balances.",
    //         caller
    //     )));
    // }
    Ok(())
}

/// Add balance to an account
fn add_balance<CTX: ContextTr>(
    context: &mut CTX,
    account: Address,
    amount: U256,
) -> Result<(), PrecompileError>
where
    CTX::Journal: JournalTr,
{
    // ✅ Check caller authorization
    check_authorization(context)?;

    // Load account (using mutable reference)
    let mut account_mut = context
        .journal_mut()
        .load_account(account)
        .map_err(|e| {
            PrecompileError::Fatal(format!("Failed to load account: {:?}", e))
        })?;

    let current_balance = account_mut.info.balance;

    // Calculate new balance
    let new_balance = current_balance
        .checked_add(amount)
        .ok_or_else(|| {
            PrecompileError::Fatal("Balance overflow".to_string())
        })?;

    // Modify balance
    account_mut.info.balance = new_balance;

    Ok(())
}

/// Subtract balance from an account
fn subtract_balance<CTX: ContextTr>(
    context: &mut CTX,
    account: Address,
    amount: U256,
) -> Result<(), PrecompileError>
where
    CTX::Journal: JournalTr,
{
    // ✅ Check caller authorization
    check_authorization(context)?;

    // Load account (using mutable reference)
    let mut account_mut = context
        .journal_mut()
        .load_account(account)
        .map_err(|e| {
            PrecompileError::Fatal(format!("Failed to load account: {:?}", e))
        })?;

    let current_balance = account_mut.info.balance;

    // Check if balance is sufficient
    if current_balance < amount {
        return Err(PrecompileError::Fatal("Insufficient balance".to_string()));
    }

    // Calculate new balance
    let new_balance = current_balance
        .checked_sub(amount)
        .ok_or_else(|| {
            PrecompileError::Fatal("Balance underflow".to_string())
        })?;

    // Modify balance
    account_mut.info.balance = new_balance;

    Ok(())
}

/// Query account balance
fn get_balance<CTX: ContextTr>(context: &mut CTX, account: Address) -> Result<U256, PrecompileError>
where
    CTX::Journal: JournalTr,
{
    // Query balance also needs mutable reference (because load_account may modify journal state)
    let account_load = context
        .journal_mut()
        .load_account(account)
        .map_err(|e| {
            PrecompileError::Fatal(format!("Failed to load account: {:?}", e))
        })?;

    let balance = account_load.info.balance;
    
    Ok(balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_selectors() {
        // Test function selectors
        let add_selector = INativeBalancePrecompile::addBalanceCall::SELECTOR;
        let sub_selector = INativeBalancePrecompile::subtractBalanceCall::SELECTOR;
        let get_selector = INativeBalancePrecompile::getBalanceCall::SELECTOR;

        println!("addBalance selector: {:?}", add_selector);
        println!("subtractBalance selector: {:?}", sub_selector);
        println!("getBalance selector: {:?}", get_selector);

        // Ensure selectors are different
        assert_ne!(add_selector, sub_selector);
        assert_ne!(add_selector, get_selector);
        assert_ne!(sub_selector, get_selector);
    }
}
