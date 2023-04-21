# House Staking

House staking is a deFi protocol for creating single-token liquidity pools in
which individual liquidity providers own a share of house revenue in proportion
to the amount of liquidity they provide. For example, if you provide 50% of the
house's overall liquidity, then you're entitled to 50% of all revenue it generates.

## Technical Overview

In this repo, House Staking is implemented as a CosmWasm (CW) smart contract and
can run on any Cosmos network in which CW execution is enabled. All functions
have amortized O(1) runtime. To build and deploy this contract, you can use the
Makefile that's provided. More details on this to come.

Each contract represents an organization, or _house_, where eseentially all
functions in the API represent things that the house can do. In a nutshell, it
can receive or send payments, receive delegations, and send tokens to delegators
whenever they claim profit or withdraw their funds.

To minimize volatility in available liquidity, delegations are split between two
distinct internal pools: a _revenue growth_ pool and a _profit_ pool. When a user
delegates, they can specify separate amounts to deposit in either pool.
Moreover, when the house generates revenue, the size of the growth pool
determines the portion of revenue that goes back to the house in the form of
additional liquidity, auto-compounding it. The remaining portion gets set aside
as profit or "rewards" that users can claim at any time, without affecting the
amount of available liquidity.

The core API consists of the following functions:

### Delegate

The contract increases delegation to either or both the growth or profit pool.

### ReceivePayment

The contract receives funds as revenue from an authorized client contract.

### SendPayment

The contract sends funds as an expense incurred by an authorized client contract.

### SendProfit

The contract sends any outstanding claimable profit owed to the claimant.

### Withdraw

The contract removes a delegation account and sends the owner of the delegation any outstanding house revenue and profit.
