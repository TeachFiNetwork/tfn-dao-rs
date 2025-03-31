multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait LaunchpadProxy {
    #[endpoint(upgradeFranchise)]
    fn upgrade_franchise(&self, franchise_address: ManagedAddress, args: OptionalValue<ManagedArgBuffer<Self::Api>>);

    #[endpoint(newLaunchpad)]
    fn new_launchpad(
        &self,
        owner: ManagedAddress,
        kyc_enforced: bool,
        description: ManagedBuffer,
        token: TokenIdentifier,
        payment_token: TokenIdentifier,
        price: BigUint, // if payment token is USDC (6 decimals), price should be x_000_000
        min_buy_amount: BigUint,
        max_buy_amount: BigUint,
        start_time: u64,
        end_time: u64
    ) -> u64;
}
