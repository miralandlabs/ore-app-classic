use serde_json::{json, Value};

use super::{GatewayError, GatewayResult, RPC_URL};

pub async fn get_recent_priority_fee_estimate(treasury: bool) -> u64 {
    let http_client = reqwest::Client::new();
    let mut ore_addresses: Vec<String> = vec![ore_api::id().to_string()];
    if treasury {
        ore_addresses.push(ore_api::consts::TREASURY_ADDRESS.to_string());
    }

    // let req = json!({
    //     "jsonrpc": "2.0",
    //     "id": "priority-fee-estimate",
    //     "method": "getPriorityFeeEstimate",
    //     "params": [{
    //         "accountKeys": ore_addresses,
    //         "options": {
    //             "recommended": true
    //         }
    //     }]
    // });

    // MI
    let strategy: &str;
    let req = if RPC_URL.contains("helius") {
        strategy = "helius";
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
    } else if RPC_URL.contains("alchemy") {
        strategy = "alchemy";
        json!({
            "jsonrpc": "2.0",
            "id": "priority-fee-estimate",
            "method": "getRecentPrioritizationFees",
            "params": [
                ore_addresses,
            ]
        })
    } else {
        // assume triton
        strategy = "triton";
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
    };

    // log::info!("Start request sending...");
    if let Ok(res) = http_client
        .post(RPC_URL.to_string())
        .json(&req)
        .send()
        .await
    {
        // log::info!("Got response from post.");
        if let Ok(res) = res.json::<Value>().await {
            // log::info!("Got result from response.");
            match strategy {
                "helius" => {
                    return res["result"]["priorityFeeEstimate"]
                    .as_f64()
                    .map(|fee| fee as u64)
                    .unwrap_or(0);
                }
                "alchemy" => {
                    return res["result"]
                    .as_array()
                    .and_then(|arr| {
                        Some(
                            arr.into_iter()
                                .map(|v| v["prioritizationFee"].as_u64().unwrap())
                                .collect::<Vec<u64>>(),
                        )
                    })
                    .and_then(|fees| {
                        Some((fees.iter().sum::<u64>() as f32 / fees.len() as f32).ceil()
                            as u64)
                    })
                    .ok_or_else(|| {
                        format!("Failed to parse priority fee. Response: {:?}", res)
                    })
                    .unwrap();
                }
                "triton" => {
                    return res["result"]
                    .as_array()
                    .and_then(|arr| arr.last())
                    .and_then(|last| last["prioritizationFee"].as_u64())
                    .ok_or_else(|| {
                        format!("Failed to parse priority fee. Response: {:?}", res)
                    })
                    .unwrap();
                }
                _ => return 0,
            }
        }
    } else {
        // log::info!("Failed send request."); // MI
    }

    0
}
