mod error;
mod pfee;
mod pubkey;

// MI
use crate::{
    components::PriorityFeeStrategy,
    hooks::{MinerStatusMessage, MinerToolbarState, UpdateMinerToolbarState},
};
use async_std::future::{timeout, Future};
use cached::proc_macro::cached;
use dioxus::prelude::*;
pub use error::*;
use gloo_storage::{LocalStorage, Storage};
use ore_api::{
    consts::{BUS_ADDRESSES, CONFIG_ADDRESS},
    state::{Bus, Config, Proof},
};
// use ore_types::{response::ListTransfersResponse};
use ore_utils::AccountDeserialize;
pub use pfee::*;
pub use pubkey::*;
use rand::Rng;
use solana_client_wasm::{
    solana_sdk::{
        clock::Clock,
        commitment_config::{CommitmentConfig, CommitmentLevel},
        compute_budget::ComputeBudgetInstruction,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        sysvar,
        transaction::Transaction,
    },
    utils::rpc_config::RpcSendTransactionConfig,
    WasmClient,
};
use solana_extra_wasm::{
    account_decoder::parse_token::UiTokenAccount,
    program::{
        spl_associated_token_account::{
            get_associated_token_address, instruction::create_associated_token_account,
        },
        spl_memo, spl_token,
    },
    transaction_status::{TransactionConfirmationStatus, UiTransactionEncoding},
};
use std::str::FromStr;
use web_time::Duration;

pub const API_URL: &str = "https://ore-v2-api-lthm.onrender.com"; // MI: dummy
pub const FEE_URL: &str =
    "https://mainnet.helius-rpc.com/?api-key=cb135900-fab9-4a6c-acaa-9148d6585dc7"; // MI: initial setting
                                                                                    // royal: ore-app-classic ironforge RPC Endpoint
pub const RPC_URL: &str = "https://rpc.ironforge.network/mainnet?apiKey=01J3ZM0ECN63VB741S74YPCFWS";

pub const PRIORITY_FEE_CAP: u64 = 1_000_000; // microlamport

pub const CU_LIMIT_CREATE_ATA: u32 = 85_000; // MI added
pub const CU_LIMIT_CLAIM: u32 = 12_000;
pub const CU_LIMIT_STAKE: u32 = 12_000; // MI added
pub const CU_LIMIT_TRANSFER: u32 = 30_000; // MI added, incl. memo
pub const CU_LIMIT_MINE: u32 = 1_400_000; // MI vanilla: 500_000;
pub const CU_LIMIT_UPGRADE: u32 = 30_000; // MI

const RPC_RETRIES: usize = 0;
const GATEWAY_RETRIES: usize = 64;
const CONFIRM_RETRIES: usize = 8;

const CONFIRM_DELAY: u64 = 500;
const GATEWAY_DELAY: u64 = 0; //300;

const TIP_AMOUNT: u64 = 100_000; // lamports
pub const DEFAULT_CU_LIMIT: u32 = 200_000;
pub const DEFAULT_CU_PRICE: u64 = 10_000;

#[allow(dead_code)]
pub enum ComputeBudget {
    DynamicLimitEstimatePrice,
    DynamicLimitStaticPrice(u64), // price: u64
    FixedLimitEstimatePrice(u32), // limit: u32
    FixedLimitStaticPrice(u32, u64),
}

pub const CB: ComputeBudget =
    ComputeBudget::FixedLimitStaticPrice(DEFAULT_CU_LIMIT, DEFAULT_CU_PRICE);

    #[allow(dead_code)]
    pub struct Gateway {
    pub rpc: WasmClient,
    api_url: String,
    rpc_url: String,
    fee_url: String,
}

impl Gateway {
    pub fn new(api_url: String, rpc_url: String, fee_url: String) -> Self {
        Gateway {
            api_url,
            fee_url,
            rpc_url: rpc_url.clone(),
            rpc: WasmClient::new(&rpc_url),
        }
    }

    pub async fn get_clock(&self) -> GatewayResult<Clock> {
        retry(|| self.try_get_clock()).await
    }

    pub async fn try_get_clock(&self) -> GatewayResult<Clock> {
        let data = self
            .rpc
            .get_account_data(&sysvar::clock::ID)
            .await
            .map_err(GatewayError::from)?;
        bincode::deserialize::<Clock>(&data).or(Err(GatewayError::FailedDeserialization))
    }

    pub async fn get_config(&self) -> GatewayResult<Config> {
        retry(|| self.try_get_config()).await
    }

