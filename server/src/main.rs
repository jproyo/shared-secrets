use slog::{info, slog_o, warn, Drain};
use sss_wrap::secret::secret::{RenewableShare, Share};

use actix_web::{get, post, web, App, HttpServer, Responder};
use async_trait::async_trait;
use bincode::{deserialize, serialize};
use riteraft::{Mailbox, Raft, Result, Store};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use structopt::StructOpt;
use tokio::signal::unix::{signal, SignalKind};

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(long)]
    raft_addr: String,
    #[structopt(long)]
    peer_addr: Option<String>,
    #[structopt(long)]
    web_server: String,
    #[structopt(long)]
    node_id: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Insert {
        client_id: ClientId,
        value: Share,
    },
    StartRefresh {
        node_id: NodeId,
    },
    Refresh {
        client_id: ClientId,
        new_share: Share,
    },
    FinishRefresh {
        node_id: NodeId,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
struct ClientId(u64);

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Copy)]
struct NodeId(u8);

impl Deref for ClientId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for NodeId {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
struct HashStore {
    node_id: NodeId,
    storage: Arc<RwLock<HashMap<ClientId, Share>>>,
    refreshing: Arc<AtomicBool>,
}

impl HashStore {
    fn new(node_id: NodeId) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            node_id,
            refreshing: Arc::new(AtomicBool::new(false)),
        }
    }
    fn get(&self, id: ClientId) -> Option<Share> {
        self.storage.read().unwrap().get(&id).cloned()
    }
}

#[async_trait]
impl Store for HashStore {
    async fn apply(&mut self, message: &[u8]) -> Result<Vec<u8>> {
        let message: Message = deserialize(message).unwrap();
        let message: Vec<u8> = match message {
            Message::Insert { client_id, value } => {
                if value.id() == *self.node_id.deref() {
                    let mut db = self.storage.write().unwrap();
                    db.insert(client_id.clone(), value.clone());
                    serialize(&value).unwrap()
                } else {
                    serialize(&self.get(client_id).unwrap()).unwrap()
                }
            }
            Message::StartRefresh { node_id } => {
                if node_id != self.node_id {
                    self.refreshing
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                serialize(&Message::StartRefresh { node_id }).unwrap()
            }
            Message::Refresh {
                client_id,
                new_share,
            } => {
                if new_share.id() == *self.node_id.deref() {
                    let db_read = self.storage.read().unwrap();
                    let old_share = db_read.get(&client_id).unwrap();
                    let new_share_to_store =
                        RenewableShare::renew_with_share(&new_share, &old_share);
                    let mut db = self.storage.write().unwrap();
                    db.insert(client_id, new_share_to_store);
                }
                serialize(&Message::Refresh {
                    client_id,
                    new_share: new_share.clone(),
                })
                .unwrap()
            }
            Message::FinishRefresh { node_id } => {
                if node_id != self.node_id {
                    self.refreshing
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                }
                serialize(&Message::FinishRefresh { node_id }).unwrap()
            }
        };
        Ok(message)
    }

    async fn snapshot(&self) -> Result<Vec<u8>> {
        Ok(serialize(&self.storage.read().unwrap().clone())?)
    }

    async fn restore(&mut self, snapshot: &[u8]) -> Result<()> {
        let new: HashMap<ClientId, Share> = deserialize(snapshot).unwrap();
        let mut db = self.storage.write().unwrap();
        let _ = std::mem::replace(&mut *db, new);
        Ok(())
    }
}

struct AppContext {
    mailbox: Arc<Mailbox>,
    store: HashStore,
    logger: slog::Logger,
    node_id: NodeId,
}

impl AppContext {
    fn new(mailbox: Arc<Mailbox>, store: HashStore, logger: slog::Logger, node_id: u8) -> Self {
        Self {
            mailbox,
            store,
            logger,
            node_id: NodeId(node_id),
        }
    }

