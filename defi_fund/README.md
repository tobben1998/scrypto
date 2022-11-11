# DeFi Fund

This is a proof of conept scrypto blueprint that lets you create or join a fund. As a fund manager you will be able to trade cryptocurrecnies, and collect a fee from those who want to join your fund. If you do not want to create a fund, you can also join a fund that someone else has created. They will then trade on your behalf. You do not need to trust the fund manager to hold your funds. They are kept securly in a vault and can only be traded with whitelisted tokens by the fund manager. He has never accecs to withdraw your funds.

## Getting Started

If you haven't installed essensitals for scrypto yet look here first: https://docs.radixdlt.com/main/scrypto/getting-started/install-scrypto.html. If you haven't cloned the github repo you need to clone the repo and then move into the defi_fund folder before you can follow the instructions below.

Start by resetting the simulator:

```sh
resim reset
```

Create some new accounts:

```sh
op1=$(resim new-account)
export pk1=$(echo "$op1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acc1=$(echo "$op1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

op2=$(resim new-account)
export pk2=$(echo "$op2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acc2=$(echo "$op2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

op3=$(resim new-account)
export pk3=$(echo "$op3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acc3=$(echo "$op3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

op4=$(resim new-account)
export pk4=$(echo "$op4" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acc4=$(echo "$op4" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
```

Create some tokens to test with and and send some tokens to the different accounts you created:

```sh
resim set-default-account $acc1 $pk1

export xrd=resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag

op5=$(resim new-token-fixed --name Bitcoin --symbol BTC 1000000)
export btc=$(echo "$op5" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

op6=$(resim new-token-fixed --name Ethereum --symbol ETH 1000000)
export eth=$(echo "$op6" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

op7=$(resim new-token-fixed --name Tether --symbol USDT 1000000)
export usdt=$(echo "$op7" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

op8=$(resim new-token-fixed --name Dogecoin --symbol DOGE 1000000)
export doge=$(echo "$op8" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")


resim transfer 1000 $btc $acc2
resim transfer 1000 $btc $acc3
resim transfer 1000 $btc $acc4
resim transfer 1000 $eth $acc2
resim transfer 1000 $eth $acc3
resim transfer 1000 $eth $acc4
resim transfer 1000 $usdt $acc2
resim transfer 1000 $usdt $acc3
resim transfer 1000 $usdt $acc4
resim transfer 1000 $doge $acc2
resim transfer 1000 $doge $acc3
resim transfer 1000 $doge $acc4
```

publish the package and create a fund with account2:

```sh
resim set-default-account $acc2 $pk2

pkg=$(resim publish ".")
export pkg=$(echo "$pkg" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

fund=$(resim run "./transactions/instantiate_fund.rtm")
export fund=$(echo "$fund" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")
```

You have now succsefully created and fund, and have the enviroments variables you need for testing some examples. The fund manager is acc2.

## Examples

Example
