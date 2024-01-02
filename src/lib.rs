#![cfg_attr(not(feature = "std"), no_std)]





use concordium_std::*;
use concordium_std::Amount;
use core::fmt::Debug;






/// Struct to represent information about a stake.
#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
pub struct StakeEntry {
    pub staker: AccountAddress,
     pub amount: Amount,
    pub release_time: Timestamp,
}








/// Error types
#[derive(Debug, PartialEq, Eq, Clone, Reject, Serialize, SchemaType)]
pub enum StakingError {
    StakingNotFound,
    InsufficientFunds,
    InvalidPrice,
    InvalidReleaseTime,
    InvalidStakingState,
    #[from(ParseError)]
    ParseParams,
    #[from(TransferError)]
    TransferError,
}









#[derive(Serialize, SchemaType)]
pub struct StakeParams {
    pub staker: AccountAddress,
    pub amount: Amount,
    pub release_time: Timestamp,
}







/// Smart contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi> {
    pub stake_entries: StateMap<u64, StakeEntry, S>,
    pub next_stake_id: u64,
}





// Constants for the reward rate and seconds in a year
const REWARD_RATE: u64 = 100; // 100% APY
const SECONDS_IN_YEAR: u64 = 31536000; // 60 seconds/minute * 60 minutes/hour * 24 hours/day * 365 days/year







impl State {






     fn empty(state_builder: &mut StateBuilder) -> Self {
        State {
            stake_entries: state_builder.new_map(),
            next_stake_id: 1,
        }
    }




}













    /// Init function to initialize the staking state
#[init(contract = "staking_contract")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
    Ok(State {
        stake_entries: state_builder.new_map(),
        next_stake_id: 1,
    })
}










/// Function to handle staking funds
#[receive(contract = "staking_contract", name = "stake_funds", parameter = "StakeParams", mutable)]
fn stake_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {
    let parameter: StakeParams = ctx.parameter_cursor().get()?;
    
    // Store information about the stake in the state
    let stake_info = StakeEntry {
        staker: parameter.staker,
        amount: parameter.amount,
        release_time: parameter.release_time,
    };

    let next_stake_id = host.state_mut().next_stake_id;
    host.state_mut().stake_entries.insert(next_stake_id, stake_info);
    
    // Update next_stake_id for the next stake
    host.state_mut().next_stake_id += 1;

    // ... (perform other staking logic as needed)

    Ok(())
}





