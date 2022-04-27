# How the Layers of Lightning Fit Together


### Q. How can nodes that are not always online prevent loss of funds?

Watchtowers. 

Each commitment transaction uses a unique set of keys: `localkey` and `remotekey`. The HTLC-success and HTLC-timeout transactions use `local_delayedkey` and `revocationkey`. These are changed every time depending on the `per_commitment_point`.

The reason for key change is so that trustless watching for revoked transactions can be outsourced. Such a watcher should not be able to determine the contents of a commitment transaction — even if the watcher knows which transaction ID to watch for and can make a reasonable guess as to which HTLCs and balances may be included. Nonetheless, to avoid storage of every commitment transaction, a watcher can be given the `per_commitment_secret` values (which can be stored compactly) and the `revocation_basepoint` and `delayed_payment_basepoint` used to regenerate the scripts required for the penalty transaction; thus, a watcher need only be given (and store) the signatures for each penalty input.

Changing the `localkey` and `remotekey` every time ensures that commitment transaction ID cannot be guessed; every commitment transaction uses an ID in its output script. Splitting the `local_delayedkey`, which is required for the penalty transaction, allows it to be shared with the watcher without revealing `localkey`; even if both peers use the same watcher, nothing is revealed.

Finally, even in the case of normal unilateral close, the HTLC-success and/or HTLC-timeout transactions do not reveal anything to the watcher, as it does not know the corresponding `per_commitment_secret` and cannot relate the `local_delayedkey` or `revocationkey` with their bases.

For efficiency, keys are generated from a series of per-commitment secrets that are generated from a single seed, which allows the receiver to compactly store them.

Details [here](https://github.com/davecgh/lightning-rfc/blob/master/03-transactions.md#key-derivation).

### Q. What is the relationship of the revocation keys for multiple subsequent commitment transactions? What would change if revocation keys were independent?

For each new commitment tx there is a revocation key generated when the channel state is moved forward. This can cause an inconvenience since all the previous revocation keys may have to be remembered for previous revoked commitment transactions (i think this is the same as an RSMC (Revocable Sequence Maturity Contract)).

Instead of remembering all of the old revocation keys, they can be derived. Read more [here](https://github.com/davecgh/lightning-rfc/blob/master/03-transactions.md#efficient-per-commitment-secret-storage).

### Q. Why 49 pairs?
2^48 - 1 per-commitment secrets

### Q. What is the difference between a public revocation key and private revocation key?

Line 297 src/ln/chan_utils.rs
Q. How/why does this work?
```
	fn place_secret(idx: u64) -> u8 {
		for i in 0..48 {
			if idx & (1 << i) == (1 << i) {
				return i
			}
		}
		48
	}

```

### Q. How can you send a payment to a node without getting an invoice? What are the tradeoffs of this payment method?

Keysend

### Q. Dust limit
-> the fees will be higher to spend the output than the amount attached to it
-> the blockchain can be spammed
if outputs in the commitment tx are below the dust limit
	-> they will be omitted
	-> added to fees
while LN supports tiny payments, they cannot be enforced on chain
