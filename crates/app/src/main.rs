#![allow(missing_docs)]

use metis_app::cmd;
use metis_app_options as options;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let opts = options::parse();

    // Log events to stdout.
    if let Some(level) = opts.tracing_level() {
        // Writing to stderr so if we have output like JSON then we can pipe it to something else.
        let subscriber = FmtSubscriber::builder()
            .with_max_level(level)
            .with_writer(std::io::stderr)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }

    if let Err(e) = cmd::exec(&opts).await {
        tracing::error!("failed to execute {:?}: {e:?}", opts);
        std::process::exit(1);
    }
}
