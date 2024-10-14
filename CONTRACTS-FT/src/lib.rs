use near_sdk::store::LookupMap;
use near_sdk::{env, log, near, AccountId, NearToken, PanicOnDefault, Promise};

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
    reward_amount: NearToken,
    participants_rewarded: u64,
    is_canceled: bool,
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

    #[payable]
    pub fn create_survey(
        &mut self,
        survey_id: String,
        participants_limit: u64,
        reward_amount: NearToken,
        gas_fee: NearToken,
    ) {
        let attached_deposit = env::attached_deposit();

        let fee_needed = NearToken::from_yoctonear(15 * 10u128.pow(21))
            .saturating_mul(participants_limit.clone() as u128);
        assert!(
            gas_fee >= fee_needed,
            "Gas fee is not sufficient. Required: {}, Attached: {}",
            fee_needed,
            gas_fee
        );

        let required_deposit =
            (reward_amount.as_yoctonear() * participants_limit as u128) + gas_fee.as_yoctonear();

        assert!(
            attached_deposit >= NearToken::from_yoctonear(required_deposit),
            "Attached deposit is not sufficient. Required: {}, Attached: {}",
            required_deposit,
            attached_deposit
        );

        let survey = Survey {
            survey_creator: env::predecessor_account_id(),
            participants_limit,
            reward_amount: reward_amount.into(),
            participants_rewarded: 0,
            is_canceled: false,
        };
        self.surveys.insert(survey_id.clone(), survey);

        Promise::new(self.gas_station.clone()).transfer(gas_fee.into());

        log!("survey_id: {}", survey_id);
        log!("participants_limit: {}", participants_limit);
        log!("reward_amount: {}", reward_amount);
        log!("gas_fee: {}", gas_fee);
        log!("survey_creator: {}", env::predecessor_account_id());
    }

    pub fn get_survey(&self, survey_id: String) -> Survey {
        self.surveys
            .get(&survey_id)
            .expect("Survey does not exist")
            .clone()
    }

    pub fn reward_participant(&mut self, survey_id: String, participant: AccountId) {
        self.assert_manager();

        let survey = self
            .surveys
            .get_mut(&survey_id)
            .expect("Survey does not exist");
        assert!(!survey.is_canceled, "Survey is canceled");

        let prefix = format!("{}-r", survey_id).into_bytes();
        let rewarded = self
            .surveys_users_rewarded
            .entry(survey_id)
            .or_insert_with(|| LookupMap::new(prefix));

        if !rewarded.contains_key(&participant) {
            assert!(
                survey.participants_rewarded < survey.participants_limit,
                "Participant limit reached"
            );

            Promise::new(participant.clone()).transfer(survey.reward_amount);
            survey.participants_rewarded += 1;
            rewarded.insert(participant, true);
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

        let non_rewarded_users = survey.participants_limit - survey.participants_rewarded;
        let refund_amount = survey
            .reward_amount
            .saturating_mul(non_rewarded_users as u128);

        Promise::new(survey_creator.clone()).transfer(refund_amount.into());
        survey.is_canceled = true;
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
