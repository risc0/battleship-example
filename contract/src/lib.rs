use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;
use near_sdk::collections::UnorderedMap;
use near_sdk::near_bindgen;
use near_sdk::env;

use serde::{Deserialize, Serialize};
use arrayref::array_ref;
use risc0_verify::receipt::Receipt;
use zkvm_core::Digest;

#[derive(Default, Deserialize, Serialize, BorshDeserialize, BorshSerialize)]
pub struct PlayerState {
    id: AccountId,
    board:  [u32; 8],
    shot_x: u32,
    shot_y: u32,
}

#[derive(Default, Deserialize, Serialize, BorshDeserialize, BorshSerialize)]
pub struct GameState {
    // 0 means p1 has setup game, and p2 needs to do setup
    // 1 means p1 needs to process p2's shot and make it's own
    // 2 means p2 needs to process p1's shot and make it's own
    next_turn: u32,  
    p1: PlayerState,
    p2: PlayerState,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct BattleshipContract {
    games: UnorderedMap<String, GameState>,
}

impl Default for BattleshipContract {
    fn default() -> Self {
        BattleshipContract { games: UnorderedMap::<String, GameState>::new(0 as u8) }
    }
}

pub fn verify_receipt(str: &String) -> Vec<u32> {
    let as_bytes = base64::decode(str).unwrap();
    let receipt = bincode::deserialize::<Receipt>(&as_bytes).unwrap();
    receipt.verify();
    receipt.get_journal_u32()
}

#[near_bindgen]
impl BattleshipContract {
    // View state of a game
    pub fn game_state(&self, name: String) -> Option<GameState> {
        self.games.get(&name)
    }

    // Set's p1's initial state
    pub fn new_game(&mut self, name: String, receipt_str: String) {
        // Game must not exist
        assert!(self.games.get(&name).is_none());
        let journal = verify_receipt(&receipt_str);
        let digest = zkvm_serde::from_slice::<Digest>(&journal).unwrap();
        self.games.insert(&name, &GameState { 
            next_turn: 0, 
            p1: PlayerState {
                id: env::signer_account_id(),
                board: *array_ref![digest.as_slice(), 0, 8],
                shot_x: 0,
                shot_y: 0,
            },
            p2: PlayerState::default(),
        });
    }

    /*
    // Set's p2's state, and makes the first shot at p1
    pub fn join_game(name: String, init_proof: String, shot_x: u32, shot_y: u32) {
    }
    // Do a normal turn
    pub fn turn(name: String, update_proof: String, shot_x: u32, shot_y: u32) {
    }
    */
}

