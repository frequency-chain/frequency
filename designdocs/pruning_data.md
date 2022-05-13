# Pruning Data

MRC is focused on being able to send a high volume of messages.
To this end, we must worry about the cost of storing those messages over time.
As of right now, Ethereum has an [archive node](https://etherscan.io/chartsync/chainarchive) storage requirement of over 10TB of data.
This makes running an archive node expensive to do.
How is MRC going to solve this problem?

## Where is Data?

We have two data storage locations that we can adjust.
First, we have the historical state in blocks.
For Ethereum this would be log and transaction information.
For MRC, it is extrinsic information and prior state information.

Second, is the active state.
For Ethereum, this would be balances, smart contracts, and smart contract state.
For MRC, this is balances, schemas, messages, and to a lesser degree staking information and governance voting.

## Historical Block Pruning

Many blockchains hold that anyone should be able to do a complete validation of the chain from the genesis block.
If you want to pull all 10TB+ of data down (after several restarts), you can test to make sure that each state transition of each transaction and block were applied correctly and that it matches the rest of the world.
Or you could just be a "full node" pull the block headers, the current state, and the past few hundred blocks for about 1TB of data.
The full node version trusts that the information prior to the few hundred blocks is good.

MRC is structured such that we can survive without any historical block data beyond the last few hundred.
Effectively, we don't require archive nodes.
Someone could run one, or at least run one that starts from the time they started it, but the network functions without it.
Also as we still maintain block headers, a historical block is still able to be validated.

Effectively, after a given point in time, we trust that a given block was validated correctly by consensus and by the [relay chain](https://polkadot.network/technology/).
The relay chain's secure validation provides a greater degree of security for MRC without having to rely on the MRC's collator nodes.

The outcome of this choice does mean that slightly more information is held in active state.
Messages store additional information that would normally be stored in the transaction alone, because that transaction might not be retrievable after some time.
The total storage require is still reduced.

## Active State Pruning

Since we have historical block pruning, we have to be extra careful what we prune from active state.
Any information we prune, must be expected to be lost forever.
However that is also a benefit.
MRC can be structured such that selected information is forgotten.

We leverage this with allowing schemas to be configured with a time to live or [TTL](https://en.wikipedia.org/wiki/Time_to_live) for messages.
After some number of blocks, MRC is able to discard those messages.
How this is accomplished becomes a tradeoff between storage and computation.

## Handling Growth

We are still left with a net growth in information and storage needs.
The growth of storage needs does not however mean that the costs to operate MRC would increase.
As technology improves, the cost of storage falls.
As long as the growth of MRC data does not greatly outstrip the falling cost of storage, the problem is pushed out into the future.

## Future Ideas

Blockchain in general is struggling with this problem.
There are a lot of ideas about how to handle the increasing storage.
Some are optimizations around storage or compression.
Some are ideas around checkpoint and archiving.

We think MRC is well situated to take advantage of those and other new ideas.
