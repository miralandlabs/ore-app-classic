use std::rc::Rc;

use dioxus::prelude::*;
use solana_client_wasm::solana_sdk::signer::Signer;

use crate::{
    gateway::{signer, Gateway, GatewayResult},
    hooks::{MinerStatusMessage, MinerToolbarState, UpdateMinerToolbarState},
    miner::Miner,
};

// TODO Move this somewhere

pub async fn try_start_mining(
    gateway: Rc<Gateway>,
    miner: Signal<Miner>,
    toolbar_state: &mut Signal<MinerToolbarState>,
) -> GatewayResult<()> {
    loop {
        if gateway.open_ore().await.is_ok() {
            break;
        }
    }

    // let signer = signer();
    let authority = signer().pubkey();
    let proof = gateway.get_proof(authority).await?;
    let clock = gateway.get_clock().await?;
    // if let Ok(proof) = gateway.get_proof(signer.pubkey()).await {
    //     if let Ok(clock) = gateway.get_clock().await {
    let cutoff_time = proof
        .last_hash_at
        .saturating_add(60)
        .saturating_sub(clock.unix_timestamp)
        .max(0) as u64;
    toolbar_state.set_status_message(MinerStatusMessage::Searching);
    miner
        .read()
        .start_mining(proof.challenge.into(), 0, cutoff_time)
        .await;
    //     } else {
    //         log::error!("Failed to get clock");
    //     }
    // } else {
    //     log::error!("Failed to get proof");
    // }

    Ok(())
}
