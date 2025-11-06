use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct InputState {
    pub throttle: f32,
    pub turn: f32,
    pub fire: bool,
}

pub struct Player {
    pub id: Uuid,
    pub tx: mpsc::UnboundedSender<Message>,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub vel: f32,
    pub hp: i32,
    pub latest_input: RwLock<Option<InputState>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerSnapshot {
    id: String,
    x: f32,
    y: f32,
    angle: f32,
    hp: i32,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "snapshot")]
    Snapshot {
        tick: u64,
        players: Vec<PlayerSnapshot>,
    },
    #[serde(rename = "welcome")]
    Welcome { id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "input")]
    Input {
        seq: u64,
        throttle: f32,
        turn: f32,
        fire: bool,
    },
}

pub type Players = Arc<RwLock<HashMap<Uuid, Arc<Player>>>>;
