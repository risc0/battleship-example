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

use std::{collections::HashMap, rc::Rc};

use gloo::{dialogs::alert, timers::future::TimeoutFuture, storage::LocalStorage, storage::Storage};
use serde_with::serde_as;
use rand::{thread_rng, Rng};
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged, Dispatched, Dispatcher};

use crate::{
    bus::EventBus,
    contract::{Contract, ContractState},
    near::NearContract,
    wallet::WalletContext,
};
use battleship_core::{
    GameCheck, GameState, Position, RoundParams, RoundResult, Ship, ShipDirection, BOARD_SIZE,
    SHIP_SPANS,
};

pub type CoreHitType = battleship_core::HitType;

const WAIT_TURN_INTERVAL: u32 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Local,
    Remote,
}

#[derive(Deserialize, Serialize)]
pub struct TurnResult {
    state: RoundResult,
    receipt: String,
}

#[derive(Clone, PartialEq)]
pub enum GameMsg {
    Init,
    Shot(Position),
    WaitTurn,
    CheckTurn,
    SaveAndWait,
    ProcessTurn(ContractState),
    UpdateState(String, RoundResult, Position),
    Resume,
    Error(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Hash)]
pub enum HitType {
    Core(CoreHitType),
    Pending,
}

#[serde_as]
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub name: String,
    pub state: GameState,
    #[serde_as(as = "Vec<(_, _)>")]
    pub local_shots: HashMap<Position, HitType>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub remote_shots: HashMap<Position, HitType>,
    pub last_receipt: String,
    pub last_shot: Option<Position>,
    pub is_first: bool,
    pub status: String,
    pub og_until: usize,
    pub turn_processed: bool,
}

fn create_random_ships() -> [Ship; 5] {
    // randomly place 5 ships on the board
    let mut rng = thread_rng();
    let mut game_check = GameCheck::new();

    let ships: [Ship; 5] = array_init::array_init(|i| {
        loop {
            // pick a random starting point on the board
            let x = rng.gen_range(0..BOARD_SIZE - 1);
            let y = rng.gen_range(0..BOARD_SIZE - 1);

            // pick between 0 and 1 for randomized ship placement
            let dir = if rng.gen::<bool>() {
                ShipDirection::Horizontal
            } else {
                ShipDirection::Vertical
            };

            let ship = Ship::new(x as u32, y as u32, dir);

            // does it fit on the board
            let span = SHIP_SPANS[i];
            if !ship.check(span) {
                continue;
            }

            // does it cross any other ship
            if !game_check.check(&ship, span, false) {
                continue;
            }

            // mark the ship as taken
            game_check.commit(&ship, span);

            return ship;
        }
    });
    ships
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
    pub until: usize,
    #[prop_or_else(create_random_ships)]
    pub ships: [Ship; 5],
    #[prop_or_default]
    pub children: Children,
}

pub struct GameProvider {
    _bridge: Box<dyn Bridge<EventBus<GameMsg>>>,
    journal: Dispatcher<EventBus<String>>,
    game: GameSession,
    contract: Rc<NearContract>,
}

impl Component for GameProvider {
    type Message = GameMsg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (wallet, _) = ctx
            .link()
            .context::<WalletContext>(Callback::noop())
            .unwrap();

        // check by 'name' for an existing game session in local storage
        let res: Result<GameSession, _> = LocalStorage::get(ctx.props().name.clone());

        // if we have a game session, set game equal to it or create a new game session
        let (game_session_exists, game) = match res {
            Ok(game) => (true,game),
            Err(_) => (false, GameSession {
                name: ctx.props().name.clone(),
                state: GameState {
                    ships: ctx.props().ships.clone(),
                    salt: 0xDEADBEEF,
                },
                local_shots: HashMap::new(),
                remote_shots: HashMap::new(),
                last_receipt: String::new(),
                last_shot: None,
                is_first: ctx.props().until == 2,
                status: format!("Ready!"),
                og_until: ctx.props().until,
                turn_processed: false,
            })
        };

