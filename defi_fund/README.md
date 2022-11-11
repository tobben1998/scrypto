# DeFi Fund

This is a scrypto blueprint that let you create or join a fund. As a fund manager you will be able to trade among different cryptocurrecnies, and collect a fee from those who want to join your fund. If you do not want o create a fund, you can also join another fund someone else has created. They will then trade on your behalf. You do not need to trust the fund manager to hold your funds. The are kept securly in a vault and can only get accesed to trade from whitlisted pools.

## Getting Started

If you haven't installed essensitals for scrypto yet look here first: https://docs.radixdlt.com/main/scrypto/getting-started/install-scrypto.html. If you haven't cloned the github repo you need to clone the repo and then move into the defi_fund folder before you can follow the instructions below.

Start by resetting the simulator.

```sh
resim reset
```

Create some new accounts.

```sh
OP1=$(resim new-account)
export PRIV_KEY1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY1=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP2=$(resim new-account)
export PRIV_KEY2=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY2=$(echo "$OP2" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS2=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP3=$(resim new-account)
export PRIV_KEY3=$(echo "$OP3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY3=$(echo "$OP3" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS3=$(echo "$OP3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP4=$(resim new-account)
export PRIV_KEY4=$(echo "$OP4" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY4=$(echo "$OP4" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS4=$(echo "$OP4" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP5=$(resim new-account)
export PRIV_KEY5=$(echo "$OP5" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY5=$(echo "$OP5" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS5=$(echo "$OP5" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
OP6=$(resim new-account)
export PRIV_KEY6=$(echo "$OP6" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export PUB_KEY6=$(echo "$OP6" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export ACC_ADDRESS6=$(echo "$OP6" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")
```

Create some tokens to test with and and send some tokens to the different accounts you created.

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

publish the package and create a fund with account2.

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
