# QSTN + NEAR Network - work in progress

<p align="center">
  <a href="https://qstn.us/"><img src="https://qstn.us/icon-256x256.png" alt="QSTN Marketplace"></a>
</p>

**_🚀 QSTN is a platform that connects businesses and individuals through market research surveys. We partner with companies that are looking for feedback from consumers like you, and we provide the opportunity for you to earn rewards while sharing your opinions._**

**_Mainnet_**
https://qstn.us

**_Testnet_**
https://testnet.qstnus.com

**_Business Demo_**
[[https://drive.google.com/file/d/13xazUCDOVFUpGGrDff0YnXcdu9DXWT8D/view?usp=drive_link](https://drive.google.com/file/d/1RvteIQxxXUbCIxWuPFjO_FS0tEfSfEPD/view?usp=sharing)](https://drive.google.com/file/d/1GpHQLzgqlgt-SOAZWeH2HUsjmbNWeWpD/view?usp=sharing)

**_User Demo_**
[[https://drive.google.com/file/d/1PkL0nq9getU_tvM8L0itXx_k4m2xBoT2/view?usp=drive_link](https://drive.google.com/file/d/19ZXFe4WjdrI80f3wfEwsUmM9K2Gh8r_w/view?usp=sharing)
](https://drive.google.com/file/d/11G7658gi7fu-yu-DL0lZErGy5Kk0jNkM/view?usp=sharing)

**QSTN Survey Smart Contracts Documentation**

Welcome to the QSTN Survey Smart Contracts repository. This guide will help you understand how to deploy and use our smart contracts for business funding on the NEAR blockchain.

Table of Contents
Introduction
Prerequisites
Installation
Contract Overview
Deploying the Contracts
Using the Contracts
Examples
Contributing
Support
License

**Introduction**

QSTN provides a decentralized solution for businesses to fund surveys using smart contracts on the NEAR blockchain. This guide explains how to set up, deploy, and interact with the QSTN survey smart contracts.

**Prerequisites**

To run, test, or deploy this smart contract, make sure you have everything necessary to run the project on Cargo installed. You can find all the necessary instructions for setting up the Cargo environment [here](https://github.com/near/cargo-near).

**Installation of Native contract**

Clone the repository and navigate to the contracts directory:

```bash
git clone https://github.com/QSTN-US/NEAR-QSTN-v1
cd NEAR-QSTN-v1/CONTRACTS-FT
cargo near build
cargo test
```

**Installation of NFT contract**

Clone the repository and navigate to the contracts directory:

```bash
git clone https://github.com/QSTN-US/NEAR-QSTN-v1
cd NEAR-QSTN-v1/CONTRACTS-NFT
cargo near build
cargo test
```

**Contract Overview (Native NEAR Token)**

This repository contains a NEAR smart contract designed for creating and managing surveys using native NEAR tokens. Key components include:

lib.rs: Main contract file for survey creation and management using NEAR tokens.
quizzler_tests.rs: Test cases demonstrating how to interact with the contract using NEAR tokens.

The contract allows the creation of surveys, rewards participants with NEAR tokens, and includes an emergency withdrawal feature. Key functions include:

```rust
create_survey(
  survey_id: String,
  participants_limit: u64,
  reward_amount: NearToken,
  gas_fee: NearToken
)
```

Creates a new survey. The caller needs to provide enough deposit to cover participant rewards and gas fees. Only business users who create the surveys can call this function.

```rust
reward_participant(
  survey_id: String,
  participant: AccountId
)
```

`reward_participant` distributes a reward in NEAR tokens to a participant who completes the survey.

```rust
cancel_survey(survey_id: String)
```

`cancel_survey` cancels a survey and refunds unused funds to the survey creator. Only the survey creator or a designated manager can call this function.

```rust
emergency_withdraw(amount: NearToken, account_id: AccountId)
```

`emergency_withdraw` allows the contract owner to withdraw funds in case of an emergency.

**Deploying the Contracts**

Follow these steps to deploy the contracts on the NEAR blockchain:

```bash
near deploy --wasmFile contract.wasm --accountId <your_account>
```

**Examples**

Creating a survey with NEAR tokens:

```typescript
const result = await contract.create_survey({
  survey_id: "123dqwc-3gpomp-32oims-9ngn9ws",
  participants_limit: 100,
  reward_amount: "5000000000000000000000000", // Amount in yoctoNEAR
  gas_fee: "150000000000000000000000", // Gas fee in yoctoNEAR
});
```

Rewarding a participant with NEAR tokens:

```typescript
const result = await contract.reward_participant({
  survey_id: "123dqwc-3gpomp-32oims-9ngn9ws",
  participant: "participant.testnet",
});
```

**Contract Overview (NFT)**

This repository contains a NEAR smart contract designed for creating and managing surveys with rewards in the form of NFTs. Key components include:

lib.rs: Main contract file for survey creation and management using NFTs.
quizzler_tests.rs: Test cases demonstrating how to interact with the contract using NFTs.

The contract supports creating surveys, rewarding participants with NFTs, and canceling surveys. Key functions include:

```rust
create_survey(
  survey_id: String,
  participants_limit: u64,
  gas_fee: NearToken,
  metadata: NFTContractMetadata
)
```

Creates a new survey where participants are rewarded with NFTs. The caller must provide sufficient deposit for minting NFTs and gas fees.

```rust
reward_participant(
  survey_id: String,
  participant: AccountId,
  metadata: TokenMetadata
)
```

`reward_participant` mints and transfers an NFT to the participant as a reward for completing the survey.

```rust
cancel_survey(survey_id: String)
```

`cancel_survey` cancels the survey and prevents further rewards from being issued. Only the survey creator or a manager can call this function.

```rust
emergency_withdraw(amount: NearToken, account_id: AccountId)
```

`emergency_withdraw` wWithdraws funds in case of an emergency. This function can only be called by the contract owner.

**Deploying the Contracts**

Follow these steps to deploy the contracts on the NEAR blockchain:

```bash
near deploy --wasmFile contract.wasm --accountId <your_account>
```

**Examples**

Creating a survey with NFT rewards:

```typescript
const metadata = {
  spec: "nft-1.0.0",
  name: "Quizzler NFT",
  symbol: "QUIZ",
  icon: null,
  base_uri: null,
  reference: null,
  reference_hash: null,
};

const result = await contract.create_survey({
  survey_id: "1dqwc-3gpomp-32oims-9ngn9ws",
  participants_limit: 3,
  gas_fee: "150000000000000000000000", // Gas fee in yoctoNEAR
  metadata,
});
```

Rewarding a participant with an NFT:

```typescript
const metadata = {
  title: "Survey NFT",
  description: "Reward for completing the survey",
  media: null,
  copies: 1,
};

const result = await contract.reward_participant({
  survey_id: "1dqwc-3gpomp-32oims-9ngn9ws",
  participant: "participant.testnet",
  metadata,
});
```

**Contributing**

We welcome contributions! Please read our contributing guide to get started.

**Support**

If you encounter any issues or have questions, please open an issue on GitHub or contact our support team at support@qstn.us.

**License**

This project is licensed under the MIT License. See the LICENSE file for details.

```

```
