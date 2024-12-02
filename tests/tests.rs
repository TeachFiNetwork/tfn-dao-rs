mod consts;

use consts::*;

use multiversx_sc_scenario::{
    managed_token_id, scenario_model::{
        Account, ScCallStep, SetStateStep
    }, ScenarioWorld, WhiteboxContract
};
use tfn_dao::common::config::*;

pub fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        TFN_DAO_PATH_EXPR,
        tfn_dao::ContractBuilder,
    );

    blockchain
}

pub fn setup() -> ScenarioWorld {
    let mut world = world();

    let tfn_dao_whitebox = WhiteboxContract::new(TFN_DAO_ADDRESS_EXPR, tfn_dao::contract_obj);
    let tfn_dao_code = world.code_expression(TFN_DAO_PATH_EXPR);

    world.set_state_step(
        SetStateStep::new()
            .put_account(
                OWNER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance("1_000_000_000_000_000_000")
            )
            .put_account(
                CALLER_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .balance("1_000_000_000_000_000_000")
            )
            .new_address(OWNER_ADDRESS_EXPR, 1, TFN_DAO_ADDRESS_EXPR)
            .put_account(
                TFN_DAO_ADDRESS_EXPR,
                Account::new()
                    .nonce(1)
                    .code(tfn_dao_code.clone())
                    .owner(OWNER_ADDRESS_EXPR)
                    // .esdt_roles(GOVERNANCE_TOKEN_ID_EXPR, roles.clone())
            )
    );

    world.whitebox_call(
        &tfn_dao_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_governance_token(managed_token_id!(GOVERNANCE_TOKEN_ID));
            sc.set_state_active();
        }
    );

    world.whitebox_query(
        &tfn_dao_whitebox, |sc| {
            assert_eq!(sc.state().get(), State::Active);
        }
    );

    world
}

#[test]
fn test_init() {
    setup();
}
