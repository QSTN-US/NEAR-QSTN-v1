use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken, Promise};
use std::collections::HashMap;

type Balance = NearToken;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct CheckInContract {
    check_ins: HashMap<AccountId, (String, u64)>,
    quota: Balance,
    manager: AccountId,
}

impl Default for CheckInContract {
    fn default() -> Self {
        Self {
            check_ins: HashMap::new(),
            quota: NearToken::from_yoctonear(1_000_000_000_000_000_000_000_000), // 1 NEAR in yoctoNEAR
            manager: env::predecessor_account_id(),
        }
    }
}

#[near_bindgen]
impl CheckInContract {
    #[payable]
    pub fn check_in(&mut self, message: String) {
        let deposit: Balance = env::attached_deposit().into();
        assert!(deposit >= self.quota, "Insufficient deposit");

        let account_id: AccountId = env::signer_account_id();
        let timestamp = env::block_timestamp();
        self.check_ins
            .insert(account_id.clone(), (message.clone(), timestamp));
        env::log_str(&format!(
            "Check-in successful for account: {} with message: {}",
            account_id, message
        ));
    }

    pub fn get_check_in(&self, account_id: AccountId) -> Option<(String, u64)> {
        env::log_str(&format!("Fetching check-in for account: {}", account_id));
        self.check_ins.get(&account_id).cloned()
    }

    pub fn set_quota(&mut self, new_quota: Balance) {
        self.assert_manager();
        self.quota = new_quota;
    }

    pub fn withdraw(&mut self, to: AccountId) {
        self.assert_manager();
        let balance: Balance = env::account_balance().into();
        Promise::new(to).transfer(balance.into());
    }

    fn assert_manager(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.manager,
            "Only the manager can call this method"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    fn get_context(predecessor: AccountId, deposit: Balance) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder.attached_deposit(deposit.into());
        builder
    }

    #[test]
    fn test_check_in() {
        let deposit_amount = NearToken::from_yoctonear(1_000_000_000_000_000_000_000_000);
        let context = get_context(accounts(1), deposit_amount);
        testing_env!(context.build());
        let mut contract = CheckInContract::default();

        contract.check_in("Hello, NEAR!".to_string());

        let context = get_context(accounts(0), NearToken::from_yoctonear(0));
        testing_env!(context.build());

        let check_in = contract.get_check_in(accounts(1));
        assert!(
            check_in.is_some(),
            "Check-in not found for account: {:?}",
            accounts(1)
        );
        if let Some((message, _timestamp)) = check_in {
            assert_eq!(message, "Hello, NEAR!");
        }
    }

    #[test]
    #[should_panic(expected = "Insufficient deposit")]
    fn test_check_in_insufficient_deposit() {
        let context = get_context(
            accounts(1),
            NearToken::from_yoctonear(500_000_000_000_000_000_000_000),
        );
        testing_env!(context.build());
        let mut contract = CheckInContract::default();

        contract.check_in("Hello, NEAR!".to_string());
    }

    #[test]
    fn test_set_quota() {
        let context = get_context(accounts(0), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let mut contract = CheckInContract::default();

        contract.set_quota(NearToken::from_yoctonear(2_000_000_000_000_000_000_000_000));
        assert_eq!(
            contract.quota,
            NearToken::from_yoctonear(2_000_000_000_000_000_000_000_000)
        );
    }

    #[test]
    #[should_panic(expected = "Only the manager can call this method")]
    fn test_set_quota_not_manager() {
        let context = get_context(accounts(1), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let mut contract = CheckInContract::default();

        testing_env!(get_context(accounts(2), NearToken::from_yoctonear(0)).build());
        contract.set_quota(NearToken::from_yoctonear(2_000_000_000_000_000_000_000_000));
    }

    #[test]
    fn test_withdraw() {
        let context = get_context(accounts(0), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let mut contract = CheckInContract::default();

        contract.withdraw(accounts(1));
    }

    #[test]
    #[should_panic(expected = "Only the manager can call this method")]
    fn test_withdraw_not_manager() {
        let context = get_context(accounts(1), NearToken::from_yoctonear(0));
        testing_env!(context.build());
        let mut contract = CheckInContract::default();

        testing_env!(get_context(accounts(2), NearToken::from_yoctonear(0)).build());
        contract.withdraw(accounts(2));
    }
}
