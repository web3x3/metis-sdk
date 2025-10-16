#![allow(missing_docs)]

#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

use metis_chain::provider::ParallelExecutorBuilder;
use reth::cli::Cli;
use reth_node_ethereum::node::EthereumAddOns;
use reth_node_ethereum::EthereumNode;
use tracing::info;

fn main() {
    reth_cli_util::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    if let Err(err) = Cli::parse_args().run(async move |builder, _| {
        info!(target: "metis::cli", "🔥 Launching Metis node with MetisPrecompiles support");

        // ✅ Use ParallelExecutorBuilder - it correctly integrates MetisPrecompiles
        // via the build_evm function in metis-sdk/crates/pe/src/vm.rs
        let handle = builder
            .with_types::<EthereumNode>()
            .with_components(
                EthereumNode::components()
                    .executor(ParallelExecutorBuilder::default())  // ← Uses MetisPrecompiles
            )
            .with_add_ons(EthereumAddOns::default());

        info!(target: "metis::cli", "✅ Node configured with ParallelExecutorBuilder (MetisPrecompiles enabled)");
        handle.launch_with_debug_capabilities().await?.wait_for_node_exit().await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