    pub async fn try_get_config(&self) -> GatewayResult<Config> {
        let data = self
            .rpc
            .get_account_data(&CONFIG_ADDRESS)
            .await
            .map_err(GatewayError::from)?;
        Ok(*Config::try_from_bytes(&data).expect("Failed to parse config account"))
    }

    pub async fn get_proof(&self, authority: Pubkey) -> GatewayResult<Proof> {
        retry(|| self.try_get_proof(authority)).await
    }

    // pub async fn get_proof_update(
    //     &self,
    //     authority: Pubkey,
    //     challenge: [u8; 32],
    // ) -> GatewayResult<Proof> {
    //     loop {
    //         match retry(|| self.try_get_proof(authority)).await {
    //             Err(err) => return Err(err),
    //             Ok(proof) => {
    //                 if proof.challenge.ne(&challenge) {
    //                     return Ok(proof);
    //                 }
    //             }
    //         }
    //         async_std::task::sleep(Duration::from_millis(1000)).await;
    //     }
    // }

    pub async fn try_get_proof(&self, authority: Pubkey) -> GatewayResult<Proof> {
        let data = self
            .rpc
            .get_account_data(&proof_pubkey(authority))
            .await
            .map_err(GatewayError::from)?;
        Ok(*Proof::try_from_bytes(&data).expect("Failed to parse proof"))
    }

    pub async fn _get_bus(&self, id: usize) -> GatewayResult<Bus> {
        let bus_address = BUS_ADDRESSES.get(id).unwrap();
        let data = self
            .rpc
            .get_account_data(bus_address)
            .await
            .map_err(GatewayError::from)?;
        Ok(*Bus::try_from_bytes(&data).expect("Failed to parse bus"))
    }

    pub async fn get_token_account(
        &self,
        pubkey: &Pubkey,
    ) -> GatewayResult<Option<UiTokenAccount>> {
        retry(|| self.try_get_token_account(pubkey)).await
    }

    pub async fn try_get_token_account(
        &self,
        pubkey: &Pubkey,
    ) -> GatewayResult<Option<UiTokenAccount>> {
        self.rpc
            .get_token_account(pubkey)
            .await
            .map_err(GatewayError::from)
    }

    pub async fn send_and_confirm(
        &self,
        ixs: &[Instruction],
        compute_budget: ComputeBudget,
        skip_confirm: bool,
        mut toolbar_state: Option<&mut Signal<MinerToolbarState>>,
    ) -> GatewayResult<Signature> {
        let signer = signer();

        const CUS: u32 = 1_400_000;
        // Set compute budget
        let mut final_ixs = vec![];
        let (_, strategy, fee) = match compute_budget {
            ComputeBudget::DynamicLimitEstimatePrice => {
                // TODO simulate
                let fee = pfee::get_recent_priority_fee_estimate().await.unwrap();
                // final_ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(CUS));
                final_ixs.push(ComputeBudgetInstruction::set_compute_unit_price(fee));
                (CUS, PriorityFeeStrategy::Estimate, fee)
            }
            ComputeBudget::DynamicLimitStaticPrice(fee) => {
                // TODO simulate
                // final_ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(CUS));
                final_ixs.push(ComputeBudgetInstruction::set_compute_unit_price(fee));
                (CUS, PriorityFeeStrategy::Static, fee)
            }
            ComputeBudget::FixedLimitEstimatePrice(cus) => {
                let fee = pfee::get_recent_priority_fee_estimate().await.unwrap();
                // final_ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(cus));
                final_ixs.push(ComputeBudgetInstruction::set_compute_unit_price(fee));
                (cus, PriorityFeeStrategy::Estimate, fee)
            }
            ComputeBudget::FixedLimitStaticPrice(cus, fee) => {
                // final_ixs.push(ComputeBudgetInstruction::set_compute_unit_limit(cus));
                final_ixs.push(ComputeBudgetInstruction::set_compute_unit_price(fee));
                (cus, PriorityFeeStrategy::Static, fee)
            }
        };

        // Add in user instructions
        final_ixs.extend_from_slice(ixs);

        // Add tip collection instructions
        if self.rpc_url.eq(RPC_URL) {
            let mut rng = rand::thread_rng();
            let tip_accounts = &[
                // Miraland donation account only
                Pubkey::from_str("9h9TXFtSsDAiL5kpCRZuKUxPE4Nv3W56fcSyUC3zmQip").unwrap(),
            ];
            let i = rng.gen_range(0..tip_accounts.len());
            let ix = solana_sdk::system_instruction::transfer(
                &signer.pubkey(),
                &tip_accounts[i],
                TIP_AMOUNT,
            );
            final_ixs.push(ix);
        } else {
            // half tip
            let mut rng = rand::thread_rng();
            let tip_accounts = &[
                // Miraland donation account only
                Pubkey::from_str("9h9TXFtSsDAiL5kpCRZuKUxPE4Nv3W56fcSyUC3zmQip").unwrap(),
            ];
            let i = rng.gen_range(0..tip_accounts.len());
            let ix = solana_sdk::system_instruction::transfer(
                &signer.pubkey(),
                &tip_accounts[i],
                TIP_AMOUNT / 2,
            );
            final_ixs.push(ix);
        }

        // Build tx
        let send_cfg = RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: Some(CommitmentLevel::Confirmed),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: Some(RPC_RETRIES),
            min_context_slot: None,
        };
        log::info!("starting build tx via new_with_payer..."); // MI
        let mut tx = Transaction::new_with_payer(final_ixs.as_slice(), Some(&signer.pubkey()));

