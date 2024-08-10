use crate::hooks::{use_fee_url, use_priority_fee, use_priority_fee_cap};
use dioxus::signals::Readable;
use ore_api::consts::BUS_ADDRESSES;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_client_wasm::solana_sdk::clock::Slot;

use url::Url;

enum FeeStrategy {
    Helius,
    Triton,
    Alchemy,
    Quiknode,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RpcPrioritizationFee {
    pub slot: Slot,
    pub prioritization_fee: u64,
}

pub async fn get_recent_priority_fee_estimate() -> Result<u64, String> {
    // Get url
    let fee_url = use_fee_url();
    let priority_fee = use_priority_fee();
    let priority_fee_cap = use_priority_fee_cap();

    // Select fee estiamte strategy
    let host = Url::parse(&fee_url.read().0)
        .unwrap()
        .host_str()
        .unwrap()
        .to_string();
    let strategy = if host.contains("helius-rpc.com") {
        FeeStrategy::Helius
    } else if host.contains("alchemy.com") {
        FeeStrategy::Alchemy
    } else if host.contains("quiknode.pro") {
        FeeStrategy::Quiknode
    } else if host.contains("rpcpool.com") {
        FeeStrategy::Triton
    } else {
        return Err("Dynamic fees not supported by this RPC.".to_string());
    };

    // Build fee estimate request
    let client = Client::new();
    let ore_addresses: Vec<String> = std::iter::once(ore_api::ID.to_string())
        .chain(BUS_ADDRESSES.iter().map(|pubkey| pubkey.to_string()))
        .collect();
    let body = match strategy {
        FeeStrategy::Helius => {
            json!({
                "jsonrpc": "2.0",
                "id": "priority-fee-estimate",
                "method": "getPriorityFeeEstimate",
                "params": [{
                    "accountKeys": ore_addresses,
                    "options": {
                        "recommended": true
                    }
                }]
            })
        }
        FeeStrategy::Alchemy => {
            json!({
                "jsonrpc": "2.0",
                "id": "priority-fee-estimate",
                "method": "getRecentPrioritizationFees",
                "params": [
                    ore_addresses
                ]
            })
        }
        FeeStrategy::Quiknode => {
            json!({
                "jsonrpc": "2.0",
                "id": "1",
                "method": "qn_estimatePriorityFees",
                "params": {
                    "account": BUS_ADDRESSES[0].to_string(),
                    "last_n_blocks": 100
                }
            })
        }
        FeeStrategy::Triton => {
            json!({
                "jsonrpc": "2.0",
                "id": "priority-fee-estimate",
                "method": "getRecentPrioritizationFees",
                "params": [
                    ore_addresses,
                    {
                        "percentile": 5000,
                    }
                ]
            })
        }
    };

    // // Send request
    // let response: Value = client
    //     .post(fee_url.read().0.clone())
    //     .json(&body)
    //     .send()
    //     .await
    //     .unwrap()
    //     .json()
    //     .await
    //     .unwrap();

    // MI, Send request in two steps
    // split json from send
    // 1) handle response
    let Ok(resp) = client
        .post(fee_url.read().0.clone())
        .json(&body)
        .send()
        .await
    else {
        eprintln!("didn't get dynamic fee estimate, use default instead.");
        return Ok(priority_fee.read().0);
    };

    // 2) handle json
    let Ok(response) = resp.json::<Value>().await else {
        eprintln!("didn't get json data from fee estimate response, use default instead.");
        return Ok(priority_fee.read().0);
    };

    // Parse response
    let calculated_fee = match strategy {
        FeeStrategy::Helius => response["result"]["priorityFeeEstimate"]
            .as_f64()
            .map(|fee| fee as u64)
            .ok_or_else(|| format!("Failed to parse priority fee response: {:?}", response)),
        FeeStrategy::Quiknode => response["result"]["per_compute_unit"]["medium"]
            .as_f64()
            .map(|fee| fee as u64)
            .ok_or_else(|| {
                format!(
                    "Please enable the Solana Priority Fee API add-on in your QuickNode account."
                )
            }),
        FeeStrategy::Alchemy => response["result"]
            .as_array()
            .and_then(|arr| {
                Some(
                    arr.into_iter()
                        .map(|v| v["prioritizationFee"].as_u64().unwrap())
                        .collect::<Vec<u64>>(),
                )
            })
            .and_then(|fees| {
                Some(((fees.iter().sum::<u64>() as f32 / fees.len() as f32).ceil() * 1.2) as u64)
            })
            .ok_or_else(|| format!("Failed to parse priority fee response: {:?}", response)),
        FeeStrategy::Triton => {
            serde_json::from_value::<Vec<RpcPrioritizationFee>>(response["result"].clone())
                .map(|prioritization_fees| {
                    estimate_prioritization_fee_micro_lamports(prioritization_fees)
                })
                .or_else(|error: serde_json::Error| {
                    Err(format!(
                        "Failed to parse priority fee response: {response:?}, error: {error}"
                    ))
                })
        }
    };

    // Check if the calculated fee is higher than max
    match calculated_fee {
        Err(err) => Err(err),
        Ok(fee) => Ok(fee.min(priority_fee_cap.read().0)),
    }
}

/// Our estimate is the average over the last 20 slots
pub fn estimate_prioritization_fee_micro_lamports(
    prioritization_fees: Vec<RpcPrioritizationFee>,
) -> u64 {
    let prioritization_fees = prioritization_fees
        .into_iter()
        .rev()
        .take(20)
        .map(
            |RpcPrioritizationFee {
                 prioritization_fee, ..
             }| prioritization_fee,
        )
        .collect::<Vec<_>>();
    if prioritization_fees.is_empty() {
        panic!("Response does not contain any prioritization fees");
    }

    let prioritization_fee =
        prioritization_fees.iter().sum::<u64>() / prioritization_fees.len() as u64;

    prioritization_fee
}
