multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::common::errors::*;

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[multiversx_sc::module]
pub trait ConfigModule {
    // state
    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        require!(!self.governance_token().is_empty(), ERROR_TOKEN_NOT_SET);

        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    // governance token
    #[only_owner]
    #[endpoint(setGovernanceToken)]
    fn set_governance_token(&self, token: TokenIdentifier) {
        require!(self.governance_token().is_empty(), ERROR_TOKEN_ALREADY_SET);

        self.governance_token().set(token);
    }

    #[view(getGovernanceToken)]
    #[storage_mapper("governance_token")]
    fn governance_token(&self) -> SingleValueMapper<TokenIdentifier>;
}