        // Submit tx
        let mut attempts = 0;
        loop {
            log::info!("Attempt: {:?}", attempts);
            if toolbar_state.is_some() {
                toolbar_state.as_mut().unwrap().set_status_message(MinerStatusMessage::Submitting(attempts as u64, fee));
            }

            // Sign tx with a new blockhash (after approximately ~45 sec)
            if attempts % 10 == 0 {
                // Reset the compute unit price
                let fee = if strategy.eq(&PriorityFeeStrategy::Estimate) {
                    if let Ok(fee) = pfee::get_recent_priority_fee_estimate().await {
                        fee
                    } else {
                        log::info!("failed to get fee estimate, use last known priority fee setting instead."); // MI
                        fee
                    }
                } else {
                    fee
                };

                if toolbar_state.is_some() {
                    toolbar_state.as_mut().unwrap().set_status_message(MinerStatusMessage::Submitting(attempts as u64, fee));
                }

                final_ixs.remove(1);
                final_ixs.insert(1, ComputeBudgetInstruction::set_compute_unit_price(fee));

                // Resign the tx
                let (hash, _slot) = self
                    .rpc
                    .get_latest_blockhash_with_commitment(CommitmentConfig {
                        commitment: self.rpc.commitment(),
                    })
                    .await
                    .unwrap();
                tx.sign(&[&signer], hash);
            }

            // Send transaction
            match self.rpc.send_transaction_with_config(&tx, send_cfg).await {
                Ok(sig) => {
                    log::info!("Sig: {:?}", sig);
                    // Skip confirmation
                    if skip_confirm {
                        // TODO: what msg to show on status bar?
                        return Ok(sig);
                    }

                    // Confirm transaction
                    for _ in 0..CONFIRM_RETRIES {
                        async_std::task::sleep(Duration::from_millis(CONFIRM_DELAY)).await;

                        // Fetch transaction status
                        match self.rpc.get_signature_statuses(&[sig]).await {
                            Ok(signature_statuses) => {
                                for signature_status in signature_statuses {
                                    if let Some(signature_status) = signature_status { // .as_ref()
                                        if let Some(err) = signature_status.err {
                                            log::error!("Error: {err}");
                                            return Err(GatewayError::Unknown);
                                        } else if let Some(confirmation) = signature_status.confirmation_status {
                                            match confirmation {
                                                TransactionConfirmationStatus::Processed => {}
                                                TransactionConfirmationStatus::Confirmed
                                                | TransactionConfirmationStatus::Finalized => {
                                                    log::info!("Tx sig confirmed: true");
                                                    return Ok(sig);
                                                }
                                            }
                                        } else {
                                            log::info!("No confirmation status available for current signature status.");
                                        }
                                    } else {
                                        // MI
                                        log::info!("No status available for current signature.");
                                    }
                                }
                            }

                            // Handle confirmation errors
                            Err(err) => {
                                log::error!("Error confirming tx: {:?}", err);
                            }
                        }
                    }

                    // Failed to confirm tx
                    log::info!("Tx sig confirmed: false");
                }

                // Handle submit errors
                Err(err) => {
                    log::error!("Error {:?}", err);
                }
            }

            // Retry
            async_std::task::sleep(Duration::from_millis(GATEWAY_DELAY)).await;
            attempts += 1;
            if attempts >= GATEWAY_RETRIES {
                return Err(GatewayError::TransactionTimeout);
            }
        }
    }

    // Ore
    pub async fn open_ore(&self) -> GatewayResult<()> {
        // Return early, if account is already initialized
        let signer = signer();
        let proof_address = proof_pubkey(signer.pubkey());
        if self.rpc.get_account(&proof_address).await.is_ok() {
            return Ok(());
        }

        // Sign and send transaction.
        let ix = ore_api::instruction::open(signer.pubkey(), signer.pubkey(), signer.pubkey());

        match self.send_and_confirm(&[ix], CB, false, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err(GatewayError::FailedOpen),
        }
    }

    pub async fn claim_ore(&self, amount: u64, priority_fee: u64) -> GatewayResult<Signature> {
        let signer = signer();
        let beneficiary = ore_token_account_address(signer.pubkey());

        let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_CLAIM);
        let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        let mut ixs = vec![cu_limit_ix, cu_price_ix];

        if let Ok(Some(_)) = self.get_token_account(&beneficiary).await {
            // nothing
        } else {
            // Add create ata ix
            ixs.remove(0);
            ixs.insert(0, ComputeBudgetInstruction::set_compute_unit_limit(100_000));
            ixs.push(create_associated_token_account(
                &signer.pubkey(),
                &signer.pubkey(),
                &ore_api::consts::MINT_ADDRESS,
                &spl_token::id(),
            ));
        }
        let ix = ore_api::instruction::claim(signer.pubkey(), beneficiary, amount);
        ixs.push(ix);
        self.send_and_confirm(&ixs, CB, false, None).await
    }

    // MI
    pub async fn stake_ore(&self, amount: u64, priority_fee: u64) -> GatewayResult<Signature> {
        let signer = signer();
        let sender = ore_token_account_address(signer.pubkey());

        let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_STAKE);
        let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        let ix = ore_api::instruction::stake(signer.pubkey(), sender, amount);

        self.send_and_confirm(&[cu_limit_ix, cu_price_ix, ix], CB, false, None)
            .await
    }

    // MI
    pub async fn upgrade_ore(&self, amount: u64, priority_fee: u64) -> GatewayResult<Signature> {
        let signer = signer();

        // Build initial ixs
        let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_UPGRADE);
        let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);

        let mut ixs = vec![cu_limit_ix, cu_price_ix];

        // Create target(v2) token account if necessary
        if self
            .get_token_account_ore_from_pubkey(signer.pubkey())
            .await
            .is_err()
        {
            ixs.push(create_associated_token_account(
                &signer.pubkey(),
                &signer.pubkey(),
                &ore_api::consts::MINT_ADDRESS,
                &spl_token::id(),
            ));
        }

        // Append upgrade ix
        let v1_token_account_address = ore_token_account_address_v1(signer.pubkey());
        let v2_token_account_address = ore_token_account_address(signer.pubkey());
        ixs.push(ore_api::instruction::upgrade(
            signer.pubkey(),
            v2_token_account_address,
            v1_token_account_address,
            amount,
        ));

        self.send_and_confirm(&ixs, CB, false, None).await
    }

    pub async fn transfer_ore(
        &self,
        amount: u64,
        to: Pubkey,
        memo: String,
        priority_fee: u64,
    ) -> GatewayResult<Signature> {
        // Create recipient token account, if necessary
        self.create_token_account_ore(to, priority_fee).await?;

        // Submit transfer ix
        let signer = signer();

        let from_token_account = ore_token_account_address(signer.pubkey());
        let to_token_account = ore_token_account_address(to);

        let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_TRANSFER);
        let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);

        let memo_ix = spl_memo::build_memo(&memo.into_bytes(), &[&signer.pubkey()]);
        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::ID,
            &from_token_account,
            &to_token_account,
            &signer.pubkey(),
            &[&signer.pubkey()],
            amount,
        )
        .unwrap();

        self.send_and_confirm(&[cu_limit_ix, cu_price_ix, memo_ix, transfer_ix], CB, false, None)
            .await
    }

    pub async fn create_token_account_ore(
        &self,
        owner: Pubkey,
        priority_fee: u64,
    ) -> GatewayResult<Pubkey> {
        // Build instructions.
        let signer = signer();

        // Check if account already exists.
        let token_account_address = ore_token_account_address(owner);
        match self
            .rpc
            .get_token_account(&token_account_address)
            .await
            .map_err(GatewayError::from)
        {
            Ok(token_account) => {
                if token_account.is_some() {
                    return Ok(token_account_address);
                }
            }
            Err(err) => {
                match err {
                    GatewayError::AccountNotFound => {
                        // Noop, continue on to account creation
                        log::info!("Token account not found")
                    }
                    _ => return Err(err),
                }
            }
        }

        // account not exist, create ata
        let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_CREATE_ATA);
        let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        // Sign and send transaction.
        let ix = create_associated_token_account(
            &signer.pubkey(),
            &owner,
            &ore_api::consts::MINT_ADDRESS,
            &spl_token::id(),
        );

        match self
            .send_and_confirm(&[cu_limit_ix, cu_price_ix, ix], CB, false, None)
            .await
        {
            Ok(_) => {}
            Err(_) => return Err(GatewayError::FailedAta),
        }

        // Return token account address
        Ok(token_account_address)
    }

    // // API
    // pub async fn get_transfer(&self, sig: String) -> GatewayResult<Transfer> {
    //     match reqwest::Client::new()
    //         .get(format!("{}/transfers/{}", self.api_url, sig))
    //         .send()
    //         .await
    //     {
    //         Ok(res) => res.json::<Transfer>().await.map_err(GatewayError::from),
    //         Err(e) => Err(e.into()),
    //     }
    // }

    // pub async fn _list_transfers(
    //     &self,
    //     user: Option<Pubkey>,
    //     offset: u64,
    //     limit: usize,
    // ) -> GatewayResult<ListTransfersResponse> {
    //     let offset = offset.to_string();
    //     let limit = limit.to_string();
    //     let mut query = vec![("offset", offset.as_str()), ("limit", limit.as_str())];
    //     let user_str = user.map(|u| u.to_string());
    //     let user_ref = user_str.as_deref();
    //     if let Some(user_str) = user_ref {
    //         query.push(("user", user_str));
    //     };
    //     match reqwest::Client::new()
    //         .get(format!("{}/transfers", &self.api_url))
    //         .query(&query)
    //         .send()
    //         .await
    //     {
    //         Ok(res) => res
    //             .json::<ListTransfersResponse>()
    //             .await
    //             .map_err(GatewayError::from),
    //         Err(e) => Err(e.into()),
    //     }
    // }

    // // asserts that the token account is already initialized
    // pub async fn get_token_account_ore_from_pubkey_v1(
    //     &self,
    //     pubkey: Pubkey,
    // ) -> GatewayResult<Pubkey> {
    //     let token_account_address = ore_token_account_address_v1(pubkey);
    //     self.assert_token_account_ore_exists(token_account_address)
    //         .await
    // }

    // asserts that the token account is already initialized
    pub async fn get_token_account_ore_from_pubkey(&self, pubkey: Pubkey) -> GatewayResult<Pubkey> {
        let token_account_address = ore_token_account_address(pubkey);
        self.assert_token_account_ore_exists(token_account_address)
            .await
    }

    // asserts that the token account is already initialized
    async fn assert_token_account_ore_exists(&self, ata: Pubkey) -> GatewayResult<Pubkey> {
        self.rpc
            .get_token_account(&ata)
            .await
            .map_err(GatewayError::from)
            .and_then(|maybe_some_token_account| {
                // assert that ok(none) was not returned
                maybe_some_token_account.ok_or(GatewayError::FailedAta)
            })
            .map(|_| ata)
    }
}

