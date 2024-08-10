mod web_worker;

use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_sdk::utils::channel::UseChannel;
use drillx::Solution;
use lazy_static::lazy_static;
use ore_api::{
    consts::{BUS_COUNT, EPOCH_DURATION},
    state::Proof,
};
use rand::Rng;
use serde_wasm_bindgen::to_value;
use solana_client_wasm::solana_sdk::{
    compute_budget::ComputeBudgetInstruction, pubkey::Pubkey, signature::Signature, signer::Signer,
};
use solana_sdk::keccak::Hash as KeccakHash;
use web_sys::{window, Worker};
pub use web_worker::*;

use crate::{
    components::PriorityFeeStrategy,
    gateway::{self, signer, ComputeBudget, Gateway, GatewayResult, CU_LIMIT_MINE},
    hooks::{
        use_priority_fee_strategy, MinerStatus, MinerStatusMessage, MinerToolbarState, PowerLevel,
        PriorityFee, ReadMinerToolbarState, UpdateMinerToolbarState,
    },
    utils,
};

// Number of physical cores on machine
lazy_static! {
    pub static ref WEB_WORKERS: usize = fetch_logical_processors();
}

fn fetch_logical_processors() -> usize {
    let window = window().expect("should have a window");
    let navigator = window.navigator();
    navigator.hardware_concurrency() as usize
}

/// Miner encapsulates the logic needed to efficiently mine for valid hashes according to the application runtime and hardware.
pub struct Miner {
    power_level: Signal<PowerLevel>,
    priority_fee: Signal<PriorityFee>,
    priority_fee_strategy: Signal<PriorityFeeStrategy>,
    web_workers: Vec<Worker>,
}

impl Miner {
    pub fn new(
        cx: UseChannel<WebWorkerResponse>,
        power_level: Signal<PowerLevel>,
        priority_fee: Signal<PriorityFee>,
        priority_fee_strategy: Signal<PriorityFeeStrategy>,
    ) -> Self {
        Self {
            power_level: power_level.clone(),
            priority_fee: priority_fee.clone(),
            priority_fee_strategy: priority_fee_strategy.clone(),
            web_workers: (0..*WEB_WORKERS)
                .map(|_| create_web_worker(cx.clone()))
                .collect(),
        }
    }

    pub async fn start_mining(&self, challenge: [u8; 32], offset: u64, cutoff_time: u64) {
        self.start_mining_web(challenge, offset, cutoff_time).await;
    }

    pub async fn start_mining_web(&self, challenge: [u8; 32], offset: u64, cutoff_time: u64) {
        let nonce = u64::MAX.saturating_div(self.web_workers.len() as u64);
        let power_level = self.power_level.read().0.saturating_sub(1) as usize;
        log::info!("Start mining web...");
        for (i, web_worker) in self.web_workers.iter().enumerate() {
            let nonce = nonce.saturating_mul(i as u64).saturating_add(offset);
            if i.le(&power_level) {
                log::info!("Posting message: {:?} {:?}", i, nonce);
                web_worker
                    .post_message(
                        &to_value(
                            &(WebWorkerRequest {
                                challenge,
                                nonce: nonce.to_le_bytes(),
                                offset,
                                cutoff_time,
                                power_level,
                            }),
                        )
                        .unwrap(),
                    )
                    .unwrap();
            }
        }
    }

