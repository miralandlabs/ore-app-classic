use dioxus::prelude::*;
use ore_api::consts::{TOKEN_DECIMALS, TOKEN_DECIMALS_V1};
use solana_extra_wasm::account_decoder::parse_token::UiTokenAmount;

use crate::gateway::{ore_token_account_address, ore_token_account_address_v1};

use super::{use_gateway, use_pubkey};

// MI
// current dixous only have one resource per hook
pub fn use_ore_v1_v2_balances() -> Resource<Option<Balances>> {
    let gateway = use_gateway();
    let pubkey = use_pubkey();
    use_resource(move || {
        let gateway = gateway.clone();
        async move {
            let token_account_address_v1 = ore_token_account_address_v1(pubkey);
            let token_account_address_v2 = ore_token_account_address(pubkey);
            let balance_v1 = gateway
                .rpc
                .get_token_account_balance(&token_account_address_v1)
                .await
                .unwrap_or(UiTokenAmount::default(TOKEN_DECIMALS_V1));
            let balance_v2 = gateway
                .rpc
                .get_token_account_balance(&token_account_address_v2)
                .await
                .unwrap_or(UiTokenAmount::default(TOKEN_DECIMALS));
            Some(Balances {
                v1: balance_v1,
                v2: balance_v2,
            })
        }
    })
}

pub trait UiTokenAmountDefault {
    fn default(decimals: u8) -> Self;
}

impl UiTokenAmountDefault for UiTokenAmount {
    fn default(decimals: u8) -> Self {
        UiTokenAmount {
            ui_amount: None,
            decimals,
            amount: "0".to_string(),
            ui_amount_string: "0".to_string(),
        }
    }
}

// MI
#[derive(Clone)]
pub struct Balances {
    pub v1: UiTokenAmount,
    #[allow(dead_code)]
    pub v2: UiTokenAmount,
}