pub fn signer() -> Keypair {
    let key = "keypair";
    let value = LocalStorage::get(key).ok().unwrap_or_else(|| {
        let x = Keypair::new().to_base58_string();
        LocalStorage::set(key, &x).ok();
        x
    });
    Keypair::from_base58_string(&value)
}

pub async fn retry<F, Fut, T>(f: F) -> GatewayResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = GatewayResult<T>>,
{
    const MAX_RETRIES: u32 = 8;
    const INITIAL_BACKOFF: Duration = Duration::from_millis(200);
    const TIMEOUT: Duration = Duration::from_secs(8);
    let mut backoff = INITIAL_BACKOFF;
    for attempt in 0..MAX_RETRIES {
        match timeout(TIMEOUT, f()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(_e)) if attempt < MAX_RETRIES - 1 => {
                async_std::task::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }
            Ok(Err(e)) => return Err(e),
            Err(_) if attempt < MAX_RETRIES - 1 => {
                async_std::task::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }
            Err(_) => return Err(GatewayError::RetryFailed),
        }
    }

    Err(GatewayError::RetryFailed)
}

#[cached]
pub fn ore_token_account_address(pubkey: Pubkey) -> Pubkey {
    get_associated_token_address(&pubkey, &ore_api::consts::MINT_ADDRESS)
}

#[cached]
pub fn ore_token_account_address_v1(pubkey: Pubkey) -> Pubkey {
    get_associated_token_address(&pubkey, &ore_api::consts::MINT_V1_ADDRESS)
}
