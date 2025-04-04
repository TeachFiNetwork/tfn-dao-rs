multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::common::errors::*;
use super::board_config;
use tfn_platform::common::config::ProxyTrait as _;
use crate::proxies::launchpad_proxy::{self};
use tfn_digital_identity::common::config::Identity;

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Debug, ManagedVecItem)]
pub struct Action<M: ManagedTypeApi> {
    pub gas_limit: u64,
    pub dest_address: ManagedAddress<M>,
    pub payment_token: EgldOrEsdtTokenIdentifier<M>,
    pub payment_amount: BigUint<M>,
    pub endpoint_name: ManagedBuffer<M>,
    pub arguments: ManagedVec<M, ManagedBuffer<M>>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum VoteType {
    Upvote = 1,
    DownVote = 2,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, ManagedVecItem)]
pub enum ProposalStatus {
    Pending, //Starts from 0
    Active,
    Defeated,
    Succeeded,
    Executed,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, ManagedVecItem)]
pub enum ProposalTypeEnum {
    Nothing,

    NewLaunchpad,
    NewTransfer,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Debug)]
pub enum ProposalType<M: ManagedTypeApi> {
    Nothing,

    NewLaunchpad(LaunchpadProposal<M>),
    NewTransfer(TransferProposal<M>),
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct Proposal<M: ManagedTypeApi> {
    pub id: u64,
    pub proposal_data: ProposalType<M>,
    pub proposal_type: ProposalTypeEnum,
    pub creation_timestamp: u64,
    pub proposer: ManagedAddress<M>,
    pub title: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub status: ProposalStatus,
    pub was_executed: bool,
    pub num_upvotes: BigUint<M>,
    pub num_downvotes: BigUint<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Debug)]
pub struct LaunchpadProposal<M: ManagedTypeApi> {
    pub details: Identity<M>,
    pub kyc_enforced: bool,
    pub token: TokenIdentifier<M>,
    pub payment_token: TokenIdentifier<M>,
    pub price: BigUint<M>,
    pub min_buy_amount: BigUint<M>,
    pub max_buy_amount: BigUint<M>,
    pub start_time: u64,
    pub end_time: u64,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Debug)]
pub struct TransferProposal<M: ManagedTypeApi> {
    pub actions: ManagedVec<M, Action<M>>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Clone, Debug)]
pub struct ContractInfo<M: ManagedTypeApi> {
    pub state: State,
    pub governance_token: TokenIdentifier<M>,
    pub voting_tokens: ManagedVec<M, TokenIdentifier<M>>,
    pub voting_token_weights: ManagedVec<M, BigUint<M>>,
    pub voting_period: u64,
    pub quorum: BigUint<M>,
    pub board_quorum: usize,
    pub board_members: ManagedVec<M, ManagedAddress<M>>,
    pub launchpad_sc: ManagedAddress<M>,
    pub last_proposal_id: u64,
    pub proposals_count: u64,
    pub deployed_franchises: ManagedVec<M, ManagedAddress<M>>,
}

