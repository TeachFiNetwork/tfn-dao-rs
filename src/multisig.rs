multiversx_sc::imports!();

use crate::common::{board_config::*, errors::*};
use crate::proxies::launchpad_proxy::ProxyTrait as _;

#[multiversx_sc::module]
pub trait MultisigModule:
crate::common::board_config::BoardConfigModule
+ crate::common::config::ConfigModule
{
    #[endpoint]
    fn sign(&self, action_id: usize) {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);

        self.action_signers(action_id).insert(caller);
    }

    #[endpoint]
    fn unsign(&self, action_id: usize) {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);

        self.action_signers(action_id).swap_remove(&caller);
    }

    #[endpoint(discardAction)]
    fn discard_action(&self, action_id: usize) {
        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);
        require!(
            self.get_action_valid_signer_count(action_id) == 0,
            "cannot discard action with valid signatures"
        );

        self.action_mapper().clear_entry_unchecked(action_id);
        self.action_signers(action_id).clear();
    }

    fn propose_action(&self, action: BoardAction<Self::Api>) -> usize {
        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);

        let action_id = self.action_mapper().push(&action);
        self.action_signers(action_id).insert(caller);

        action_id
    }

    #[endpoint(proposeAddBoardMember)]
    fn propose_add_board_member(&self, board_member_address: ManagedAddress) -> usize {
        require!(!self.board_members().contains(&board_member_address), ERROR_ALREADY_BOARD_MEMBER);

        self.propose_action(BoardAction::AddBoardMember(board_member_address))
    }

    #[endpoint(proposeRemoveUser)]
    fn propose_remove_user(&self, user_address: ManagedAddress) -> usize {
        require!(self.board_members().contains(&user_address), ERROR_NOT_BOARD_MEMBER);
        require!(self.board_members().len() > 1, ERROR_LAST_BOARD_MEMBER);

        self.propose_action(BoardAction::RemoveBoardMember(user_address))
    }

    #[endpoint(proposeChangeBoardQuorum)]
    fn propose_change_board_quorum(&self, new_quorum: usize) -> usize {
        require!(new_quorum > 0, ERROR_ZERO_VALUE);
        require!(new_quorum <= self.board_members().len(), ERROR_QUORUM_TOO_HIGH);

        self.propose_action(BoardAction::ChangeBoardQuorum(new_quorum))
    }

    #[endpoint(proposeChangeQuorum)]
    fn propose_change_quorum(&self, new_quorum: BigUint) -> usize {
        require!(new_quorum > 0, ERROR_ZERO_VALUE);

        self.propose_action(BoardAction::ChangeQuorum(new_quorum))
    }

    #[endpoint(proposeChangeVotingPeriod)]
    fn propose_change_vorint_period(&self, new_period: u64) -> usize {
        require!(new_period > 0, ERROR_ZERO_VALUE);

        self.propose_action(BoardAction::ChangeVotingPeriod(new_period))
    }

    #[endpoint(proposeAddVotingToken)]
    fn propose_add_voting_token(
        &self,
        token: TokenIdentifier,
        weight: BigUint,
    ) -> usize {
        require!(!self.voting_tokens().contains_key(&token), ERROR_TOKEN_ALREADY_EXISTS);
        require!(weight > 0, ERROR_ZERO_VALUE);

        self.propose_action(BoardAction::AddVotingToken(token, weight))
    }

    #[endpoint(proposeRemoveVotingToken)]
    fn propose_remove_voting_token(&self, token: TokenIdentifier) -> usize {
        require!(self.voting_tokens().contains_key(&token), ERROR_TOKEN_NOT_FOUND);

        self.propose_action(BoardAction::RemoveVotingToken(token))
    }

    #[endpoint(proposeDeleteProposal)]
    fn propose_delete_proposal(&self, proposal_id: u64) -> usize {
        require!(!self.proposals(proposal_id).is_empty(), ERROR_PROPOSAL_NOT_FOUND);

        let proposal = self.proposals(proposal_id).get();
        require!(!proposal.was_executed, ERROR_PROPOSAL_WAS_EXECUTED);
        require!(self.proposal_voters(proposal_id).is_empty(), ERROR_PROPOSAL_VOTERS_NOT_EMPTY);

        self.propose_action(BoardAction::DeleteProposal(proposal_id))
    }

    #[endpoint(proposeUpgradeFranchise)]
    fn propose_upgrade_franchise(
        &self,
        franchise_address: ManagedAddress,
        args: OptionalValue<ManagedArgBuffer<Self::Api>>,
    ) -> usize {
        let franchises = self.franchises().get();
        let mut found = false;
        for franchise in franchises.into_iter() {
            if franchise == franchise_address {
                found = true;
                break;
            }
        }
        require!(found, ERROR_FRANCHISE_NOT_DEPLOYED);

        let upgrade_args = match args {
            OptionalValue::Some(args) => args,
            OptionalValue::None => ManagedArgBuffer::new(),
        };
        self.propose_action(BoardAction::UpgradeFranchise(franchise_address, upgrade_args))
    }

    #[endpoint(performAction)]
    fn perform_action_endpoint(&self, action_id: usize) {
        let caller = self.blockchain().get_caller();
        require!(self.board_members().contains(&caller), ERROR_ONLY_BOARD_MEMBERS);
        require!(self.quorum_reached(action_id), ERROR_QUORUM_NOT_REACHED);

        self.perform_action(action_id)
    }

    fn perform_action(&self, action_id: usize) {
        require!(!self.action_mapper().item_is_empty_unchecked(action_id), ERROR_ACTION_NOT_FOUND);

        let action = self.action_mapper().get(action_id);
        self.action_mapper().clear_entry_unchecked(action_id);
        self.action_signers(action_id).clear();
        match action {
            BoardAction::Nothing=>return,
            BoardAction::AddBoardMember(board_member_address) => {
                require!(!self.board_members().contains(&board_member_address), ERROR_ALREADY_BOARD_MEMBER);

                self.board_members().insert(board_member_address);
            },
            BoardAction::RemoveBoardMember(board_member_address) => {
                require!(self.board_members().len() > 1, ERROR_LAST_BOARD_MEMBER);

                self.board_members().swap_remove(&board_member_address);
                let new_len = self.board_members().len();
                if self.board_quorum().get() > new_len {
                    self.board_quorum().set(new_len);
                }
            },
            BoardAction::ChangeBoardQuorum(new_quorum) => {
                require!(new_quorum <= self.board_members().len(), ERROR_QUORUM_TOO_HIGH);

                self.board_quorum().set(new_quorum);
            },
            BoardAction::ChangeQuorum(new_quorum) => {
                self.quorum().set(new_quorum);
            },
            BoardAction::ChangeVotingPeriod(new_voting_period) => {
                self.voting_period().set(new_voting_period);
            },
            BoardAction::AddVotingToken(token, weight) => {
                require!(!self.voting_tokens().contains_key(&token), ERROR_TOKEN_ALREADY_EXISTS);

                self.voting_tokens().insert(token, weight);
            },
            BoardAction::RemoveVotingToken(token) => {
                require!(self.voting_tokens().contains_key(&token), ERROR_TOKEN_NOT_FOUND);

                self.voting_tokens().remove(&token);
                if self.voting_tokens().is_empty() {
                    self.set_state_inactive();
                }
            },
            BoardAction::DeleteProposal(proposal_id) => {
                require!(!self.proposals(proposal_id).is_empty(), ERROR_PROPOSAL_NOT_FOUND);

                let proposal = self.proposals(proposal_id).get();
                require!(!proposal.was_executed, ERROR_PROPOSAL_WAS_EXECUTED);
                require!(self.proposal_voters(proposal_id).is_empty(), ERROR_PROPOSAL_VOTERS_NOT_EMPTY);

                self.proposals(proposal_id).clear();
                if proposal_id == self.last_proposal_id().get() - 1 {
                    self.last_proposal_id().set(proposal_id);
                }
            },
            BoardAction::UpgradeFranchise(franchise_address, args) => {
                let upgrade_args = if !args.is_empty() {
                    OptionalValue::Some(args)
                } else {
                    OptionalValue::None
                };
                self.launchpad_contract_proxy()
                    .contract(self.launchpad_sc().get())
                    .upgrade_franchise(franchise_address, upgrade_args)
                    .sync_call();
            },
        };
    }
}