    fn mailbox(&self) -> Arc<Mailbox> {
        self.mailbox.clone()
    }

    fn store(&self) -> HashStore {
        self.store.clone()
    }

    fn logger(&self) -> slog::Logger {
        self.logger.clone()
    }
}

#[post("/{client_id}/secret")]
async fn create_share(
    data: web::Data<AppContext>,
    path: web::Path<ClientId>,
    share: web::Json<Share>,
) -> impl Responder {
    info!(
        data.logger(),
        "Creating new share from client {:?} with value {:?}", path, share
    );
    let client_id = path.into_inner();
    let message = Message::Insert {
        client_id,
        value: share.deref().clone(),
    };
    let message = serialize(&message).unwrap();
    let result = data.mailbox().send(message).await.unwrap();
    info!(data.logger(), "Result: {:?}", result);
    let result: Share = deserialize(&result).unwrap();
    format!("{:?}", result)
}

#[get("/{id}/share")]
async fn get_share(data: web::Data<AppContext>, path: web::Path<ClientId>) -> impl Responder {
    let id = path.into_inner();

    let response = data.store().get(id);
    format!("{:?}", response)
}

#[get("/leave")]
async fn leave(data: web::Data<AppContext>) -> impl Responder {
    data.mailbox().leave().await.unwrap();
    "OK".to_string()
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog_o!());

    // converts log to slog
    let _log_guard = slog_stdlog::init().unwrap();

    let options = Options::from_args();
    let store = HashStore::new(NodeId(options.node_id));

    let raft = Raft::new(options.raft_addr.clone(), store.clone(), logger.clone());
    let mailbox = Arc::new(raft.mailbox());
    let (raft_handle, mailbox) = match options.peer_addr {
        Some(addr) => {
            info!(logger, "running in follower mode");
            let handle = tokio::spawn(raft.join(addr));
            (handle, mailbox)
        }
        None => {
            info!(logger, "running in leader mode");
            let handle = tokio::spawn(raft.lead());
            (handle, mailbox)
        }
    };

    let logger_server = logger.clone();

    let server = HttpServer::new(move || {
        let app_context = AppContext::new(
            mailbox.clone(),
            store.clone(),
            logger_server.clone(),
            options.node_id,
        );
        App::new()
            .app_data(web::Data::new(app_context))
            .service(create_share)
            .service(get_share)
            .service(leave)
    })
    .bind(options.web_server.clone())
    .unwrap()
    .run();

    let server_handle = server.handle();

    let http_server = tokio::spawn(server);

    let raft_abortable = raft_handle.abort_handle();

    let log_spawn = logger.clone();
    let http_server_shutdown = tokio::spawn(async move {
        let mut sig_int = signal(SignalKind::interrupt()).unwrap();
        let mut sig_term = signal(SignalKind::terminate()).unwrap();
        let mut sig_hup = signal(SignalKind::hangup()).unwrap();
        let cancel = async {
            warn!(
                log_spawn,
                "Shutdown was requested.\nShutting down http server...."
            );
            server_handle.stop(true).await;
            warn!(log_spawn, "Shutting down Consensus Module....");
            raft_abortable.abort();
        };
        tokio::select! {
            _ = sig_int.recv() => cancel.await,
            _ = sig_term.recv() => cancel.await,
            _ = sig_hup.recv() => cancel.await,
        }
    });

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

    info!(logger, "{}", str_log_wellcome);
    info!(
        logger,
        "\n\n ---- Starting Node Id {} ----- \n", options.node_id
    );
    info!(
        logger,
        "\n\n ---- Starting API Server on {} ----- \n", options.web_server
    );
    info!(
        logger,
        "\n\n ---- Starting Consensus Server on {} ----- \n", options.raft_addr
    );

    let result = tokio::try_join!(raft_handle, http_server, http_server_shutdown)?;
    let (raft_result, http_server_result, _) = result;
    raft_result?;
    http_server_result?;
    Ok(())
}