#[multiversx_sc::module]
pub trait ConfigModule:
board_config::BoardConfigModule
{
    // state
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        self.only_board_members();
        require!(!self.quorum().is_empty(), ERROR_QUORUM_NOT_SET);
        require!(!self.voting_period().is_empty(), ERROR_VOTING_PERIOD_NOT_SET);
        require!(!self.voting_tokens().is_empty(), ERROR_NO_VOTING_TOKENS);
        require!(!self.platform_sc().is_empty(), ERROR_PLATFORM_NOT_SET);
        require!(!self.launchpad_sc().is_empty(), ERROR_LAUNCHPAD_NOT_SET);
        require!(!self.template_franchise_dao().is_empty(), ERROR_TEMPLATE_ADDRESSES_NOT_SET);

        self.state().set(State::Active);
    }

    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.only_board_members();

        self.state().set(State::Inactive);
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    // governance token
    #[view(getGovernanceToken)]
    #[storage_mapper("governance_token")]
    fn governance_token(&self) -> SingleValueMapper<TokenIdentifier>;

    // voting tokens
    #[view(getVotingTokens)]
    #[storage_mapper("voting_tokens")]
    fn voting_tokens(&self) -> MapMapper<TokenIdentifier, BigUint>;

    // voting period (seconds)
    #[only_owner]
    #[endpoint(setVotingPeriod)]
    fn set_voting_period(&self, period: u64) {
        self.voting_period().set(period);
    }

    #[view(getVotingPeriod)]
    #[storage_mapper("voting_period")]
    fn voting_period(&self) -> SingleValueMapper<u64>;

    // quorum
    #[endpoint(setQuorum)]
    fn set_quorum(&self, quorum: &BigUint) {
        self.only_board_members();

        self.quorum().set(quorum);
    }

    #[view(getQuorum)]
    #[storage_mapper("quorum")]
    fn quorum(&self) -> SingleValueMapper<BigUint>;

    // CONTRACTS
    #[view(getPlatform)]
    #[storage_mapper("platform_sc")]
    fn platform_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[endpoint(setPlatformAddress)]
    fn set_platform_address(&self, address: ManagedAddress) {
        self.only_board_members();

        self.platform_sc().set_if_empty(&address);
        self.platform_contract_proxy()
            .contract(address)
            .set_main_dao()
            .execute_on_dest_context::<()>();
    }

    #[view(getLaunchpadAddress)]
    #[storage_mapper("launchpad_sc")]
    fn launchpad_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[endpoint(setLaunchpadAddress)]
    fn set_launchpad_address(&self, address: ManagedAddress) {
        self.only_board_members();

        self.launchpad_sc().set_if_empty(&address);
        self.launchpad_contract_proxy()
            .contract(address)
            .set_main_dao()
            .execute_on_dest_context::<()>();
    }

    // template dao sc address
    #[view(getTemplateFranchiseDAO)]
    #[storage_mapper("template_franchise_dao")]
    fn template_franchise_dao(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setTemplateFranchiseDAO)]
    fn set_template_franchise_dao(&self, template_franchise_dao: ManagedAddress) {
        self.template_franchise_dao().set(template_franchise_dao);
    }

    #[view(getDigitalIdentityAddress)]
    #[storage_mapper("digital_identity_sc")]
    fn digital_identity_sc(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setDigitalIdentityAddress)]
    fn set_digital_identity_address(&self, address: ManagedAddress) {
        require!(!self.launchpad_sc().is_empty(), ERROR_LAUNCHPAD_NOT_SET);
        require!(!self.platform_sc().is_empty(), ERROR_PLATFORM_NOT_SET);

        self.only_board_members();

        self.digital_identity_sc().set_if_empty(&address);
        self.launchpad_contract_proxy()
            .contract(self.launchpad_sc().get())
            .set_digital_identity(address.clone())
            .execute_on_dest_context::<()>();
        self.platform_contract_proxy()
            .contract(self.platform_sc().get())
            .set_digital_identity(address)
            .execute_on_dest_context::<()>();
    }
    // END CONTRACTS

    // last proposal id
    #[view(getLastProposalId)]
    #[storage_mapper("last_proposal_id")]
    fn last_proposal_id(&self) -> SingleValueMapper<u64>;

    // proposal
    #[view(getProposal)]
    #[storage_mapper("proposals")]
    fn proposals(&self, id: u64) -> SingleValueMapper<Proposal<Self::Api>>;

    // voters amounts
    #[view(getVoterAmount)]
    #[storage_mapper("voters_amounts")]
    fn voters_amounts(&self, voter: &ManagedAddress, proposal_id: u64) -> SingleValueMapper<ManagedVec<EsdtTokenPayment>>;

    #[view(getRedeemableProposalIDs)]
    fn get_redeemable_proposals(&self, user: ManagedAddress) -> ManagedVec<u64> {
        let mut ids: ManagedVec<u64> = ManagedVec::new();
        for idx in self.voter_proposals(&user).iter() {
            if self.proposals(idx).is_empty() {
                continue;
            }

            let proposal = self.proposals(idx).get();
            let payments = self.voters_amounts(&user, idx).get();
            let pstat = self.get_proposal_status(&proposal);
            if (pstat == ProposalStatus::Succeeded || pstat == ProposalStatus::Defeated || pstat == ProposalStatus::Executed) && !payments.is_empty() {
                ids.push(idx);
            }
        }

        ids
    }

    // proposal voters
    #[view(getProposalVoters)]
    #[storage_mapper("proposal_voters")]
    fn proposal_voters(&self, id: u64) -> UnorderedSetMapper<ManagedAddress>;

    // voter proposals
    #[view(getVoterProposals)]
    #[storage_mapper("voter_proposals")]
    fn voter_proposals(&self, voter: &ManagedAddress) -> UnorderedSetMapper<u64>;

    // get number of proposals with the specified type
    #[view(getProposalsCountByType)]
    fn get_proposals_count(&self, status: OptionalValue<ProposalTypeEnum>) -> u64 {
        let all = status.is_none();
        let filter_type = match status {
            OptionalValue::Some(value) => value,
            OptionalValue::None => ProposalTypeEnum::Nothing
        };
        let mut count = 0;
        for idx in 0..self.last_proposal_id().get() {
            if self.proposals(idx).is_empty() {
                continue;
            }

            let proposal = self.proposals(idx).get();
            if all || proposal.proposal_type == filter_type {
                count += 1;
            }
        }

        count
    }

    // view paginated proposals of certain type
    #[view(getProposals)]
    fn get_proposals(&self, idx_from: u64, idx_to: u64, proposal_type: ProposalTypeEnum) -> MultiValueEncoded<Proposal<Self::Api>> {
        let mut proposals = MultiValueEncoded::new();
        let mut real_idx: u64 = 0;
        for idx in 0..self.last_proposal_id().get() {
            if self.proposals(idx).is_empty() {
                continue;
            }

            let mut proposal = self.proposals(idx).get();
            if proposal.proposal_type != proposal_type {
                continue
            }

            if real_idx >= idx_from && real_idx <= idx_to {
                proposal.status = self.get_proposal_status(&proposal);
                proposals.push(proposal);
            }
            real_idx += 1;
        }

        proposals
    }

    // proposal status
    #[view(getProposalStatus)]
    fn get_proposal_status_view(&self, proposal_id: u64) -> ProposalStatus {
        require!(!self.proposals(proposal_id).is_empty(), ERROR_PROPOSAL_NOT_FOUND);

        let proposal = self.proposals(proposal_id).get();
        self.get_proposal_status(&proposal)
    }

    fn get_proposal_status(&self, proposal: &Proposal<Self::Api>) -> ProposalStatus {
        if proposal.was_executed {
            return ProposalStatus::Executed;
        }

        let current_timestamp = self.blockchain().get_block_timestamp();
        let proposal_timestamp = proposal.creation_timestamp;
        let voting_period = self.voting_period().get();

        let voting_start = proposal_timestamp;
        let voting_end = voting_start + voting_period;

        if current_timestamp < voting_start {
            return ProposalStatus::Pending;
        }
        if current_timestamp >= voting_start && current_timestamp < voting_end {
            return ProposalStatus::Active;
        }

        let total_upvotes = &proposal.num_upvotes;
        let total_downvotes = &proposal.num_downvotes;
        let quorum = self.quorum().get();

        if total_upvotes > total_downvotes && total_upvotes - total_downvotes >= quorum {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        }
    }

    // deployed franchises
    #[view(getFranchises)]
    #[storage_mapper("franchises")]
    fn franchises(&self) -> SingleValueMapper<ManagedVec<ManagedAddress>>;

    #[view(isFranchise)]
    fn is_franchise(&self, address: ManagedAddress) -> bool {
        let franchises = self.franchises().get();
        for franchise in franchises.into_iter() {
            if franchise == address {
                return true;
            }
        }

        false
    }

    #[endpoint(franchiseDeployed)]
    fn franchise_deployed(&self, address: ManagedAddress) {
        require!(self.blockchain().get_caller() == self.launchpad_sc().get(), ERROR_ONLY_LAUNCHPAD);

        let mut franchises = self.franchises().get();
        franchises.push(address);
        self.franchises().set(franchises);
    }

    // contract info
    #[view(getContractInfo)]
    fn get_contract_info(&self) -> ContractInfo<Self::Api> {
        let state = self.state().get();
        let governance_token = self.governance_token().get();
        let mut voting_tokens = ManagedVec::new();
        let mut voting_token_weights = ManagedVec::new();
        for (token, weight) in self.voting_tokens().iter() {
            voting_tokens.push(token);
            voting_token_weights.push(weight);
        }
        let voting_period = self.voting_period().get();
        let quorum = self.quorum().get();
        let board_quorum = self.board_quorum().get();
        let mut board_members = ManagedVec::new();
        for member in self.board_members().into_iter() {
            board_members.push(member);
        }
        let launchpad_sc = self.launchpad_sc().get();
        let last_proposal_id = self.last_proposal_id().get();
        let proposals_count = self.get_proposals_count(OptionalValue::None);
        let deployed_franchises = self.franchises().get();

        ContractInfo {
            state,
            governance_token,
            voting_tokens,
            voting_token_weights,
            voting_period,
            quorum,
            board_quorum,
            board_members,
            launchpad_sc,
            last_proposal_id,
            proposals_count,
            deployed_franchises
        }
    }

    // helpers
    fn only_board_members(&self) {
        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);
    }

    // proxies
    #[proxy]
    fn platform_contract_proxy(&self) -> tfn_platform::Proxy<Self::Api>;

    #[proxy]
    fn launchpad_contract_proxy(&self) -> launchpad_proxy::Proxy<Self::Api>;
}
