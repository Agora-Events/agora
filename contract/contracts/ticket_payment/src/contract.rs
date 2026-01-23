use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TicketPaymentContract;

#[contractimpl]
impl TicketPaymentContract {
    pub fn init(_env: Env) {}
}
