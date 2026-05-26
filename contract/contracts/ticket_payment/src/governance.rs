use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterChange {
    AddGovernor(Address),
    RemoveGovernor(Address),
    AddTokenToWhitelist(Address),
    RemoveTokenFromWhitelist(Address),
    UpdateWithdrawalCap(Address, i128), // This is still i128 amount
    UpdateSlippage(u32),
    UpdateTransferFee(String, u32), // Changed from i128 to u32 basis points
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Executed,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterProposal {
    pub id: u64,
    pub proposer: Address,
    pub change: ParameterChange,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub vote_count: u32,
    pub voters: soroban_sdk::Vec<Address>,
}