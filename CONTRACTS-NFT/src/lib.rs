use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::serde_json;
use near_sdk::store::LookupMap;
use near_sdk::{env, log, near, AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseError};

const NFT_WASM_CODE: &[u8] = include_bytes!("./nft/non_fungible_token.wasm");
const TGAS: Gas = Gas::from_tgas(1); // 10e12yⓃ
const NO_DEPOSIT: NearToken = NearToken::from_near(0); // 0yⓃ

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Quizzler {
    owner: AccountId,
    gas_station: AccountId,
    managers: LookupMap<AccountId, bool>,
    surveys: LookupMap<String, Survey>,
    surveys_users_rewarded: LookupMap<String, LookupMap<AccountId, bool>>,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Survey {
    survey_creator: AccountId,
    participants_limit: u64,
    nft_contract_id: AccountId,
    participants_rewarded: u64,
    is_canceled: bool,
}

#[near(serializers = [json, borsh])]
struct NFTInitArgs {
    owner_id: AccountId,
}

#[near(serializers = [json, borsh])]
pub struct TokenArgs {
    owner_id: AccountId,
    metadata: NFTContractMetadata,
}

#[near(serializers = [json, borsh])]
pub struct MintRequiredArgs {
    gas_fee: NearToken,
    mint_fee: NearToken,
    common_fee: NearToken,
}

#[near(serializers = [json, borsh])]
pub struct MintArgs {
    token_id: TokenId,
    receiver_id: AccountId,
    token_metadata: TokenMetadata,
}

#[near]
impl Quizzler {
    #[init]
    pub fn new(gas_station: AccountId) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");

        let predecessor = env::predecessor_account_id();
        let mut managers = LookupMap::new(b"m");
        managers.insert(predecessor.clone(), true);

        Self {
            owner: predecessor,
            gas_station,
            managers: managers,
            surveys: LookupMap::new(b"s"),
            surveys_users_rewarded: LookupMap::new(b"surveys_users_rewarded".to_vec()),
        }
    }

    pub fn set_manager(&mut self, manager: AccountId, status: bool) {
        self.assert_owner();
        self.managers.insert(manager, status);
    }

    pub fn is_manager(&self, manager: AccountId) -> bool {
        self.managers.get(&manager).unwrap_or(&false).clone()
    }

    pub fn set_gas_station(&mut self, gas_station: AccountId) {
        self.assert_owner();
        self.gas_station = gas_station;
    }

    pub fn get_gas_station(&self) -> AccountId {
        self.gas_station.clone()
    }

    pub fn get_required_deposit(&self, participants_limit: u64) -> MintRequiredArgs {
        assert!(
            participants_limit > 0,
            "Participants limit must be greater than 0"
        );

        let fee_needed = NearToken::from_yoctonear(15 * 10u128.pow(21))
            .saturating_mul(participants_limit.clone() as u128);

        let code = NFT_WASM_CODE.to_vec();
        let minimum_needed = NearToken::from_yoctonear(
            env::storage_byte_cost()
                .saturating_mul(code.len() as u128)
                .as_yoctonear()
                + 5 * 10u128.pow(23),
        );

        let required_deposit =
            NearToken::from_yoctonear(fee_needed.as_yoctonear() + minimum_needed.as_yoctonear());

        let required_data = MintRequiredArgs {
            gas_fee: fee_needed,
            mint_fee: minimum_needed,
            common_fee: required_deposit,
        };

        return required_data;
    }

    #[payable]
    pub fn create_survey(
        &mut self,
        survey_id: String,
        participants_limit: u64,
        gas_fee: NearToken,
        metadata: NFTContractMetadata,
    ) -> Promise {
        assert!(
            participants_limit > 0,
            "Participants limit must be greater than 0"
        );

        let attached_deposit = env::attached_deposit();
        let current_account = env::current_account_id().to_string();

        metadata.assert_valid();

        let args = TokenArgs {
            owner_id: env::current_account_id(),
            metadata,
        };

        let fee_needed = NearToken::from_yoctonear(15 * 10u128.pow(21))
            .saturating_mul(participants_limit.clone() as u128);
        assert!(
            gas_fee >= fee_needed,
            "Gas fee is not sufficient. Required: {}, Attached: {}",
            fee_needed,
            gas_fee
        );

        let code = NFT_WASM_CODE.to_vec();
        let minimum_needed = NearToken::from_yoctonear(
            env::storage_byte_cost()
                .saturating_mul(code.len() as u128)
                .as_yoctonear()
                + 5 * 10u128.pow(23),
        );

        let required_deposit =
            NearToken::from_yoctonear(gas_fee.as_yoctonear() + minimum_needed.as_yoctonear());

        assert!(
            attached_deposit >= required_deposit,
            "Attached deposit is not sufficient. Required: {}, Attached: {}",
            required_deposit,
            attached_deposit
        );

        let new_nft_contract_account_id: AccountId =
            format!("{survey_id}.{current_account}").parse().unwrap();

        assert!(
            env::is_valid_account_id(new_nft_contract_account_id.as_bytes()),
            "Invalid subaccount"
        );

        log!("Creating new NFT contract: {}", minimum_needed.clone());
        Promise::new(new_nft_contract_account_id.clone())
            .create_account()
            .transfer(minimum_needed)
            .deploy_contract(code)
            .function_call(
                "new".to_owned(),
                serde_json::to_vec(&args).unwrap(),
                NO_DEPOSIT,
                TGAS.saturating_mul(5),
            )
            .then(Self::ext(env::current_account_id()).deploy_callback(
                survey_id,
                new_nft_contract_account_id.clone(),
                env::predecessor_account_id(),
                participants_limit,
                gas_fee,
                attached_deposit,
            ))
    }

    #[payable]
    pub fn reward_participant(
        &mut self,
        survey_id: String,
        participant: AccountId,
        metadata: TokenMetadata,
    ) -> Promise {
        self.assert_manager();

        let attached_deposit = env::attached_deposit();

        let survey = self
            .surveys
            .get_mut(&survey_id)
            .expect("Survey does not exist");
        assert!(!survey.is_canceled, "Survey is canceled");

        let prefix = format!("{}-r", survey_id.clone()).into_bytes();
        let rewarded = self
            .surveys_users_rewarded
            .entry(survey_id.clone())
            .or_insert_with(|| LookupMap::new(prefix));

        if !rewarded.contains_key(&participant) {
            assert!(
                survey.participants_rewarded < survey.participants_limit,
                "Participant limit reached"
            );

            let args = MintArgs {
                token_id: TokenId::from(survey.participants_rewarded.to_string()),
                receiver_id: participant.clone(),
                token_metadata: metadata,
            };

            Promise::new(survey.nft_contract_id.clone())
                .function_call(
                    "nft_mint".to_owned(),
                    serde_json::to_vec(&args).unwrap(),
                    attached_deposit.clone(),
                    TGAS.saturating_mul(5),
                )
                .then(Self::ext(env::current_account_id()).mint_callback(
                    survey_id,
                    TokenId::from(survey.participants_rewarded.to_string()),
                    participant.clone(),
                    attached_deposit,
                ))
        } else {
            panic!("Participant already rewarded");
        }
    }

    pub fn cancel_survey(&mut self, survey_id: String) {
        let survey_creator = {
            let survey = self.surveys.get(&survey_id).expect("Survey does not exist");
            survey.survey_creator.clone()
        };
        self.creator_or_manager(&survey_creator);

        let survey = self
            .surveys
            .get_mut(&survey_id)
            .expect("Survey does not exist");

        assert!(!survey.is_canceled, "Survey is canceled");
        assert!(
            survey.participants_limit > survey.participants_rewarded,
            "Survey is finished"
        );

        survey.is_canceled = true;
    }

    pub fn emergency_withdraw(&mut self, amount: NearToken, account_id: AccountId) {
        self.assert_owner();
        assert!(env::account_balance() >= amount, "Not enough balance");
        Promise::new(account_id).transfer(amount);
    }

    #[private]
    pub fn deploy_callback(
        &mut self,
        survey_id: String,
        contract_id: AccountId,
        user: AccountId,
        participants_limit: u64,
        gas_fee: NearToken,
        attached: NearToken,
        #[callback_result] create_deploy_result: Result<(), PromiseError>,
    ) -> bool {
        if let Ok(_result) = create_deploy_result {
            let survey = Survey {
                survey_creator: user.clone(),
                participants_limit,
                nft_contract_id: contract_id.clone(),
                participants_rewarded: 0,
                is_canceled: false,
            };
            self.surveys.insert(survey_id.clone(), survey);

            Promise::new(self.gas_station.clone()).transfer(gas_fee);

            log!("Correctly created and deployed to {}", contract_id);
            log!("survey_id: {}", survey_id);
            log!("participants_limit: {}", participants_limit);
            log!("gas_fee: {}", gas_fee);
            log!("survey_creator: {}", user);

            return true;
        };

        log!(
            "Error creating {}, returning {}yⓃ to {}",
            contract_id,
            attached,
            user
        );
        Promise::new(user).transfer(attached);
        false
    }

    #[private]
    pub fn mint_callback(
        &mut self,
        survey_id: String,
        token_id: TokenId,
        participant: AccountId,
        attached: NearToken,
        #[callback_result] mint_result: Result<Token, PromiseError>,
    ) -> bool {
        if let Ok(_result) = mint_result {
            let survey = self
                .surveys
                .get_mut(&survey_id)
                .expect("Survey does not exist");

            let prefix = format!("{}-r", survey_id.clone()).into_bytes();
            let rewarded = self
                .surveys_users_rewarded
                .entry(survey_id.clone())
                .or_insert_with(|| LookupMap::new(prefix));

            survey.participants_rewarded += 1;
            rewarded.insert(participant.clone(), true);

            log!("Minting successful");
            log!("survey_id: {}", survey_id);
            log!("participant: {}", participant);
            log!("token_id: {}", token_id);

            return true;
        };

        log!("Minting error");
        log!("survey_id: {}", survey_id);
        log!("participant: {}", participant);
        Promise::new(self.gas_station.clone()).transfer(attached);
        false
    }

    pub fn get_survey(&self, survey_id: String) -> Survey {
        self.surveys
            .get(&survey_id)
            .expect("Survey does not exist")
            .clone()
    }

    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Only the owner can call this method"
        );
    }

    fn assert_manager(&self) {
        assert!(
            *self
                .managers
                .get(&env::predecessor_account_id())
                .unwrap_or(&false),
            "Only a manager can call this method"
        );
    }

    fn creator_or_manager(&self, survey_creator: &AccountId) {
        assert!(
            env::predecessor_account_id() == *survey_creator
                || *self
                    .managers
                    .get(&env::predecessor_account_id())
                    .unwrap_or(&false),
            "Only the survey creator or a manager can call this method"
        );
    }
}
