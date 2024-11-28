#![no_std]

multiversx_sc::imports!();

pub mod common;

#[multiversx_sc::contract]
pub trait TFNDAOContract<ContractReader>:
    common::config::ConfigModule
{
    #[init]
    fn init(&self) {
        self.set_state_inactive();
    }

    #[upgrade]
    fn upgrade(&self) {
        self.set_state_inactive();
    }
}