        // if a game session exists, check who's turn it is and resume the game
        if game_session_exists {
            log::info!("Game session exists, checking who's turn it is");
            ctx.link().send_message(GameMsg::CheckTurn);
        } else {
            // if the game session does not exist, initialize the game if player1
            if ctx.props().until == 1 {
                log::info!("Game session does not exist, initializing game");
                ctx.link().send_message(GameMsg::Init);
            }  
        }

        let contract = wallet.contract.clone();
        GameProvider {
            _bridge: EventBus::bridge(ctx.link().callback(|msg| msg)),
            journal: EventBus::dispatcher(),
            game,
            contract,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GameMsg::Init => {
                self.game.status = format!("Init");
                let game = self.game.clone();
                let body = serde_json::to_string(&game.state).unwrap();
                let contract = self.contract.clone();
                ctx.link().send_future(async move {
                    let response = match Request::post("/prove/init")
                        .header("Content-Type", "application/json")
                        .body(body)
                        .send()
                        .await
                    {
                        Ok(response) => response,
                        Err(err) => {
                            return GameMsg::Error(format!("POST /prove/init failed: {}", err));
                        }
                    };
                    let receipt = match response.text().await {
                        Ok(receipt) => receipt,
                        Err(err) => {
                            return GameMsg::Error(format!("receipt: {}", err));
                        }
                    };
                    match contract.new_game(&game.name, &receipt).await {
                        Ok(()) => {
                            log::info!("Game created, save and wait turn {}", game.name);
                            GameMsg::SaveAndWait
                        }
                        Err(err) => GameMsg::Error(format!("new_game: {:?}", err)),
                    }
                });
                true
            }
            GameMsg::Shot(pos) => {
                if self.game.status == "Ready!" {
                    self.game.status = format!("Shot: {}", pos);
                    self.journal.send("GameMsg::Shot".into());
                    self.game.last_shot = Some(pos.clone());
                    self.game.remote_shots.insert(pos.clone(), HitType::Pending);
                    let game = self.game.clone();
                    let contract = self.contract.clone();
                    let is_first = self.game.is_first;
                    self.game.is_first = false;
                    ctx.link().send_future(async move {
                        if is_first {
                            let body = serde_json::to_string(&game.state).unwrap();
                            let response = match Request::post("/prove/init")
                                .header("Content-Type", "application/json")
                                .body(body)
                                .send()
                                .await
                            {
                                Ok(response) => response,
                                Err(err) => {
                                    return GameMsg::Error(format!("POST /prove/init: {}", err));
                                }
                            };
                            let receipt = match response.text().await {
                                Ok(receipt) => receipt,
                                Err(err) => {
                                    return GameMsg::Error(format!("receipt: {}", err));
                                }
                            };
                            match contract.join_game(&game.name, &receipt, pos.x, pos.y)
                                .await
                            {
                                Ok(()) => {
                                    log::info!("Game joined save and wait turn {}", game.name);
                                    GameMsg::SaveAndWait
                                }
                                Err(err) => {
                                    return GameMsg::Error(format!("join_game: {:?}", err));
                                }
                            }
                        } else {
                            match contract.turn(&game.name, &game.last_receipt, pos.x, pos.y)
                                .await
                            {
                                Ok(()) => {
                                    log::info!("Turn sent save and and wait turn {}", game.name);
                                    GameMsg::SaveAndWait
                                }
                                Err(err) => {
                                    return GameMsg::Error(format!("turn: {:?}", err));
                                }
                            }
                        }
                    });
                    true
                } else {
                    alert("Waiting for other player!");
                    false
                }
            }
            GameMsg::WaitTurn => {
                self.game.status = format!("Waiting for other player.");
                self.journal.send("GameMsg::WaitTurn".into());
                let until = self.game.og_until as u32; //ctx.props().until as u32;
                let game = self.game.clone();
                let contract = self.contract.clone();
                ctx.link().send_future(async move {
                    let contract_state = match contract.get_state(&game.name).await {
                        Ok(state) => state,
                        Err(err) => {
                            return GameMsg::Error(format!("get_state: {:?}", err));
                        }
                    };
                    if contract_state.next_turn == until {
                        GameMsg::ProcessTurn(contract_state)
                    } else {
                        TimeoutFuture::new(WAIT_TURN_INTERVAL).await;
                        GameMsg::WaitTurn
                    }
                });
                true
            }
            GameMsg::ProcessTurn(contract_state) => {
                self.game.status = format!("ProcessTurn");
                self.journal.send("GameMsg::ProcessTurn".into());
                let state = self.game.state.clone();
                if let Some(last_shot) = self.game.last_shot.clone() {
                    self.game.remote_shots.insert(
                        last_shot,
                        match contract_state.last_hit.unwrap() {
                            0 => HitType::Core(CoreHitType::Miss),
                            1 => HitType::Core(CoreHitType::Hit),
                            2 => {
                                alert("You sunk an opponent's ship!");
                                HitType::Core(CoreHitType::Sunk(contract_state.sunk_what.unwrap()))
                            }
                            _ => unreachable!(),
                        },
                    );
                }
                let until = self.game.og_until; //ctx.props().until;
                ctx.link().send_future(async move {
                    let player = if until == 2 {
                        contract_state.p1
                    } else {
                        contract_state.p2
                    };
                    let shot = Position::new(player.shot_x, player.shot_y);
                    let params = RoundParams {
                        state: state.clone(),
                        shot: shot.clone(),
                    };
                    let body = serde_json::to_string(&params).unwrap();
                    let response = match Request::post("/prove/turn")
                        .header("Content-Type", "application/json")
                        .body(body)
                        .send()
                        .await
                    {
                        Ok(response) => response,
                        Err(err) => {
                            return GameMsg::Error(format!("POST /prove/turn: {}", err));
                        }
                    };
                    let result = match response.text().await {
                        Ok(result) => result,
                        Err(err) => {
                            return GameMsg::Error(format!("result: {}", err));
                        }
                    };
                    match serde_json::from_str::<TurnResult>(&result) {
                        Ok(result) => GameMsg::UpdateState(result.receipt, result.state, shot),
                        Err(err) => GameMsg::Error(format!("json fail: {}", err)),
                    }
                });
                true
            }
            GameMsg::CheckTurn => {
                let until = self.game.og_until as u32; //ctx.props().until as u32;
                let game = self.game.clone();
                let contract = self.contract.clone();
                let turn_processed = self.game.turn_processed;
                ctx.link().send_future(async move {
                    let contract_state = match contract.get_state(&game.name).await {
                        Ok(state) => state,
                        Err(err) => {
                            return GameMsg::Error(format!("checkturn get_state: {:?}", err));
                        }
                    };
                    if contract_state.next_turn == until {
                        // process the turn if it was not processed yet
                        if !turn_processed { GameMsg::ProcessTurn(contract_state) 
                        } else {
                            GameMsg::Resume
                        }
                    } else {
                        TimeoutFuture::new(WAIT_TURN_INTERVAL).await;
                        GameMsg::WaitTurn
                    }
                });
                true
            }
            GameMsg::UpdateState(receipt, state, shot) => {
                self.game.status = format!("Ready!");
                self.journal.send("GameMsg::UpdateState".into());
                self.game.state = state.state;
                self.game.last_receipt = receipt;
                self.game.local_shots.insert(shot, HitType::Core(state.hit));
                self.game.turn_processed = true;
                LocalStorage::set(self.game.name.clone(), self.game.clone()).unwrap();
                true
            }
            GameMsg::SaveAndWait => {
                log::info!("GameMsg::SaveAndWait {},", self.game.name);
                self.game.status = format!("Waiting for other player.");
                self.game.turn_processed = false;
                LocalStorage::set(self.game.name.clone(), self.game.clone()).unwrap();
                ctx.link().send_message(GameMsg::WaitTurn);
                true
            }
            GameMsg::Resume => {
                log::info!("GameMsg::Resume {},", self.game.name);
                self.game.status = format!("Ready!");
                true
            }
            GameMsg::Error(msg) => {
                self.game.status = msg;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<GameSession> context={self.game.clone()}>
                {ctx.props().children.clone()}
            </ContextProvider<GameSession>>
        }
    }
}
