# Payment channels, Intro to Lightning, and HTLCs

**Q. What are timelocks?**
- smart contract primitive
- time-based conditions which restricts spending until a future specified time or block height.
- 3 important attributes⏤location, targeting, and metric.

**Q. Why use timelocks?**
- Used in smart contracts like payment channels and HTLCs
- Can be used to lock-up bitcoin held as an investment for months or years
- Also used to make [fee sniping](https://en.bitcoin.it/wiki/Fee_sniping) less profitable, and for [trustless precomputed fee bumping](https://en.bitcoin.it/wiki/Techniques_to_reduce_transaction_fees#Pre-computed_fee_bumping)

**Q. What are the different locations for timelocks?**
- Transaction: 
	- Found in the tx itself.
	- Determine when a tx can be confirmed.
	- Cause a tx to be invalid until a certain time, regardless of validity of signatures and scripts.
	- Eg. future-dating a check - only applicable to that tx
	- Eg. `nLocktime` (absolute) and `nSequence` (relative)
	- `nLocktime`
		- transaction-level
		- if `nLocktime` = 0 -> immediate propogation and execution
		- if `nLocktime` != 0 and < 500 million, it is block height
		- if `nLocktime` >= 500 million, unix epoch timestamp
		- No guarantee that original signer of nLocktime tx will not double-spend the same UTXO before `nLocktime` specified time. Need `CLTV`/script locktime for that.
	- `nSequence`
		- input-level
		- invalidate txs based on the time elapsed since the previous outputs' confirmations.
		- is 32-bits long, but we use only 18-bits. 14 bits are reserved for future uses.
		- 2 bits are flags
			- most significant bit is disable flag
			- type-flag is set to 1 << 22 or (2^22) (bit 22)
			- if type-flag exists, nSequence = multiple of 512 seconds, else number of blocks
- Script
	- Found in associated scripts of Pay to Script Hash (P2SH) inputs
	- Scripts have 0 or n timelock fields.
	- Causes scriptsig to be invalid. 
	- Applies to all transactions spending an output i.e. an input
	- Eg. `OP_CLTV` (absolute) and `OP_CSV` (relative)
	- `OP_CLTV` -> [https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
	- `OP_CSV` -> [https://github.com/bitcoin/bips/blob/master/bip-0068.mediawiki](https://github.com/bitcoin/bips/blob/master/bip-0068.mediawiki)

Transaction-level locks affect what you can do with a transaction after it’s constructed, but Script-level locks determine what transactions can be made in the first place.

**Q. What are the different targeting and metrics for timelocks?**

- Targeting: Absolute vs relative
- Metrics: Blocks vs Seconds

**Q. What is P2SH?**

It's a type of ScriptPubKey which allows for the spending of bitcoin based on the satisfaction of the script whose hash is specified within the transaction.

**Q. What is CSV and CLTV?**

CLTV -> `OP_CHECKLOCKTIMEVERIFY` allows transaction outputs (rather than whole transactions) to be encumbered with a timelock. When the CLTV opcode is called, it will cause the script to fail unless the nLockTime on the transaction is equal to or greater than the time parameter provided to the CLTV opcode. Since a transaction may only be included in a valid block if its nLockTime is in the past, this ensures the CLTV-based timelock has expired before the transaction may be included in a valid block. [https://en.bitcoin.it/wiki/Timelock#CheckLockTimeVerify](https://en.bitcoin.it/wiki/Timelock#CheckLockTimeVerify)

CSV -> `OP_CHECKSEQUENCEVERIFY` (a.k.a `OP_RELATIVECHECKLOCKTIMEVERIFY`) checks if an input's sequence  number is smaller than the sequence threshold (1 << 31 = 2147483648) and if so, it will compare the `nLocktime` against the sequence number of the input. [https://bitcoin.stackexchange.com/a/38846](https://bitcoin.stackexchange.com/a/38846). This is enforced by the `nSequence` field that is a part of every tx input, creating a "relative locktime". This allowed an input to specify the earlierst time it can be added to a block based on how long ago the output being spent by that input was included in a block.


`OP_CheckSequenceVerify` allows locking for at most 65535 blocks (about 455 days) or for at most 65535\*512 seconds (about 388 days). 

`OP_CheckLockTimeVerify` could be used to lock up coins for several centuries.

**Q. Why do we use CLTV instead of CSV in the HTLC output of the commitment transaction?**

CSV is relative to when the tx appears in the blockchain, while CLTV is absolute. In HTLC, the timelock is added to make it temporary. In most cases it will not be broadcasted on the blockchain, so it needs to be absolute.

Relative timelocks are useful because they allow a chain of two or more interdependent transactions to be held off chain, while imposing a time constraint on one transaction that is dependent on the elapsed time from the confirmation of a previous transaction. In other words, the clock doesn’t start counting until the UTXO is recorded on the blockchain. This functionality is especially useful in bidirectional state channels and Lightning Networks.

Relative timelocks with CSV are especially useful when several (chained) transactions are created and signed, but not propagated, when they’re kept "off-chain." A child transaction cannot be used until the parent transaction has been propagated, mined, and aged by the time specified in the relative timelock.

**Q. Why do CSV and CLTV rely on nLocktime or nSequence being
set in the transaction spending the outputs when the script could
check if the requirements are being met by itself?**

`CLTV` is a script-level absolute time lock. It compares the top item of the stack to the transactions `nLocktime`. It checks that the top item of the stack is a valid time in seconds or blocks, and that the transaction itself is locked for at least that long via an appropriate `lock_time`. It does this so that it can check that the transaction can't be confirmed before a certain time.
- Comparing the lock time specified in the script to the lock time of the transaction is great because the time is checked only indirectly.
- It passes enforcement to the `nLocktime` consensus rules while still allowing scipts to specify multiple different time-locked conditions.
- It allows scriptsig validity to be checked at any time and [cached](https://bitcointechtalk.com/whats-new-in-bitcoin-core-v0-15-part-5-6a9cfa85821f).
	- cache size: 60mb which stores up to 500,000 scripts
	- Why not cache the entire tx? Tx validity is context-specific. A tx can be valid in one block and invalid in another.
- The downside is that if `OP_CLTV` is used in the script, `lock_time` must be specified in the spending transaction, and a `sequence_no` less than `0xFFFFFFFF` must be present in the input.

`CSV` is a script-level relative time lock. It compares the top item of the stack to the input's `sequence_no` field. `OP_CSV` parses stack items the same way nSequence interprets lock-times. It respects nSequence's disable flag and type flag, and reads 16-bit lock duration specifications from the last 16 bits of the stack item. 

The reason `CSV` checks `sequence_no` is the same as it is for `CLTV`.

Read [here](https://bitcoin.stackexchange.com/questions/45806/why-does-the-time-interval-for-op-csv-need-to-be-in-the-nsequence-field-when-it).

**Q. What is Median-Time-Past?**

[https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch07.asciidoc#median-time-past](https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch07.asciidoc#median-time-past)

**Q. Can both CSV and CLTV be used in the same output and are there any known use cases for it?**
	
--


**Q. In what scenarios is `OP_CLTV` used in Lightning and in what scenarios is `OP_CSV` used?**

HTLCs include a refund clause that is connected to a timelock. This is needed incase the payment fails because of an offline node for eg. It ensures atomicity so that the entire end-to-end payment either succeeds or fails gracefully. `OP_CLTV` is used for this. Read more [here](https://github.com/lnbook/lnbook/blob/develop/08_routing_htlcs.asciidoc#htlc-cooperative-and-timeout-failure).

To allow an opportunity for penalty transactions, in case of a revoked commitment transaction, all outputs that return funds to the owner of the commitment transaction must be delayed for x blocks. This is to allow the counterparty to claim penalty incase an incorrectly broadcasted revoked commitment tx. This delay is done in a second-stage HTLC transaction. This output sends funds back to the owner of this commitment transaction and thus must be timelocked using `OP_CSV`. Read more [here](https://github.com/lightning/bolts/blob/master/03-transactions.md#commitment-transaction-outputs). To know more on why it is done in a second-stage HTLC, read [this](https://bitcoin.stackexchange.com/questions/95355/why-does-every-htlc-in-a-commitment-transaction-require-its-own-signature). [Why do we need revocation in the first place and how does LN-penalty work?](https://www.derpturkey.com/revocable-transactions-with-ln-penalty/) (key blinding is interesting).
 

`OP_CSV` is used in lightning commitment transactions to enforce a delay between publishing the commitment transaction, and spending the output -- that delay is needed so that the counterparty has time to prove the commitment was revoked and claim the outputs as a penalty. Why [here](https://lists.linuxfoundation.org/pipermail/bitcoin-dev/2015-October/011423.html).


**Why can OP_CLTV and OP_CSV not touch the stack? Why are they always either followed by OP_DROP or at the end of the script? What are the pros and cons of real-time negotiation of channel parameters?**

--

----

**Q. When two parties exchange the previous commitment's secrets
(to invalidate previous state), how do you make sure that the
exchange happens atomically? (i.e., that both receive the other's
secret, or none at all)**

If we're going from Alice to Bob (A --> B), Alice sends a `commitment_signed` first. Bob sends a `revoke_and_ack` followed by a `commitment_signed`. Alice then sends a `revoke_and_ack`.

When B sends the revoke_and_ack first, it includes the `per_commitment_secret` of the previous commitment tx (which belongs to B). A is giving money to B, so the previous commitment tx is one in which B has less, A has more. The secret B shares...


----

**Q. How can we prepare and anticipate for miners fees volatility
when creating new payment channels? (What is the mechanic?)**

I believe it's by creating multiple 

anchor outputs
rbf - 
cpfp

----

**Q. Why was Segwit an important upgrade to the base layer for Lightning?**

Segwit is a soft-fork which prevent nonintentional bitcoin transaction malleability, allow optional data transmission, and to bypass certain protocol restrictions (like the block size limit). It changes the transaction format of Bitcoin.

It was also intended to mitigate a blockchain size limitation problem that reduces bitcoin transaction speed. It does this by splitting the transaction into two segments, removing the unlocking signature ("witness" data) from the original portion and appending it as a separate structure at the end.[3] The original section would continue to hold the sender and receiver data, and the new "witness" structure would contain scripts and signatures. The original data segment would be counted normally, but the "witness" segment would, in effect, be counted as a quarter of its real size. Read more [here](https://en.wikipedia.org/wiki/SegWit)

A transaction malleability bug allows a user to change the tx id of tx before it is confirmed on the blockchain. Segwit fixes the issue. LN relies on a funding transaction to start a payment channel. If the txid can be changed, the channel becomes invalid.

Read more [here](https://github.com/lnbook/lnbook/blob/develop/07_payment_channels.asciidoc#solving-malleability-segregated-witness).

----

**Q. How do you exchange previous commitment data if the parties
aren't online at the same time?**

Peer disconnected and connected? You wait for the peer to reconnect before sending the next messages?

----
