use serde_json::{json, Value};

use super::{GatewayError, GatewayResult, RPC_URL};

pub async fn get_recent_priority_fee_estimate(treasury: bool) -> u64 {
    let http_client = reqwest::Client::new();
    let mut ore_addresses: Vec<String> = vec![ore_api::id().to_string()];
    if treasury {
        ore_addresses.push(ore_api::consts::TREASURY_ADDRESS.to_string());
    }

    let req = json!({
        "jsonrpc": "2.0",
        "id": "priority-fee-estimate",
        "method": "getPriorityFeeEstimate",
        "params": [{
            "accountKeys": ore_addresses,
            "options": {
                "recommended": true
            }
        }]
    });

    // MI: test
    let rpc_url = "https://mainnet.helius-rpc.com/?api-key=c9de0b57-067d-4b66-963b-416667c07d10";

    // MI: replace RPC_URL with rpc_url
    if let Ok(res) = http_client
        .post(rpc_url.to_string())
        .json(&req)
        .send()
        .await
    {
        if let Ok(res) = res.json::<Value>().await {
            return res["result"]["priorityFeeEstimate"]
                .as_f64()
                .map(|fee| fee as u64)
                .unwrap_or(0);
        }
    }

    0
}
