use dioxus::prelude::*;
use ore_api::consts::{TOKEN_DECIMALS, TOKEN_DECIMALS_V1};
use solana_client_wasm::solana_sdk::pubkey::Pubkey;
use solana_extra_wasm::account_decoder::parse_token::UiTokenAmount;

use crate::gateway::{
    ore_token_account_address, ore_token_account_address_v1, GatewayError, GatewayResult,
};

use super::{use_gateway, use_pubkey};

pub fn use_ore_balance() -> Resource<GatewayResult<UiTokenAmount>> {
    let gateway = use_gateway();
    let pubkey = use_pubkey();
    let token_account_address = ore_token_account_address(pubkey);
    use_resource(move || {
        let gateway = gateway.clone();
        async move {
            // MI
            // gateway
            //     .rpc
            //     .get_token_account_balance(&token_account_address)
            //     .await
            //     .map_err(GatewayError::from)

            match gateway
                .rpc
                .get_token_account_balance(&token_account_address)
                .await
            {
                Ok(token_account_balance) => {
                    GatewayResult::Ok(token_account_balance)
                }
                Err(err) => {
                    let err = GatewayError::from(err);
                    match err {
                        GatewayError::AccountNotFound => {
                            GatewayResult::Ok(UiTokenAmount {
                                ui_amount: Some(0f64),
                                decimals: TOKEN_DECIMALS,
                                amount: "0.00".to_string(),
                                ui_amount_string: "0.00".to_string(),
                            })
                        }
                        _ => {
                            GatewayResult::Err(err)
                        }
                    }
                }
            }
        }
    })
}

pub fn use_ore_v1_balance() -> Resource<GatewayResult<UiTokenAmount>> {
    let gateway = use_gateway();
    let pubkey = use_pubkey();
    let token_account_address = ore_token_account_address_v1(pubkey);
    use_resource(move || {
        let gateway = gateway.clone();
        async move {
            // MI
            // gateway
            //     .rpc
            //     .get_token_account_balance(&token_account_address)
            //     .await
            //     .map_err(GatewayError::from)

            match gateway
                .rpc
                .get_token_account_balance(&token_account_address)
                .await
            {
                Ok(token_account_balance) => {
                    GatewayResult::Ok(token_account_balance)
                }
                Err(err) => {
                    let err = GatewayError::from(err);
                    match err {
                        GatewayError::AccountNotFound => {
                            GatewayResult::Ok(UiTokenAmount {
                                ui_amount: Some(0f64),
                                decimals: TOKEN_DECIMALS_V1,
                                amount: "0.00".to_string(),
                                ui_amount_string: "0.00".to_string(),
                            })
                        }
                        _ => {
                            GatewayResult::Err(err)
                        }
                    }
                }
            }
        }
    })
}

pub fn use_ore_balance_user(pubkey: Pubkey) -> Resource<GatewayResult<UiTokenAmount>> {
    let gateway = use_gateway();
    let token_account_address = ore_token_account_address(pubkey);
    use_resource(move || {
        let gateway = gateway.clone();
        async move {
            // MI
            // gateway
            //     .rpc
            //     .get_token_account_balance(&token_account_address)
            //     .await
            //     .map_err(GatewayError::from)

            match gateway
                .rpc
                .get_token_account_balance(&token_account_address)
                .await
            {
                Ok(token_account_balance) => {
                    GatewayResult::Ok(token_account_balance)
                }
                Err(err) => {
                    let err = GatewayError::from(err);
                    match err {
                        GatewayError::AccountNotFound => {
                            GatewayResult::Ok(UiTokenAmount {
                                ui_amount: Some(0f64),
                                decimals: ore_api::consts::TOKEN_DECIMALS,
                                amount: "0.00".to_string(),
                                ui_amount_string: "0.00".to_string(),
                            })
                        }
                        _ => {
                            GatewayResult::Err(err)
                        }
                    }
                }
            }
        }
    })
}

pub trait UiTokenAmountBalance {
    fn balance(&self) -> u64;
}

impl UiTokenAmountBalance for UiTokenAmount {
    fn balance(&self) -> u64 {
        self.amount.parse().unwrap_or(0)
    }
}
