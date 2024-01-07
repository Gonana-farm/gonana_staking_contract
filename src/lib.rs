#![cfg_attr(not(feature = "std"), no_std)]




use concordium_cis2::*;
use concordium_std::*;
use concordium_std::Amount;
use core::fmt::Debug;


#[derive(Serialize, PartialEq, Eq, Clone, Debug)]
pub struct  ApproveParam {
    pub amount: TokenAmountU64,
    pub spender: Address,
    pub token_id: TokenIdUnit,
 } 




/// smallest possible token ID.
pub type ContractTokenId = TokenIdUnit;
pub type ContractTokenAmount = TokenAmountU64;

pub const TOKEN_ID_GONA:ContractTokenId = TokenIdUnit();


/// Enum representing the possible states of a product
#[derive(Debug, Serialize, SchemaType, PartialEq, Eq, Clone)]
pub enum StakeEntryState {
  Active,
  Inactive
}





/// Struct to represent information about a stake.
#[derive(Serialize, SchemaType, PartialEq, Eq, Clone, Debug)]
pub struct StakeEntry {
    pub staker: AccountAddress,
     pub amount: TokenAmountU64,
    pub release_time: Timestamp,
    pub state: StakeEntryState
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
    ContractInvokeError,
}



impl<A> From<CallContractError<A>> for StakingError {
    fn from(_: CallContractError<A>) -> Self { Self::ContractInvokeError }
}





#[derive(Serialize, SchemaType)]
pub struct StakeParams {
    pub staker: AccountAddress,
    pub amount: ContractTokenAmount,
    pub release_time: Timestamp,
}







#[derive(Serialize, SchemaType)]
pub struct ReleaseFundsParams {
    pub token_id: ContractTokenId
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
#[init(contract = "gonana_staking_smart_contract")]
fn init(_ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
    Ok(State::empty(state_builder))
}



/// Function to handle staking funds
#[receive(contract = "gonana_staking_smart_contract", name = "stake_funds", parameter = "StakeParams", mutable)]
fn stake_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {
    let parameter: StakeParams = ctx.parameter_cursor().get()?;
    
    // Store information about the stake in the state
    let stake_info = StakeEntry {
        staker: parameter.staker,
        amount: parameter.amount,
        release_time: parameter.release_time,
        state: StakeEntryState::Active
    };

    host.state_mut().stake_entries.insert(parameter.staker, stake_info);
    
    // Update next_stake_id for the next stake
    host.state_mut().next_stake_id += 1;

    // ... (perform other staking logic as needed)

    Ok(())
}




//Function to release the staked funds
#[receive(contract = "gonana_staking_smart_contract", name = "release_funds", mutable)]
fn release_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {

    let mut stake_entry = host.state_mut().stake_entries.get_mut(&ctx.invoker()).ok_or(StakingError::StakingNotFound)?;
    
    // Ensure that the product is in a valid state for confirming the escrow
    ensure!(stake_entry.state == StakeEntryState::Active, StakingError::InvalidStakingState);

    let token_id = TokenIdUnit();
    let gona_token = ContractAddress::new(7643,0);
    
    // Create a Transfer instance
    let transfer_payload = Transfer{
        token_id,
        amount: stake_entry.amount,
        to: Receiver::Account(stake_entry.staker),
        from: Address::Contract(ctx.self_address()),
        data: AdditionalData::empty()
    };
    let entry_point= EntrypointName::new_unchecked("transfer".into());

    let mut transfers = Vec::new();
    transfers.push(transfer_payload);
    let payload = TransferParams::from(transfers);
    stake_entry.state = StakeEntryState::Inactive;
    drop(stake_entry);
    //Check if the release time has passed
   
        host.invoke_contract(&gona_token, &payload, entry_point, Amount::zero())?;   
    
    Ok(())
}







/// Function to get stake information by ID
#[receive(
    contract = "gonana_staking_smart_contract",
    name = "get_stake_info",
    parameter = "AccountAddress",
    return_value = "Option<StakeEntry>"
)]
fn get_stake_info(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Option<StakeEntry>>{
    let param : AccountAddress= ctx.parameter_cursor().get()?;
    
      let stake_entry_ref = host.state().stake_entries.get(&param);

      // Convert the StateRef to Option<StakeEntry>
      let stake_entry_option = stake_entry_ref.map(|entry_ref| entry_ref.to_owned());
  
      Ok(stake_entry_option)
}




//Module successfully deployed with reference: '2eadfae54e3f063c5bda0a27129390c0dd8ebdb2f3196edb0b0d3743f9bdb5ee'.
//Module reference 2eadfae54e3f063c5bda0a27129390c0dd8ebdb2f3196edb0b0d3743f9bdb5ee was successfully named 'gonana_staking__module'.
//Module successfully deployed with reference: 'b2584adc2a4fec426cb16ee891fb0183525628412f8209acec2d32d0e0c2f2b1'.
//Module reference b2584adc2a4fec426cb16ee891fb0183525628412f8209acec2d32d0e0c2f2b1 was successfully named 'gonana_staking_smart_contract'.
//{"index":7644,"subindex":0}