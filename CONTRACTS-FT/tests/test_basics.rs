use near_sdk::serde::{Deserialize, Serialize};
use near_workspaces::sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use serde_json::json;

// const FIVE_NEAR: NearToken = NearToken::from_near(5);
const ONE_HUNDRED_NEAR: NearToken = NearToken::from_near(100);
const SURVEY_ID: &str = "1dqwc-3gpomp-32oims-9ngn9ws";

#[tokio::test]
async fn test_check_in() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    let root = sandbox.root_account()?;

    let user_account = root
        .create_subaccount("user")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let deployer_account = root
        .create_subaccount("deployer")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let server_account = root
        .create_subaccount("server")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let contract = deployer_account.deploy(&contract_wasm).await?.unwrap();

    let outcome = deployer_account
        .call(contract.id(), "new")
        .args_json(json!({"gas_station": server_account.id()}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = deployer_account
        .call(contract.id(), "set_manager")
        .args_json(json!({"manager": server_account.id(), "status": true}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let is_manager_outcome: bool = contract
        .view("is_manager")
        .args_json(json!({"manager": server_account.id()}))
        .await?
        .json()?;
    assert_eq!(is_manager_outcome, true);

    let get_gas_station_outcome: AccountId = contract
        .view("get_gas_station")
        .args_json(json!({}))
        .await?
        .json()?;
    assert_eq!(
        get_gas_station_outcome.to_string(),
        server_account.id().to_string()
    );

    let rewarded_user_account_1 = root
        .create_subaccount("rewarded_user_1")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    test_survey_create(&user_account, &contract).await?;
    test_reward(&root, &rewarded_user_account_1, &server_account, &contract).await?;
    test_cancel(&root, &rewarded_user_account_1, &server_account, &contract).await?;

    Ok(())
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
struct Survey {
    survey_creator: AccountId,
    participants_limit: u64,
    reward_amount: NearToken,
    participants_rewarded: u64,
    is_canceled: bool,
}

async fn test_survey_create(
    user_account: &Account,
    contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let fee_amount = 10u128.pow(23);
    let reward_amount = 10u128.pow(22);
    let participants_limit = 3u128;
    let deposit_amount = fee_amount + (reward_amount * participants_limit);

    let outcome = user_account
        .call(&contract.id(), "create_survey")
        .args_json(json!({"survey_id": SURVEY_ID, "participants_limit": participants_limit as u64, "reward_amount": reward_amount.to_string(), "gas_fee": fee_amount.to_string()}))
        .deposit(NearToken::from_yoctonear(deposit_amount))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let get_survey_outcome: Survey = contract
        .view("get_survey")
        .args_json(json!({"survey_id": SURVEY_ID}))
        .await?
        .json()?;
    assert_eq!(
        get_survey_outcome.survey_creator.to_string(),
        user_account.id().to_string()
    );
    assert_eq!(
        get_survey_outcome.reward_amount.as_yoctonear().to_string(),
        reward_amount.to_string()
    );
    assert_eq!(get_survey_outcome.participants_rewarded, 0);
    assert_eq!(get_survey_outcome.is_canceled, false);

    Ok(())
}

async fn test_reward(
    root_account: &Account,
    rewarded_user_account_1: &Account,
    server_account: &Account,
    contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let reward_amount = 10u128.pow(22);
    let user_balance_prev = rewarded_user_account_1.view_account().await?.balance;
    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_1.id()}))
        .transact()
        .await?;
    assert!(outcome.is_success());
    let user_balance_after = rewarded_user_account_1.view_account().await?.balance;

    assert_eq!(
        user_balance_after.as_yoctonear(),
        user_balance_prev.as_yoctonear() + reward_amount
    );

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_1.id()}))
        .transact()
        .await?;
    assert!(!outcome.is_success());
    outcome
        .into_result()
        .expect_err("Participant already rewarded");

    let rewarded_user_account_2 = root_account
        .create_subaccount("rewarded_user_2")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_2.id()}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let rewarded_user_account_3 = root_account
        .create_subaccount("rewarded_user_3")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_3.id()}))
        .transact()
        .await?;
    assert!(outcome.is_success());

    let rewarded_user_account_4 = root_account
        .create_subaccount("rewarded_user_4")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_4.id()}))
        .transact()
        .await?;
    assert!(!outcome.is_success());
    outcome
        .into_result()
        .expect_err("Participant limit reached");

    Ok(())
}

async fn test_cancel(
    root_account: &Account,
    rewarded_user_account_1: &Account,
    server_account: &Account,
    contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    const NEW_SURVEY_ID: &str = "2iopoj-ku788q-q231r9-9cgiu87";
    let fee_amount = 10u128.pow(23);
    let reward_amount = 10u128.pow(22);
    let participants_limit = 3u128;
    let deposit_amount = fee_amount + (reward_amount * participants_limit);

    let business_user_account_1 = root_account
        .create_subaccount("business_user_1")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let outcome = business_user_account_1
        .call(&contract.id(), "create_survey")
        .args_json(json!({"survey_id": NEW_SURVEY_ID, "participants_limit": participants_limit as u64, "reward_amount": reward_amount.to_string(), "gas_fee": fee_amount.to_string()}))
        .deposit(NearToken::from_yoctonear(deposit_amount))
        .max_gas()
        .transact()
        .await?;
    assert!(outcome.is_success());

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": NEW_SURVEY_ID, "participant": rewarded_user_account_1.id()}))
        .transact()
        .await?;
    assert!(outcome.is_success());
    // println!("{:#?}", outcome.clone().into_result());

    let user_balance_prev = business_user_account_1.view_account().await?.balance;
    let outcome = server_account
        .call(&contract.id(), "cancel_survey")
        .args_json(json!({"survey_id": NEW_SURVEY_ID}))
        .transact()
        .await?;
    assert!(outcome.is_success());
    let user_balance_after = business_user_account_1.view_account().await?.balance;

    assert_eq!(
        user_balance_after.as_yoctonear(),
        user_balance_prev.as_yoctonear() + reward_amount * 2
    );

    let rewarded_user_account_6 = root_account
        .create_subaccount("rewarded_user_6")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": NEW_SURVEY_ID, "participant": rewarded_user_account_6.id()}))
        .transact()
        .await?;
    assert!(!outcome.is_success());
    outcome.into_result().expect_err("Survey is canceled");

    Ok(())
}
