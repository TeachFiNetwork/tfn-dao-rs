multiversx_sc::imports!();

use tfn_digital_identity::common::config::Identity;

#[multiversx_sc::proxy]
pub trait LaunchpadProxy {
    #[endpoint(upgradeFranchise)]
    fn upgrade_franchise(&self, franchise_address: ManagedAddress, args: OptionalValue<ManagedArgBuffer<Self::Api>>);

    #[endpoint(newLaunchpad)]
    fn new_launchpad(
        &self,
        owner: ManagedAddress,
        details: Identity<Self::Api>,
        kyc_enforced: bool,
        token: TokenIdentifier,
        payment_token: TokenIdentifier,
        price: BigUint, // if payment token is USDC (6 decimals), price should be x_000_000
        min_buy_amount: BigUint,
        max_buy_amount: BigUint,
        start_time: u64,
        end_time: u64
    ) -> u64;

    #[view(isTokenLaunched)]
    fn is_token_launched(&self, token: TokenIdentifier) -> bool;

    #[endpoint(setMainDAO)]
    fn set_main_dao(&self);

    #[endpoint(setDigitalIdentity)]
    fn set_digital_identity(&self, address: ManagedAddress);
}
