use shared_secret_server::secret::secret::Share;
use slog::{info, slog_o, warn, Drain};

use actix_web::{get, put, web, App, HttpServer, Responder};
use async_trait::async_trait;
use bincode::{deserialize, serialize};
use riteraft::{Mailbox, Raft, Result, Store};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
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
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    Insert { key: ClientId, value: Share },
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ClientId(u64);

impl Deref for ClientId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
struct HashStore(Arc<RwLock<HashMap<ClientId, Share>>>);

impl Deref for HashStore {
    type Target = Arc<RwLock<HashMap<ClientId, Share>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl HashStore {
    fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
    fn get(&self, id: ClientId) -> Option<Share> {
        self.read().unwrap().get(&id).cloned()
    }
}

#[async_trait]
impl Store for HashStore {
    async fn apply(&mut self, message: &[u8]) -> Result<Vec<u8>> {
        let message: Message = deserialize(message).unwrap();
        let message: Vec<u8> = match message {
            Message::Insert { key, value } => {
                let mut db = self.0.write().unwrap();
                db.insert(key.clone(), value.clone());
                log::info!("inserted: ({:?}, {:?})", key, value);
                serialize(&value).unwrap()
            }
        };
        Ok(message)
    }

    async fn snapshot(&self) -> Result<Vec<u8>> {
        Ok(serialize(&self.0.read().unwrap().clone())?)
    }

    async fn restore(&mut self, snapshot: &[u8]) -> Result<()> {
        let new: HashMap<ClientId, Share> = deserialize(snapshot).unwrap();
        let mut db = self.0.write().unwrap();
        let _ = std::mem::replace(&mut *db, new);
        Ok(())
    }
}

struct AppContext {
    mailbox: Arc<Mailbox>,
    store: HashStore,
}

impl AppContext {
    fn new(mailbox: Arc<Mailbox>, store: HashStore) -> Self {
        Self { mailbox, store }
    }

    fn mailbox(&self) -> Arc<Mailbox> {
        self.mailbox.clone()
    }

    fn store(&self) -> HashStore {
        self.store.clone()
    }
}

#[put("/{client_id}/secret")]
async fn create_share(
    data: web::Data<AppContext>,
    path: web::Path<ClientId>,
    share: web::Json<Share>,
) -> impl Responder {
    let client_id = path.into_inner();
    let message = Message::Insert {
        key: client_id,
        value: share.deref().clone(),
    };
    let message = serialize(&message).unwrap();
    let result = data.mailbox().send(message).await.unwrap();
    let result: String = deserialize(&result).unwrap();
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
    let store = HashStore::new();

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

    let server = HttpServer::new(move || {
        let app_context = AppContext {
            mailbox: mailbox.clone(),
            store: store.clone(),
        };
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
