use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;

use arrayref::array_ref;
use base64::decode;
use risc0_verify::risc0_circuit::Risc0Circuit;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct VerifyContract {
    verify_count : u32,
}

#[near_bindgen]
impl VerifyContract {
    pub fn verify(seal_str: String) {
        let seal = decode(seal_str).unwrap();
        assert!(seal.len() % 4 == 0);
        let mut proof: Vec<u32> = vec![];
        for i in 0..(seal.len() / 4) {
            proof.push(u32::from_le_bytes(*array_ref![&seal, i * 4, 4]));
        }
        let mut circuit: Risc0Circuit = Risc0Circuit::default();
        risc0_verify::verify::verify(&mut circuit, &proof);
    }
}

