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

Each contract represents an organization, or _house_, where essentially all API
functions represent capabilities of the house. In a nutshell, the house can
receive or send payments, modify delegations, and transfer tokens when a user
takes profit or withdraws their stake.

To minimize volatility in available liquidity, delegations are split between two
internal pools: a _revenue growth_ pool and a _profit_ pool. When a user
delegates, they specify distinct amounts to add to each pool.

Moreover, when the house generates revenue, the size of the growth pool
determines the portion of revenue that goes back to the house in the form of
additional liquidity, auto-compounding it. The revenue left over gets set aside
as profit that users can claim at any time, proportional to their delegation,
without impacting the available liquidity of the house.

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
