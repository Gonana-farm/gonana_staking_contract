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




#[derive(Serialize, PartialEq, Eq, Clone, Debug)]
pub struct SpendParam {
  pub amount: TokenAmountU64,
  pub owner: Address,
  pub token_id: TokenIdUnit,
} 



impl SpendParam {

     fn new(amount: TokenAmountU64, owner: Address, token_id: TokenIdUnit) -> Self {
        SpendParam { amount, owner, token_id }
    }

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
const REWARD_RATE: u64 = 1; // 100% APY




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

    let amount = parameter.amount;
    let owner = ctx.sender();
    let token_id = TokenIdUnit();
    let gona_token = ContractAddress::new(7656,0);
    let entry_point= EntrypointName::new_unchecked("transfer_from".into());
    let spend_param = SpendParam::new(amount, owner, token_id);

    

    // Check if an AccountAddress has staked before 
    if let Some(mut stake_entry) = host.state_mut().stake_entries.remove_and_get(&ctx.invoker()){
             // Ensure that the stake-entry is in an Active state 
             ensure!(stake_entry.state == StakeEntryState::Active, StakingError::InvalidStakingState);  
             host.invoke_contract(&gona_token, &spend_param, entry_point, Amount::zero())?; 
             stake_entry.amount += amount; 
             stake_entry.release_time = ctx.metadata().block_time()
    } else {
        // If an AccountAddress has not staked before go ahead to stake_funds
            host.invoke_contract(&gona_token, &spend_param, entry_point, Amount::zero())?;   
    
        // Store information about the stake in the state
        let stake_info = StakeEntry {
            staker: parameter.staker,
            amount: parameter.amount,
            release_time: ctx.metadata().block_time(),
            state: StakeEntryState::Active
        };

        host.state_mut().stake_entries.insert(parameter.staker, stake_info);
    
        // Update next_stake_id for the next stake
        host.state_mut().next_stake_id += 1;
    }

    Ok(())
}




//Function to release the staked funds
#[receive(contract = "gonana_staking_smart_contract", name = "release_funds", mutable)]
fn release_funds(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), StakingError> {

    let mut stake_entry = host.state_mut().stake_entries.get_mut(&ctx.invoker()).ok_or(StakingError::StakingNotFound)?;
    
    // Ensure that the stake-entry is in a valid state for releasing the funds
    ensure!(stake_entry.state == StakeEntryState::Active, StakingError::InvalidStakingState);
    let time = ctx.metadata().block_time().duration_since(stake_entry.release_time).unwrap();

    let seconds = time.seconds();
    let token_id = TokenIdUnit();
    let gona_token = ContractAddress::new(7656,0);

  
    let reward_amount =  REWARD_RATE * seconds + stake_entry.amount.0 ;
    let reward = TokenAmountU64(reward_amount);
    
    // Create a Transfer instance
    let transfer_payload = Transfer{
        token_id,
        amount: reward,
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

//{"index":7658,"subindex":0}
// {"index":7659,"subindex":0}


//approve
//call the stake_funds to initiate the staking pool
//release the funds