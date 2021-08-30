//! This contract implements simple counter backed by storage on blockchain.
//!
//! The contract provides methods to [tip users][send_tip].
//!
//! [send_tip]: struct.Contract.html#method.send_tip

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    collections::{Vector},
    env,
    json_types::U128,
    near_bindgen,
    AccountId,
    Promise,
};

#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc = near_sdk::wee_alloc::WeeAlloc::INIT;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub receiver_id: AccountId,
    pub tip_amount: u128,
    pub registered_receivers: Vector<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(receiver_id: AccountId, tip_amount: u128) -> Self {
        assert!(env::is_valid_account_id(receiver_id.as_bytes()), "Invalid receiver account");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            receiver_id,
            tip_amount,
            registered_receivers: Vector::new(b"registered_receivers".to_vec()),
        }
    }

    pub fn is_registered(&self) -> bool {
        let receiver_id = self.receiver_id.clone();
        self.registered_receivers.iter().any(|e| e == receiver_id)
    }

    pub fn register_receiver(&mut self) {
        if !self.is_registered() {
            self.registered_receivers.push(&self.receiver_id);
        }
    }

    pub fn get_balance(&self) -> U128 {
        U128(env::account_balance())
    }

    #[payable]
    pub fn send_tip(&mut self) {
        let receiver_id = self.receiver_id.clone();
        assert!(u128::from(self.get_balance()) >= self.tip_amount, "Insufficient funds");
        assert!(self.is_registered());
        Promise::new(receiver_id).transfer(self.tip_amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.testnet".to_string()
    }
    fn bob() -> AccountId {
        "bob.testnet".to_string()
    }

    fn get_context(predecessor_account_id: String, storage_usage: u64) -> VMContext {
        VMContext {
            current_account_id: bob(), // Recipient of the transaction
            signer_account_id: alice(), // Originator of the transaction
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn try_send_tip() {
        let mut context = get_context(alice(), 0);
        const AMOUNT_TO_SEND: u128 = 1_000_000_000_000_000_000_000_000;
        context.account_balance = AMOUNT_TO_SEND;
        testing_env!(context.clone());
        let mut contract = Contract::new(bob(), AMOUNT_TO_SEND);
        contract.register_receiver();
        contract.send_tip();
        assert_eq!(contract.get_balance(), U128(0), "Account balance should be liquidated.");
    }

    #[test]
    fn try_get_balance() {
        let mut context = get_context(alice(), 0);
        const AMOUNT_TO_SEND: u128 = 1_000_000_000_000_000_000_000_000;
        context.account_balance = AMOUNT_TO_SEND;
        testing_env!(context.clone());
        let mut contract = Contract::new(bob(), AMOUNT_TO_SEND);
        contract.register_receiver();
        assert_eq!(contract.get_balance(), U128(AMOUNT_TO_SEND), "Account balance should be equal to initial balance.");
    }
}