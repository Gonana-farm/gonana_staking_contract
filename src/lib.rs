#![cfg_attr(not(feature = "std"), no_std)]




use concordium_cis2::{Cis2Event, *};
use concordium_std::*;
use concordium_std::Amount;
use core::fmt::Debug;
use chrono::{Local, DateTime, Utc, TimeZone};



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







#[derive(Serialize, SchemaType)]
pub struct ReleaseFundsParams {
    pub contract_address: ContractAddress,
    pub contract_token_id: ContractTokenId
}











/// Smart contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi> {
    pub stake_entries: StateMap<AccountAddress, StakeEntry, S>,
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
#[init(contract = "gonana_staking_contract")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
    Ok(State::empty(state_builder))
}



/// Function to handle staking funds
#[receive(contract = "gonana_staking_contract", name = "stake_funds", parameter = "StakeParams", mutable)]
fn stake_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {
    let parameter: StakeParams = ctx.parameter_cursor().get()?;
    
    // Store information about the stake in the state
    let stake_info = StakeEntry {
        staker: parameter.staker,
        amount: parameter.amount,
        release_time: parameter.release_time,
    };

    host.state_mut().stake_entries.insert(parameter.staker, stake_info);
    
    // Update next_stake_id for the next stake
    host.state_mut().next_stake_id += 1;

    // ... (perform other staking logic as needed)

    Ok(())
}




//Function to release the staked funds
#[receive(contract = "gonana_staking_contract", name = "release_funds",parameter = "ReleaseFundsParams", mutable)]
fn release_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {
    let parameter: ReleaseFundsParams = ctx.parameter_cursor().get()?;

    let stake_entry = host.state_mut().stake_entries.get_mut(&ctx.invoker()).ok_or(StakingError::StakingNotFound)?;
    
    let current_time =  Utc::now();

    //convert release_time to DateTime<Utc>
    let release_time_utc = Utc.timestamp_millis_opt(stake_entry.release_time.timestamp_millis() as i64).unwrap();

    let token_id: ContractTokenId = "";
    let gona_token = ContractAddress::new(7265,0);
    
    // Create a Transfer instance
    let transfer_payload = Transfer{
        token_id,
        amount: stake_entry.amount,
        to: Receiver::Account((stake_entry.staker)),
        from: ctx.self_address(),
        data: AdditionalData::empty()
    };
    let entry_point= OwnedEntrypointName::new_unchecked("transfer".into());

    let mut transfers = Vec::new();
    transfers.push(transfer_payload);
    let payload = TransferEvent::from(transfers);
    //Check if the release time has passed
    if release_time_utc <= current_time {
        host.invoke_contract(&gona_token, &payload, entry_point, Amount::zero())
    }   
    
    Ok(())
}