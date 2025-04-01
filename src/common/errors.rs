pub static ERROR_NOT_ACTIVE: &[u8] = b"contract is not active";
pub static ERROR_QUORUM_NOT_SET: &[u8] = b"quorum not set";
pub static ERROR_VOTING_PERIOD_NOT_SET: &[u8] = b"voting period not set";
pub static ERROR_PROPOSAL_AMOUNT_NOT_SET: &[u8] = b"minimum proposal amount not set";
pub static ERROR_LAUNCHPAD_NOT_SET: &[u8] = b"franchises launchpad SC address not set";
pub static ERROR_LAUNCHPAD_ALREADY_SET: &[u8] = b"launchpad SC address already set";
pub static ERROR_INVALID_PAYMENT: &[u8] = b"wrong payment token";
pub static ERROR_ZERO_PAYMENT: &[u8] = b"zero payments are not allowed";
pub static ERROR_NOT_ENOUGH_FUNDS_TO_PROPOSE: &[u8] = b"not enough funds to create proposal";
pub static ERROR_PROPOSAL_NOT_FOUND: &[u8] = b"proposal does not exist";
pub static ERROR_PROPOSAL_NOT_ACTIVE: &[u8] = b"proposal is not active";
pub static ERROR_VOTING_PERIOD_NOT_ENDED: &[u8] = b"voting period not ended";
pub static ERROR_PROPOSAL_NOT_SUCCEEDED: &[u8] = b"proposal not succeeded";
pub static ERROR_ONLY_LAUNCHPAD: &[u8] = b"only the launchpad SC can call this function";
pub static ERROR_ONLY_BOARD_MEMBERS: &[u8] = b"only board members";
pub static ERROR_NOTHING_TO_REDEEM: &[u8] = b"nothing to redeem";
pub static ERROR_FRANCHISE_NOT_DEPLOYED: &[u8] = b"franchise not deployed";
pub static ERROR_NO_VOTING_TOKENS: &[u8] = b"no voting tokens set";
pub static ERROR_ALREADY_BOARD_MEMBER: &[u8] = b"already board member";
pub static ERROR_NOT_BOARD_MEMBER: &[u8] = b"not board member";
pub static ERROR_ZERO_VALUE: &[u8] = b"value cannot be zero";
pub static ERROR_TOKEN_ALREADY_EXISTS: &[u8] = b"token already exists";
pub static ERROR_TOKEN_NOT_FOUND: &[u8] = b"token not found";
pub static ERROR_QUORUM_TOO_HIGH: &[u8] = b"quorum too high";
pub static ERROR_ACTION_NOT_FOUND: &[u8] = b"action not found";
pub static ERROR_LAST_BOARD_MEMBER: &[u8] = b"cannot remove last board member";
pub static ERROR_INVALID_LAUNCHPAD_TIMES: &[u8] = b"invalid launchpad start / end times";
pub static ERROR_PROPOSAL_WAS_EXECUTED: &[u8] = b"proposal was already executed";
pub static ERROR_PROPOSAL_VOTERS_NOT_EMPTY: &[u8] = b"proposal voters not empty";
pub static ERROR_WRONG_BUY_AMOUNTS: &[u8] = b"wrong buy amounts";
pub static ERROR_TOKEN_ALREADY_LAUNCHED: &[u8] = b"token already on launchpad";
