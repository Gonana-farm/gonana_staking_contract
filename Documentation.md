# Gonana Staking Smart Contract Documentation

# Contract Address

The Gonana Marketplace Smart Contract is deployed at the following address:

<Contract Address 7669>

# Getting Started

Users need to call the approve endpoint on the gona-token contract to give permission to the Gonana Staking Smart Contract to spend tokens.

The gonana staking contract works thus:

- User calls the approve endpoint from the gona-token contract to give permission to the Gonana Staking Smart Contract to spend tokens.
- Users initiate staking by calling the stake_funds endpoint on the Gonana Staking Smart Contract.
- After a certain staking period, users can release their staked funds along with earned rewards by calling the release_funds endpoint on the Gonana Staking Smart Contract.

## Entrypoints

### `approve`

- **Description:** Allows users to approve the Gonana Staking Smart Contract to spend tokens on their behalf.
- **Parameters:** `ApproveParam`
- **Mutability:** Mutable
- Sample `ApproveParam`:
  amount: Token amount to approve.
  spender: Address of the Gonana Staking Smart Contract.
  token_id: Token ID. `token_id = TokenIdUnit();`

Example JSON

json

{
"amount": 100,
"spender": "0x1234567890123456789012345678901234567890",
"token_id": token_id
}

### `stake_funds`

- **Description:** Initiates the staking pool for a user by staking GONA tokens.
- **Parameters:** `StakeParams`
- **Mutability:** Mutable
- Sample `StakeParams`:

  staker: AccountAddress of the user initiating the stake..
  amount: Amount of GONA tokens to stake.

Example JSON

json

{
"staker": "acc1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0"
"amount": 50
}

### `release_funds`

- **Description:** Releases staked funds along with earned rewards after a certain staking period.
- **Parameters:** No specific parameters. The caller is identified implicitly.
- **Mutability:** Mutable

### `get_stake_info`

- **Description:** Retrieves information about a staking entry.
- **Parameters:** `AccountAddress`
- **Mutability:** Mutable
- Sample Parameter:
  staker: AccountAddress of the user for whom to retrieve stake information.

Example JSON

json

{
"staker": "acc9a8b7c6d5e4f3g2h1i0j9k8l7m6n5o4p3q2r1s0"
}
