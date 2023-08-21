#![feature(lazy_cell)]

use std::sync::LazyLock;

use chrono::Utc;
use hardlight::{*, rkyv::{to_bytes, from_bytes}};
use sled::{Db, transaction::abort};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let config = ServerConfig::new_self_signed("localhost:8080");
    let mut server = Server::new(config, factory!(Handler));
    let _event_emitter = server.get_event_emitter();
    let mut topic_notifier = server.get_topic_notifier().unwrap();
    
    tokio::spawn(
        async move {
            while let Some(notif) = topic_notifier.recv().await {
                match notif {
                    TopicNotification::Created(topic) => {
                        info!("Topic created: {:?}", topic);
                    }
                    TopicNotification::Removed(topic) => {
                        info!("Topic removed: {:?}", topic);
                    }
                }
            }
        }
    );
    
    server.run().await.unwrap()
}

static DB: LazyLock<Db> = LazyLock::new(|| {
    info!("Opening database at cr.db");
    let db = sled::open("cr.db").unwrap();
    info!("Database opened");
    db
});

#[rpc]
pub trait CRServer {
    async fn setup(&self, name: String) -> HandlerResult<Result<(), Error>>;
    async fn create_game(&self) -> HandlerResult<ServerResult<GameID>>;
    async fn list_games(&self) -> HandlerResult<Vec<GameID>>;
    async fn join_game(&self, game_id: GameID) -> HandlerResult<ServerResult<()>>;
    async fn update_x_position(&self, x: f32) -> HandlerResult<ServerResult<()>>;
    async fn shoot(&self) -> HandlerResult<ServerResult<()>>;
}

#[connection_state]
pub struct State {
    name: Option<String>,
    game_id: Option<GameID>,
    current_x: f32,
}

#[codable]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GameID([u8; 16]);

impl GameID {
    pub fn new() -> Self {
        Self(rand::random())
    }
}

#[rpc_handler]
impl CRServer for Handler {
    async fn setup(&self, name: String) -> HandlerResult<Result<(), Error>> {
        if name.len() > 32 {
            return Ok(Err(Error::NameTooLong));
        }
        if name.len() == 0 {
            return Ok(Err(Error::NameTooShort));
        }
        if name.contains(|c: char| !c.is_ascii_alphanumeric()) {
            return Ok(Err(Error::NameInvalid));
        }
        
        let mut key = b"name-".to_vec();
        key.extend_from_slice(name.as_bytes());
        
        if DB.contains_key(&key).unwrap() {
            return Ok(Err(Error::NameTaken));
        }
        
        DB.insert(&key, b"").unwrap();
        
        Ok(Ok(()))
    }
    
    async fn create_game(&self) -> HandlerResult<ServerResult<GameID>> {
        let game_id = GameID::new();
        let name = match self.state.read().await.name.clone() {
            Some(name) => name,
            None => return Ok(Err(Error::NameNotSet)),
        };
        let mut key = b"game-".to_vec();
        key.extend_from_slice(&game_id.0);
        let val = to_bytes::<_, 1024>(&vec![name]).unwrap().to_vec();
        DB.insert(key, val).unwrap();
        Ok(Ok(game_id))
    }
    
    async fn list_games(&self) -> HandlerResult<Vec<GameID>> {
        Ok(DB.scan_prefix(b"game-")
            .map(|res| {
                let (key, _) = res.unwrap();
                let mut game_id = [0u8; 16];
                game_id.copy_from_slice(&key[5..]);
                GameID(game_id)
            })
            .collect())
    }
    
    async fn join_game(&self, game_id: GameID) -> HandlerResult<ServerResult<()>> {
        let state = self.state.read().await;
        
        if state.game_id.is_some() {
            return Ok(Err(Error::AlreadyInGame));
        }
        
        let name = match state.name.clone() {
            Some(name) => name,
            None => return Ok(Err(Error::NameNotSet)),
        };
        
        drop(state);
        
        self.subscriptions.add(&game_id.0.to_vec().into());
        
        match DB.transaction(|tx_db| {
            let mut key = b"game-".to_vec();
            key.extend_from_slice(&game_id.0);
            
            let mut prev = match tx_db.get(key.clone())? {
                Some(prev) => from_bytes::<Vec<String>>(&prev).unwrap(),
                None => abort("prev game does not exist")?,
            };
            
            prev.push(name.clone());
            
            let val = to_bytes::<_, 1024>(&prev).unwrap().to_vec();
            tx_db.insert(key, val)?;
            
            Ok(())
        }) {
            Ok(_) => Ok(Ok(())),
            Err(_) => Ok(Err(Error::GameNotSet)),
        }
    }
    
    async fn update_x_position(&self, x: f32) -> HandlerResult<ServerResult<()>> {
        let state = self.state.read().await;
        let name = match state.name.clone() {
            Some(name) => name,
            None => return Ok(Err(Error::NameNotSet)),
        };
        let game_id = match state.game_id.clone() {
            Some(game_id) => game_id,
            None => return Ok(Err(Error::GameNotSet)),
        };
        drop(state);
        
        let key = create_x_pos_key(&name, &game_id);
        
        self.state.write().await.current_x = x;
        
        let pos = XPosition::new(x);
        let val = to_bytes::<_, 1024>(&pos).unwrap().to_vec();
        DB.insert(key, val).unwrap();
        
        Ok(Ok(()))
    }
    
    async fn shoot(&self) -> HandlerResult<ServerResult<()>> {
        let state = self.state.read().await;
        let game_id = match state.game_id.clone() {
            Some(game_id) => game_id,
            None => return Ok(Err(Error::GameNotSet)),
        };
        self.events.emit(&game_id.0.to_vec().into(), Event::Laser { x: state.current_x }).await;
        Ok(Ok(()))
    }
}

fn create_x_pos_key(name: &String, game_id: &GameID) -> Vec<u8> {
    let mut key = b"pos-".to_vec();
    key.extend_from_slice(name.as_bytes());
    key.extend_from_slice(b"-");
    key.extend_from_slice(&game_id.0);
    key
}

#[codable]
#[derive(Debug, Clone)]
enum Event {
    Laser {
        x: f32,
    }
}

#[codable]
struct XPosition {
    x: f32,
    timestamp: u64,
}

impl XPosition {
    fn new(x: f32) -> Self {
        Self {
            x,
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}

#[codable]
pub enum Error {
    NameTaken,
    NameTooLong,
    NameTooShort,
    NameInvalid,
    NameNotSet,
    GameNotSet,
    AlreadyInGame,
}

pub type ServerResult<T> = Result<T, Error>;