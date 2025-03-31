#![no_std]

use common::{config::*, consts::*, errors::*};
use crate::proxies::launchpad_proxy::ProxyTrait as _;

multiversx_sc::imports!();

pub mod common;
pub mod multisig;
pub mod proxies;

#[multiversx_sc::contract]
pub trait TFNDAOContract<ContractReader>:
common::config::ConfigModule
+ common::board_config::BoardConfigModule
+ multisig::MultisigModule
{
    #[init]
    fn init(
        &self,
        governance_token: TokenIdentifier,
    ) {
        self.board_members().insert(self.blockchain().get_caller());
        self.governance_token().set(&governance_token);
        self.voting_tokens().insert(governance_token, BigUint::from(ONE));
    }

    #[upgrade]
    fn upgrade(&self) {
    }

    // dummy endpoint for adding funds to the DAO
    #[payable("*")]
    #[endpoint(addFunds)]
    fn add_funds(&self) {}

    #[endpoint(proposeNewLaunchpad)]
    fn propose_new_launchpad(
        &self,
        title: ManagedBuffer,
        description: ManagedBuffer,
        launchpad_proposal: LaunchpadProposal<Self::Api>,
    ) -> u64 {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);

        let proposal = Proposal {
            id: self.last_proposal_id().get(),
            proposal_data: ProposalType::NewLaunchpad(launchpad_proposal),
            proposal_type: ProposalTypeEnum::NewLaunchpad,
            creation_timestamp: self.blockchain().get_block_timestamp(),
            proposer: caller,
            title,
            description,
            status: ProposalStatus::Pending,
            was_executed: false,
            num_upvotes: BigUint::zero(),
            num_downvotes: BigUint::zero(),
        };
        self.proposals(proposal.id).set(&proposal);
        self.last_proposal_id().set(proposal.id + 1);

        proposal.id
    }

    #[endpoint(proposeNewTransfer)]
    fn propose_new_transfer(
        &self,
        title: ManagedBuffer,
        description: ManagedBuffer,
        actions: ManagedVec<Action<Self::Api>>,
    ) -> u64 {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);

        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);

        let transfer_proposal = TransferProposal {
            actions,
        };
        let proposal = Proposal {
            id: self.last_proposal_id().get(),
            proposal_data: ProposalType::NewTransfer(transfer_proposal),
            proposal_type: ProposalTypeEnum::NewTransfer,
            creation_timestamp: self.blockchain().get_block_timestamp(),
            proposer: caller,
            title,
            description,
            status: ProposalStatus::Pending,
            was_executed: false,
            num_upvotes: BigUint::zero(),
            num_downvotes: BigUint::zero(),
        };
        self.proposals(proposal.id).set(&proposal);
        self.last_proposal_id().set(proposal.id + 1);

        proposal.id
    }

    #[payable("*")]
    #[endpoint(upvote)]
    fn upvote(&self, proposal_id: u64) {
        self.vote(proposal_id, VoteType::Upvote)
    }

    #[payable("*")]
    #[endpoint(downvote)]
    fn downvote(&self, proposal_id: u64) {
        self.vote(proposal_id, VoteType::DownVote)
    }

    fn vote(&self, proposal_id: u64, vote_type: VoteType) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.proposals(proposal_id).is_empty(), ERROR_PROPOSAL_NOT_FOUND);

        let mut proposal = self.proposals(proposal_id).get();
        let pstat = self.get_proposal_status(&proposal);
        require!(pstat == ProposalStatus::Active, ERROR_PROPOSAL_NOT_ACTIVE);

        let payment = self.call_value().single_esdt();
        require!(self.voting_tokens().contains_key(&payment.token_identifier), ERROR_INVALID_PAYMENT);
        require!(payment.amount > 0, ERROR_ZERO_PAYMENT);

        let vote_weight = payment.amount.clone() * self.voting_tokens().get(&payment.token_identifier).unwrap() / ONE;
        match vote_type {
            VoteType::Upvote => proposal.num_upvotes += vote_weight.sqrt(),
            VoteType::DownVote => proposal.num_downvotes += vote_weight.sqrt(),
        }
        self.proposals(proposal_id).set(&proposal);

        let caller = self.blockchain().get_caller();
        self.proposal_voters(proposal.id).insert(caller.clone());
        self.voter_proposals(&caller).insert(proposal.id);
        
        // update the amount of tokens voted by the caller
        let mut new_vec: ManagedVec<EsdtTokenPayment> = ManagedVec::new();
        let old_vec = self.voters_amounts(&caller, proposal.id).get();
        let mut found = false;
        for old_payment in old_vec.iter() {
            if old_payment.token_identifier == payment.token_identifier && old_payment.token_nonce == payment.token_nonce {
                new_vec.push(EsdtTokenPayment::new(
                    payment.token_identifier.clone(),
                    payment.token_nonce,
                    &old_payment.amount + &payment.amount,
                ));
                found = true;
            } else {
                new_vec.push(old_payment.clone());
            }
        }
        if !found {
            new_vec.push(payment.clone());
        }
        self.voters_amounts(&caller, proposal.id).set(&new_vec);
    }

    #[endpoint(redeem)]
    fn redeem(&self, proposal_id: u64) {
        let proposal = self.proposals(proposal_id).get();
        let pstat = self.get_proposal_status(&proposal);
        require!(
            pstat == ProposalStatus::Succeeded || pstat == ProposalStatus::Defeated || pstat == ProposalStatus::Executed,
            ERROR_VOTING_PERIOD_NOT_ENDED,
        );

        let caller = self.blockchain().get_caller();
        let payments = self.voters_amounts(&caller, proposal_id).take();
        self.voter_proposals(&caller).swap_remove(&proposal_id);
        self.proposal_voters(proposal_id).swap_remove(&caller);
        require!(!payments.is_empty(), ERROR_NOTHING_TO_REDEEM);

        self.send().direct_multi(&caller, &payments);
    }

    #[endpoint(execute)]
    fn execute(&self, proposal_id: u64) {
        require!(self.state().get() == State::Active, ERROR_NOT_ACTIVE);
        require!(!self.proposals(proposal_id).is_empty(), ERROR_PROPOSAL_NOT_FOUND);

        let mut proposal = self.proposals(proposal_id).get();
        let pstat = self.get_proposal_status(&proposal);
        require!(pstat == ProposalStatus::Succeeded, ERROR_PROPOSAL_NOT_SUCCEEDED);

        self.execute_proposal(&proposal);
        proposal.was_executed = true;
        self.proposals(proposal_id).set(&proposal);
    }

    fn execute_proposal(&self, proposal: &Proposal<Self::Api>) {
        match proposal.proposal_data.clone() {
            ProposalType::Nothing => return,

            ProposalType::NewLaunchpad(launchpad_proposal) => {
                self.launchpad_contract_proxy()
                    .contract(self.launchpad_sc().get())
                    .new_launchpad(
                        proposal.proposer.clone(),
                        launchpad_proposal.kyc_enforced,
                        proposal.description.clone(),
                        launchpad_proposal.token,
                        launchpad_proposal.payment_token,
                        launchpad_proposal.price,
                        launchpad_proposal.min_buy_amount,
                        launchpad_proposal.max_buy_amount,
                        launchpad_proposal.start_time,
                        launchpad_proposal.end_time,
                    )
                    .sync_call();
            },

            ProposalType::NewTransfer(transfer_proposal) => {
                for action in transfer_proposal.actions.iter() {
                    self.execute_action(&action).unwrap();
                }
            },
        };
        // self.execute_action(&proposal.action).unwrap()
    }

    fn execute_action(&self, action: &Action<Self::Api>) -> Result<(), &'static [u8]> {
        let payment =
            EgldOrEsdtTokenPayment::new(action.payment_token.clone(), 0, action.payment_amount.clone());
        if action.payment_amount > 0 {
            self.send()
                .contract_call::<()>(action.dest_address.clone(), action.endpoint_name.clone())
                .with_egld_or_single_esdt_transfer(payment)
                .with_raw_arguments(ManagedArgBuffer::from(action.arguments.clone()))
                .with_gas_limit(action.gas_limit)
                .transfer_execute();
        } else {
            self.send()
                .contract_call::<()>(action.dest_address.clone(), action.endpoint_name.clone())
                .with_raw_arguments(ManagedArgBuffer::from(action.arguments.clone()))
                .with_gas_limit(action.gas_limit)
                .transfer_execute();
        }

        Result::Ok(())
    }
}
