# Base58 Week 1 - Bitcoin Transactions

What we'll cover?

Week 1:
What is a bitcoin tx?
What is it made out of?
What kind of information can we find in it?
pre-segwit format

Week 2: script, locking and unlocking. first standard script
Week 3: segwit; learn about segwit transactions and segwit scripts
Week 4: signatures; cryptography, elliptic curves, private keys
Week 5: signing transactions: ECDSA etc... schnorr (taproot <- not enough time)
Week 6: multisig + getting your transaction mined!

----

txid is the hash256 of the final valid and signed rawtransaction
sha256(sha256(signedrawtransaction))

in bitcoin-cli whenever you see an op_code called "hash" that means 2 rounds of some hashing functions. e.g. hash160 is ripemd160(sha256(data))

bitcoin tx
encoding: hex
endianness: little

conversion from little to big endian
- reverse every pair of bits
  - 5c23210000 -> 000021235c

what's a samourai coinjoin?

python check out -> float flake


## Notes

[https://gist.github.com/niftynei/32b037faca10c0210516b676d4503716](https://gist.github.com/niftynei/32b037faca10c0210516b676d4503716)


----

## Example


### Segwit

txid => `68333a10b368e0d002098827fa3f348135fb728ade74d265e6abf41dfcb60a1c`

```
$ bitcoin-cli getrawtransaction

{
	"result": "02000000000103d19441b832d4e24e4e10c08413b57c017785ea7407b373d4566e11ad94d8134c1c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff3d416a5941422eeecbcc0e3fe6aa7a88d00d22b67df149293e3c5bee10c4719a2c000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff4f22589a292781a3cc2d636b9f1932f367305625a7874f8573b72b98ad73699600000000171600147c846a806f4d9e516c9fb2fe364f28eac4e3c3fcffffffff02f56468040000000017a9142c21151d54bd219dcc4c52e1cb38672dab8e36cc87c0276544000000001976a91439b1050dba04b1d1bc556c2dcdcb3874ba3dc11e88ac0247304402203ccede7995b26185574a050373cfe607f475f7d8ee6927647c496e3b45bf61a302202bd1ff88c7f4ee0b6f0c98f687dff9033f770b23985f590d178b9085df589101012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02483045022100b46ab18056655cc56b1778fd61a56f895c2f44c97f055ea0269d991efd181fb402206d651a5fb51081cfdb247a1d489b182f41e52434d7c4575bea30d2ce3d24087d012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab02473044022069bf2ac34569565a62a1e0c12750104f494a906fefd2f2a462199c0d4bc235d902200c37ef333b453966cc5e84b178ec62125cbed83e0c0df4448c0fb331efa49e51012103789a9d83798d4cbf688f9969a94084ee1655059e137b43492ee94dc4538790ab00000000\",\"error": null,"id": null
}
```

### Pre-segwit

txid => `db6e06ff6e53356cc22cd1b9b8d951ddf70dc6bb275ee76880a0b951c1c290e6`

```
$ bitcoin-cli getrawtransaction

{
    "result": "02000000016dbad361f6a9f0c60e8b032e2008aa0a9151c7bf691464274c89315d2f6c52cc19000000fc0047304402204945c3e4f824d263bb22e117a12bfff741d996d594f07551c93e0fde77910d32022016c2b69daec51bd4afdd81bf90f76667dda515773b3da91174043fc7299acb5301473044022053c71a4730160b20e565cb669a44b793f42d2912e84d528cf203089abcb2874402203311303cfc36b91372e47d5fa0b22104e7c25bb5a8dcccd15c423620d5700304014c69522102047464f518269c6cba42b859d28e872ef8f6bb47d93e24d5c11ac6eca8a2845721029b48417598a2d2dab54ddddfca8e1a9c8d4967002180961f53a7748710c2176521036b1023b6c7ed689aaf3bc8ca9ee5c55da383ae0c44fc8b0fec91d6965dae5d5e53aeffffffff0450da1100000000001600141e129251311437eea493fce2a3644a5a1af8d40710731d00000000001976a9140ac4423b045a0c8ed5f4fb992256ed293a313ae088ac946b9b000000000017a914cd38af19a803de11ddcee3a45221ed9ac49140478761ea945a0000000017a9143572de0bb360f212ef8813a9e012f63a7035c9c98700000000",
    "error": null,
    "id": null
}
```

-----

## Day 2

block subsidy => 6.25000000 bitcoin

## Version
Always 4 bytes long (either value 1, or 2)
Why? 1. it is the default size of any number in most computer systems 2. future proofing
[https://en.wikipedia.org/wiki/C_data_types](https://en.wikipedia.org/wiki/C_data_types)


## Inputs
count: number of inputs
txid: 32 bytes
vout: 4 bytes, little endian
scriptSigL proves that you can spend it
sequence: 4 bytes, little endian

## Outpus
count: number of outputs
number of bytes for an output (compact size, "varint")
amount: 8 bytes
scriptPubKey: lockup the bitcoin so only a certain person can spend it.
			  variable sized field

## locktime
4 bytes, little-endian

## Where do transaction id's come from?
It's the ID of a transaction (especially true for pre-segwit txs)

Take the data of the signed transaction and run it through a ahs function TWICE.

## Hashes

Take any length of data, give it to a hash function. Hash function will "hash" the input -> always return data of a known size.

Kinds
sha128
sha256
sha512
ripemd160

"SHA" or "RIPEMD" -> description of the process they're going to use to product the hash.
128, 256, 512, 160 -> the number of bits that the result (after hashing) will be (1 byte = 8 bits)

Why hash twice?
[https://crypto.stackexchange.com/a/50020](https://crypto.stackexchange.com/a/50020)

----
## Notes

[https://gist.github.com/niftynei/f7c1a00f950ac57e8d0651b43a1d5044](https://gist.github.com/niftynei/f7c1a00f950ac57e8d0651b43a1d5044)

