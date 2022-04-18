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

Transaction-level locks affect what you can do with a transaction after it’s constructed, but Script-level locks determine what transactions can be made in the first place.

**Q. What are the different targeting and metrics for timelocks?**

- Targeting: Absolute vs relative
- Metrics: Blocks vs Seconds

**Q. What is P2SH?**

It's a type of ScriptPubKey which allows for the spending of bitcoin based on the satisfaction of the script whose hash is specified within the transaction.

**Q. Transaction locks aren’t as useful as you might think. They don’t control coins, only spends. What does this mean? TODO**

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
- It allows scriptsig validity to be checked at any time and cached.
- The The downside is that if `OP_CLTV` is used in the script, `lock_time` must be specified in the spending transaction, and a `sequence_no` less than `0xFFFFFFFF` must be present in the input.

`CSV` is a script-level relative time lock. It compares the top item of the stack to the input's `sequence_no` field. `OP_CSV` parses stack items the same way nSequence interprets lock-times. It respects nSequence's disable flag and type flag, and reads 16-bit lock duration specifications from the last 16 bits of the stack item. 

Is the reason `CSV` checks `sequence_no` the same as it was for `CLTV`?

**Q. What is Median-Time-Past?**

[https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch07.asciidoc#median-time-past](https://github.com/bitcoinbook/bitcoinbook/blob/develop/ch07.asciidoc#median-time-past)


----

