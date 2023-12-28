#![cfg_attr(not(feature = "std"), no_std)]





use concordium_std::*;
use concordium_std::Amount;
use core::fmt::Debug;







/// Enum representing the possible states of a staking entry
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub enum StakingState {
    Active,
    Inactive,
}





/// Struct to represent a staking entry.
#[derive(Serialize, Clone, SchemaType, PartialEq, Eq, Debug)]
pub struct StakingEntry {
    pub staker: AccountAddress,
    pub amount: Amount,
    pub release_time: Timestamp,
    pub state: StakingState,
    pub reward: Amount
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
pub struct StakingParameter {
    pub staking_amount: Amount,
    pub release_time: Timestamp,
}







/// Smart contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
pub struct State<S = StateApi> {
    pub staking_entries: StateMap<AccountAddress, StakingEntry, S>,
}






// Constants for the reward rate and seconds in a year
const REWARD_RATE: u64 = 100; // 100% APY
const SECONDS_IN_YEAR: u64 = 31536000; // 60 seconds/minute * 60 minutes/hour * 24 hours/day * 365 days/year







impl State {






    fn empty(state_builder: &mut StateBuilder) -> Self{
        State {
            staking_entries: state_builder.new_map(),
        }
    }














    fn stake_funds(&mut self, ctx: &ReceiveContext,host: &mut Host<State>, staking_amount: Amount, release_time: Timestamp ) -> Result<(), StakingError> {

         // Check if the release time is in the future
      if release_time <= ctx.metadata().block_time() {
            return Err(StakingError::InvalidReleaseTime);
      };

            let staking_entry = StakingEntry {
                staker: ctx.invoker(),
                amount: staking_amount,
                release_time,
                state: StakingState::Active,
                reward: Amount::zero()
            };
        
            // Insert the staking entry
            host.state_mut().staking_entries
                .insert(ctx.invoker(), staking_entry);
        
        Ok(())
    }










    fn release_funds(&mut self, ctx: &ReceiveContext, host: &mut Host<Self>) -> Result<(), StakingError> {
        if let Some(mut staking_entry) = self.staking_entries.remove_and_get(&ctx.invoker()) {

            // Ensure the staking entry state is active
            ensure!(staking_entry.state == StakingState::Active, StakingError::InvalidStakingState);

            // Check if the release time has passed
            if staking_entry.release_time > ctx.metadata().block_time() {
                return Err(StakingError::InvalidReleaseTime);
            }

            // Calculate reward
            let elapsed_time = ctx.metadata().block_time().timestamp_millis() - staking_entry.release_time.timestamp_millis();
            let reward_amount =
                (staking_entry.amount.micro_ccd() * REWARD_RATE * elapsed_time) / SECONDS_IN_YEAR;
            staking_entry.reward += Amount::from_micro_ccd(reward_amount);

            
            let total_amount = staking_entry.amount + staking_entry.reward;

           // Transfer staked funds and reward back to the staker
            host.invoke_transfer(&staking_entry.staker, total_amount)?;

            // It's important to call `Deletable::delete` on the value
            staking_entry.delete();

            Ok(())
        } else {
            // Escrow not found
            Err(StakingError::StakingNotFound)
        }
    }



}








// Init function to initialize the staking state
#[init(contract = "gonana_staking_contract")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
    Ok(State::empty(state_builder))
}













/// Function to stake funds
#[receive(
    contract = "gonana_staking_contract",
    name = "stake_funds",
    parameter = "StakingParameter",
    mutable,
    payable
)]
fn stake_funds(ctx: &ReceiveContext, host: &mut Host<State>, amount: Amount) -> Result<(), StakingError> {
    let parameter: StakingParameter = ctx.parameter_cursor().get()?;

    ensure!(amount == parameter.staking_amount,  StakingError::InvalidPrice);

   // Check if the release time is in the future
   if parameter.release_time <= ctx.metadata().block_time() {
    return Err(StakingError::InvalidReleaseTime);
}

    let staking_entry = StakingEntry {
        staker: ctx.invoker(),
        amount: parameter.staking_amount,
        release_time: parameter.release_time,
        state: StakingState::Active,
        reward: Amount::zero()
    };

    // Insert the staking entry
    host.state_mut()
        .staking_entries
        .insert(ctx.invoker(), staking_entry);

   

    Ok(())
}
















/// Function to release staked funds
#[receive(
    contract = "gonana_staking_contract",
    name = "release_funds",
    parameter = "StakingParameter",
    mutable
)]
fn release_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {
   
    if let Some( mut staking_entry) = host.state_mut().staking_entries.remove_and_get(&ctx.invoker()) {
        
        ensure!(staking_entry.state == StakingState::Active, StakingError::InvalidStakingState);
        // Check if the release time has passed
            if staking_entry.release_time > ctx.metadata().block_time() {
                return Err(StakingError::InvalidReleaseTime);
            }

             // Calculate reward
        let elapsed_time = ctx.metadata().block_time().timestamp_millis() - staking_entry.release_time.timestamp_millis();
        let reward_amount =
            (staking_entry.amount.micro_ccd() * REWARD_RATE * elapsed_time) / SECONDS_IN_YEAR;
        staking_entry.reward += Amount::from_micro_ccd(reward_amount);

         // Transfer staked funds and reward back to the staker
         let total_amount = staking_entry.amount + staking_entry.reward;

        // Transfer staked funds back to the staker
        host.invoke_transfer(&staking_entry.staker, total_amount)?;

        // It's important to call `Deletable::delete` on the value
        staking_entry.delete();

        Ok(())
    } else {
        // Staking_entry not found
        Err(StakingError::StakingNotFound)
    }

 
}
















// View function to get all staking entries
#[receive(
    contract = "gonana_staking_contract",
    name = "view_staking_entries",
    return_value = "Vec<StakingEntry>"
)]
fn view_staking_entries(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Vec<StakingEntry>> {
    let state = host.state();
    let staking_entries: Vec<StakingEntry> = state
        .staking_entries
        .iter()
        .map(|(_, entry)| entry.clone())
        .collect();
    Ok(staking_entries)
}

