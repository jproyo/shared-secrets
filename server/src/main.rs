#![warn(rust_2018_idioms, missing_debug_implementations)]

use actix_web::dev::ServerHandle;
use log::{info, warn};
use shared_secret_server::conf::settings::Settings;
use shared_secret_server::consensus::handler::ConsensusHandler;
use shared_secret_server::consensus::raft::{init_consensus, HashStore};
use shared_secret_server::domain::model::NodeId;
use shared_secret_server::refresher::secret;
use shared_secret_server::routes::http;
use slog::{slog_o, Drain};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::{AbortHandle, JoinHandle};

fn gracefully_shutdown(
    server_handle: ServerHandle,
    raft_abortable: AbortHandle,
    cron_abortable: AbortHandle,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut sig_int = signal(SignalKind::interrupt()).unwrap();
        let mut sig_term = signal(SignalKind::terminate()).unwrap();
        let mut sig_hup = signal(SignalKind::hangup()).unwrap();
        let cancel = async {
            warn!("Shutdown was requested.\nShutting down http server....");
            server_handle.stop(true).await;
            warn!("Shutting down Consensus Module....");
            raft_abortable.abort();
            warn!("Shutting down Cron Module....");
            cron_abortable.abort();
        };
        tokio::select! {
            _ = sig_int.recv() => cancel.await,
            _ = sig_term.recv() => cancel.await,
            _ = sig_hup.recv() => cancel.await,
        }
    })
}

fn print_wellcome(options: &Settings) {
    let str_log_wellcome = r#"
        ------------------------------------------------------------------------
        |                                                                      |
        |  Welcome to the Distributed Secret Sharing Service!                  |
        |                                                                      |
        |  The service is implemented using the Raft consensus algorithm.      |
        |                                                                      |
        |  Interactions with Clients are accessible via a REST API             |
        |                                                                      |
        ------------------------------------------------------------------------
    "#;

    info!("{}", str_log_wellcome);
    info!("\n\n ---- Starting Node Id {} ----- \n", options.node_id());
    info!(
        "\n\n ---- Starting API Server on {} ----- \n",
        options.web_server()
    );
    info!(
        "\n\n ---- Starting Consensus Server on {} ----- \n",
        options.raft_addr()
    );
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let level = slog::LevelFilter::new(drain, slog::Level::Info).fuse();
    let drain = slog_async::Async::new(level).build().fuse();
    let logger = slog::Logger::root(drain, slog_o!());

    let _scope_guard = slog_scope::set_global_logger(logger.clone());
    let _log_guard = slog_stdlog::init().unwrap();

    let options = &Settings::new()?;
    let store = HashStore::new(NodeId(options.node_id()));

    let (raft_handle, mailbox) = init_consensus(
        options.raft_addr(),
        options.peer_addr(),
        store.clone(),
        logger.clone(),
    )
    .await?;

    let consensus_handler = ConsensusHandler::new(store, mailbox);

    let server = http::run(options, consensus_handler.clone()).await?;

    let server_handle = server.handle();

    let http_server = tokio::spawn(server);

    let cron_refresher = secret::run(options.interval_refresh_secs(), consensus_handler.clone());

    let cron_refresher = tokio::spawn(cron_refresher);

    let graceful_shutdown = gracefully_shutdown(
        server_handle,
        raft_handle.abort_handle(),
        cron_refresher.abort_handle(),
    );

    print_wellcome(options);

    let result = tokio::try_join!(raft_handle, http_server, cron_refresher, graceful_shutdown,)?;
    let (raft_result, http_server_result, _, _) = result;
    raft_result?;
    http_server_result?;
    Ok(())
}
