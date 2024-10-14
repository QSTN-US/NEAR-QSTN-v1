use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Gas;
use near_workspaces::sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use serde_json::json;

// const FIVE_NEAR: NearToken = NearToken::from_near(5);
const ONE_HUNDRED_NEAR: NearToken = NearToken::from_near(100);
const SURVEY_ID: &str = "1dqwc-3gpomp-32oims-9ngn9ws";
const TGAS: Gas = Gas::from_tgas(1);

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
struct Survey {
    survey_creator: AccountId,
    participants_limit: u64,
    nft_contract_id: AccountId,
    participants_rewarded: u64,
    is_canceled: bool,
}

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    test_survey_create(&user_account, &contract).await?;
    test_reward(&root, &server_account, &contract).await?;

    Ok(())
}

async fn test_survey_create(
    user_account: &Account,
    contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let fee_amount = 50 * 10u128.pow(21);

    let metadata = NFTContractMetadata {
        spec: "nft-1.0.0".to_string(),
        name: "Quizzler NFT".to_string(),
        symbol: "QUIZ".to_string(),
        icon: None,
        base_uri: None,
        reference: None,
        reference_hash: None,
    };

    let outcome = user_account
        .call(&contract.id(), "create_survey")
        .args_json(json!({"survey_id": SURVEY_ID, "participants_limit": 3u64, "gas_fee": fee_amount.to_string(), "metadata": metadata}))
        .deposit(NearToken::from_yoctonear(6 * 10u128.pow(24)))
        .max_gas()
        .transact()
        .await?;

    // let logs = outcome.logs();
    // println!("Transaction logs:");
    // for log in logs {
    //     println!("{}", log);
    // }
    // println!("Transaction details: {:#?}", outcome.clone().into_result());

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
    assert_eq!(get_survey_outcome.participants_rewarded, 0);
    assert_eq!(get_survey_outcome.is_canceled, false);

    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("NFT Contract ID: {}", get_survey_outcome.nft_contract_id);

    let metadata: NFTContractMetadata = user_account
        .view(&get_survey_outcome.nft_contract_id, "nft_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    println!("NFT Contract Name: {}", metadata.name);
    assert_eq!(metadata.name, "Quizzler NFT");

    Ok(())
}

async fn test_reward(
    root_account: &Account,
    server_account: &Account,
    contract: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let rewarded_user_account_1 = root_account
        .create_subaccount("rewarded_user_1")
        .initial_balance(ONE_HUNDRED_NEAR)
        .transact()
        .await?
        .unwrap();

    let metadata = TokenMetadata {
        title: Some("Olympus Mons".into()),
        description: Some("The tallest mountain in the charted solar system".into()),
        media: None,
        media_hash: None,
        copies: Some(1u64),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };

    let reward_amount = 10u128.pow(22);
    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_1.id(), "metadata": metadata}))
        .deposit(NearToken::from_yoctonear(10u128.pow(22)))
        .gas(TGAS.saturating_mul(150))
        .transact()
        .await?;
    assert!(outcome.is_success());

    // let logs = outcome.logs();
    // println!("Transaction logs:");
    // for log in logs {
    //     println!("{}", log);
    // }
    // println!("Transaction details: {:#?}", outcome.into_result());

    // assert_eq!(
    //     user_balance_after.as_yoctonear(),
    //     user_balance_prev.as_yoctonear() + reward_amount
    // );

    let outcome = server_account
        .call(&contract.id(), "reward_participant")
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_1.id(), "metadata": metadata.clone()}))
        .deposit(NearToken::from_yoctonear(10u128.pow(22)))
        .gas(TGAS.saturating_mul(150))
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
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_2.id(), "metadata": metadata.clone()}))
        .deposit(NearToken::from_yoctonear(10u128.pow(22)))
        .gas(TGAS.saturating_mul(150))
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
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_3.id(), "metadata": metadata.clone()}))
        .deposit(NearToken::from_yoctonear(10u128.pow(22)))
        .gas(TGAS.saturating_mul(150))
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
        .args_json(json!({"survey_id": SURVEY_ID, "participant": rewarded_user_account_4.id(), "metadata": metadata.clone()}))
        .deposit(NearToken::from_yoctonear(10u128.pow(22)))
        .gas(TGAS.saturating_mul(150))
        .transact()
        .await?;
    assert!(!outcome.is_success());
    outcome
        .into_result()
        .expect_err("Participant limit reached");

    Ok(())
}
