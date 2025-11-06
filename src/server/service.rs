use std::time::Duration;

use axum::extract::ws::Message;
use tokio::time::Instant;

use crate::actor::player::{PlayerSnapshot, Players, ServerMessage};

pub async fn game_loop(players: Players) {
    let tick_rate = 30u64;
    let tick_duration = Duration::from_millis(1000 / tick_rate);
    let mut tick = 0u64;

    loop {
        let now = Instant::now();
        tick += 1;
        // {
        //     let pls_map = players.read().await;
        //     for player in pls_map.values() {
        //         if let Some(input) = *player.latest_input.read() {
        //             let turn_speed = 2.0;
        //             player.angle += input.turn * (turn_speed / tick as f32);

        //             let accel = 5.0;
        //             player.vel += input.throttle * (accel / tick as f32);

        //             if player.vel > 10.0 {
        //                 player.vel = 10.0;
        //             }
        //             if player.vel < -5.0 {
        //                 player.vel = -5.0
        //             }

        //             player.x += player.vel * player.angle.cos() / tick_rate as f32;
        //             player.y += player.vel * player.angle.sin() / tick_rate as f32;
        //         } else {
        //             player.vel *= 0.98;
        //         }
        //     }
        // }

        let snapshot = {
            let pls_map = players.read().await;
            let mut vec: Vec<PlayerSnapshot> = Vec::with_capacity(pls_map.len());
            for p in pls_map.values() {
                vec.push(PlayerSnapshot {
                    id: p.id.to_string(),
                    x: p.x,
                    y: p.y,
                    angle: p.angle,
                    hp: p.hp,
                });
            }
            ServerMessage::Snapshot { tick, players: vec }
        };
        let txt = serde_json::to_string(&snapshot).unwrap();
        {
            let pls_map = players.read().await;
            for p in pls_map.values() {
                let _ = p.tx.send(Message::Text(txt.clone()));
            }
        }

        let elapsed = now.elapsed();
        if elapsed < tick_duration {
            tokio::time::sleep(tick_duration - elapsed).await
        }
    }
}
