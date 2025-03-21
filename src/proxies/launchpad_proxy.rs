multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait LaunchpadProxy {
    #[endpoint(upgradeFranchise)]
    fn upgrade_franchise(&self, franchise_address: ManagedAddress, args: OptionalValue<ManagedArgBuffer<Self::Api>>);
}