    pub async fn process_web_worker_results(
        &self,
        messages: &Vec<WebWorkerResponse>,
        toolbar_state: &mut Signal<MinerToolbarState>,
        proof: &mut Resource<GatewayResult<Proof>>,
        gateway: Rc<Gateway>,
        pubkey: Pubkey,
    ) {
        log::info!("Batch: {:?}", messages);
        // Exit early if not active
        match toolbar_state.status() {
            MinerStatus::Active => {}
            _ => return,
        }

        // Get best solution
        let mut challenge = [0; 32];
        let mut offset = 0;
        let mut best_difficulty = 0;
        let mut best_solution = Solution::new([0; 16], [0; 8]);
        let mut best_hash = [0u8; 32];
        for msg in messages {
            if msg.difficulty.gt(&best_difficulty) {
                best_solution = drillx::Solution::new(msg.digest, msg.nonce);
                best_difficulty = msg.difficulty;
                best_hash = msg.hash;
                offset = msg.offset;
                challenge = msg.challenge;
            }
        }

        // Kickoff new batch
        if let Ok(config) = gateway.get_config().await {
            if best_difficulty.lt(&(config.min_difficulty as u32)) {
                self.start_mining(challenge, offset, 0).await;
                return;
            }
        }

        // let priority_fee = self.priority_fee.read().0;
        let priority_fee = if self
            .priority_fee_strategy
            .read()
            .eq(&PriorityFeeStrategy::Dynamic)
        {
            gateway::get_recent_priority_fee_estimate().await.unwrap()
        } else {
            self.priority_fee.read().0
        };
        log::info!("current priority fee: {}", priority_fee);
        self.priority_fee.clone().set(PriorityFee(priority_fee)); // set signal
        log::info!("set priority fee signal: {}", priority_fee);

        // Update toolbar state
        toolbar_state.set_display_hash(KeccakHash::new_from_array(best_hash));
        toolbar_state.set_status_message(MinerStatusMessage::Submitting(0, priority_fee));

        // Submit solution
        match submit_solution(&gateway, best_solution, priority_fee).await {
            // Start mining again
            Ok(_sig) => {
                // log::info!("Sig: {}", sig); // MI
                proof.restart();
                if let MinerStatus::Active = toolbar_state.status() {
                    if let Ok(proof) = gateway.get_proof(pubkey).await {
                        if let Ok(clock) = gateway.get_clock().await {
                            toolbar_state.set_status_message(MinerStatusMessage::Searching);
                            let cutoff_time = proof
                                .last_hash_at
                                .saturating_add(60)
                                .saturating_sub(clock.unix_timestamp)
                                .max(0) as u64;
                            self.start_mining(proof.challenge.into(), 0, cutoff_time)
                                .await;
                        } else {
                            log::error!("Failed to get clock");
                        }
                    } else {
                        log::error!("Failed to get proof");
                    }
                }
            }

            // Display error
            Err(err) => {
                toolbar_state.set_status_message(MinerStatusMessage::Error);
                log::error!("Failed to submit hash: {:?}", err);
            }
        }
    }
}

pub async fn submit_solution(
    gateway: &Rc<Gateway>,
    solution: Solution,
    priority_fee: u64,
) -> GatewayResult<Signature> {
    let signer = signer();
    // let priority_fee = use_priority_fee();
    let priority_fee_strategy = use_priority_fee_strategy();

    // Build ixs
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT_MINE);
    let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
    let auth_ix = ore_api::instruction::auth(utils::proof_pubkey(signer.pubkey())); // MI
    let mut ixs = vec![cu_limit_ix, cu_price_ix, auth_ix];

    // Reset if needed
    // if needs_reset(gateway).await { // MI: vanilla
    if needs_reset(gateway).await && rand::thread_rng().gen_range(0..100).eq(&0) {
        ixs.push(ore_api::instruction::reset(signer.pubkey()));
    }

    // Build mine tx
    let bus_id = pick_bus();
    let ix = ore_api::instruction::mine(
        signer.pubkey(),
        signer.pubkey(),
        ore_api::consts::BUS_ADDRESSES[bus_id],
        solution,
    );
    ixs.push(ix);

    // Send and configm
    let cb = if priority_fee_strategy
        .read()
        .eq(&PriorityFeeStrategy::Dynamic)
    {
        ComputeBudget::Dynamic
    } else {
        ComputeBudget::Fixed(priority_fee as u32)
    };
    gateway.send_and_confirm(&ixs, cb, false).await
}

async fn needs_reset(gateway: &Rc<Gateway>) -> bool {
    if let Ok(clock) = gateway.get_clock().await {
        if let Ok(config) = gateway.get_config().await {
            return config
                .last_reset_at
                .saturating_add(EPOCH_DURATION)
                .saturating_sub(5) // Buffer
                .le(&clock.unix_timestamp);
        }
    }
    false
}

fn pick_bus() -> usize {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..BUS_COUNT)
}
