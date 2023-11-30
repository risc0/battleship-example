// Copyright 2022 Risc0, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{prelude::*, EnvFilter};

use base64::{engine::general_purpose::STANDARD, Engine};
use battleship_core::{GameState, RoundParams, RoundResult};
use battleship_methods::{INIT_ELF, TURN_ELF};
use risc0_zkvm::{default_prover, serde::from_slice, ExecutorEnv};

#[derive(Deserialize, Serialize)]
pub struct Receipt {
    journal: Vec<u8>,
    seal: Vec<u32>,
}

#[derive(Deserialize, Serialize)]
pub struct TurnResult {
    state: RoundResult,
    receipt: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,server,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/prove/init", post(prove_init))
        .route("/prove/turn", post(prove_turn))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn do_init_proof(input: GameState) -> Result<String> {
    let prover = default_prover();
    let env = ExecutorEnv::builder().write(&input)?.build()?;
    let receipt = prover.prove_elf(env, INIT_ELF)?;
    Ok(STANDARD.encode(bincode::serialize(&receipt).unwrap()))
}

fn do_turn_proof(input: RoundParams) -> Result<TurnResult> {
    let prover = default_prover();
    let mut output = Vec::new();
    let env = ExecutorEnv::builder()
        .write(&input)?
        .stdout(&mut output)
        .build()?;
    let receipt = prover.prove_elf(env, TURN_ELF)?;
    let result: RoundResult = from_slice(&output)?;
    Ok(TurnResult {
        state: result,
        receipt: STANDARD.encode(bincode::serialize(&receipt).unwrap()),
    })
}

async fn prove_init(Json(payload): Json<GameState>) -> impl IntoResponse {
    let out = match do_init_proof(payload) {
        Ok(receipt) => receipt,
        Err(_e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("bad proof load"),
            )
        }
    };
    (StatusCode::OK, out)
}

async fn prove_turn(Json(payload): Json<RoundParams>) -> impl IntoResponse {
    let out = match do_turn_proof(payload) {
        Ok(receipt) => receipt,
        Err(_e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("bad proof load"),
            )
        }
    };
    (StatusCode::OK, serde_json::to_string(&out).unwrap())
}
