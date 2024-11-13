#![cfg(not(target_arch = "wasm32"))]
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use cw_orch_interchain::{prelude::*, InterchainEnv};
use euclid::{
    chain::ChainUid,
    msgs::router::{QueryMsgFns, TokenDenom, TokenDenomsResponse},
    token::Token,
};
use router::RouterContract;
use virtual_balance::VirtualBalanceContract;
use vlp::VlpContract;

const _USER: &str = "user";
const _NATIVE_DENOM: &str = "native";
const _IBC_DENOM_1: &str = "ibc/denom1";
const _IBC_DENOM_2: &str = "ibc/denom2";
const _SUPPLY: u128 = 1_000_000;

#[test]
fn test_migrate_router() {
    let sender = Addr::unchecked("sender_for_all_chains").into_string();
    let interchain = MockInterchainEnv::new(vec![("nibiru", &sender)]);
    let nibiru = interchain.get_chain("nibiru").unwrap();

    nibiru
        .set_balance(sender.clone(), vec![Coin::new(100000000000000, "nibi")])
        .unwrap();

    let router_nibiru = RouterContract::new(nibiru.clone());
    let virtual_balance_nibiru = VirtualBalanceContract::new(nibiru.clone());
    let vlp_nibiru = VlpContract::new(nibiru.clone());

    router_nibiru.upload().unwrap();
    virtual_balance_nibiru.upload().unwrap();
    vlp_nibiru.upload().unwrap();
    router_nibiru.upload().unwrap();

    router_nibiru
        .instantiate(
            &euclid::msgs::router::InstantiateMsg {
                vlp_code_id: 3,
                virtual_balance_code_id: 2,
            },
            Some(&Addr::unchecked(sender)),
            None,
        )
        .unwrap();

    router_nibiru
        .migrate(
            &euclid::msgs::vlp::MigrateMsg {
                denoms: vec![(
                    Token::create("migrateToken".to_string()).unwrap(),
                    TokenDenom {
                        chain_uid: ChainUid::vsl_chain_uid().unwrap(),
                        token_type: euclid::token::TokenType::Native {
                            denom: "native".to_string(),
                        },
                    },
                )],
            },
            4,
        )
        .unwrap();

    let new_state: TokenDenomsResponse = router_nibiru
        .query_token_denoms(Token::create("migrateToken".to_string()).unwrap())
        .unwrap();

    assert_eq!(
        new_state.denoms,
        vec![TokenDenom {
            chain_uid: ChainUid::vsl_chain_uid().unwrap(),
            token_type: euclid::token::TokenType::Native {
                denom: "native".to_string(),
            },
        }]
    )
}
