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
op5=$(resim new-token-fixed --name Bitcoin --symbol BTC 10000000)
export btc=$(echo "$op5" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
op6=$(resim new-token-fixed --name Ethereum --symbol ETH 10000000)
export eth=$(echo "$op6" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
op7=$(resim new-token-fixed --name Tether --symbol USDT 10000000)
export usdt=$(echo "$op7" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
op8=$(resim new-token-fixed --name Dogecoin --symbol DOGE 10000000)
export doge=$(echo "$op8" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

resim transfer 100000 $btc $acc2
resim transfer 100000 $btc $acc3
resim transfer 100000 $btc $acc4
resim transfer 100000 $eth $acc2
resim transfer 100000 $eth $acc3
resim transfer 100000 $eth $acc4
resim transfer 100000 $usdt $acc2
resim transfer 100000 $usdt $acc3
resim transfer 100000 $usdt $acc4
resim transfer 100000 $doge $acc2
resim transfer 100000 $doge $acc3
resim transfer 100000 $doge $acc4
```

publish the package:

```sh
pkg=$(resim publish ".")
export pkg=$(echo "$pkg" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
```

Created som pools using the radiswap component, so you can test trading.

```sh
pools=$(resim run "./transactions/instantiate_radiswap_pools_acc1.rtm")
export pool_btc_usdt=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
export pool_eth_usdt=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '2q;d')
export pool_doge_usdt=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '3q;d')
```

Create a defifund component. The main purpose of the defi component is to organize all the different funds created, and have controll of what pools that can be used for trading. It will also collect fees.

```sh
defifunds=$(resim run "./transactions/instantiate_defifunds_acc1.rtm")
export defifunds_admin_badge=$(echo "$defifunds" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
export defifunds=$(echo "$defifunds" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")
```

Add some tradingpools to the whitelist and set a fee for all deposits that goes to admin of defifunds.

```sh
resim call-method $defifunds new_pool_to_whitelist_all $pool_btc_usdt --proofs 1,$defifunds_admin_badge
resim call-method $defifunds new_pool_to_whitelist_all $pool_eth_usdt --proofs 1,$defifunds_admin_badge
resim call-method $defifunds new_pool_to_whitelist_all $pool_doge_usdt --proofs 1,$defifunds_admin_badge

resim call-method $defifunds change_deposit_fee_defifunds_all 1 --proofs 1,$defifunds_admin_badge
```

Create a new fund using account 2, and set a deposit fee that goes to the fundmanager

```sh
resim set-default-account $acc2 $pk2

fund=$(resim run "./transactions/new_fund_acc2.rtm")
export fund_manager_badge=$(echo "$fund" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
export share_token=$(echo "$fund" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3q;d')
export fund=$(echo "$fund" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")

resim call-method $fund change_deposit_fee_fund_manager 1 --proofs 1,$fund_manager_badge

```

You have not created the esstial components and, and are ready to go through a simple  example so show how it works.



## Simple Example

You have already created a fund with 100usdt on acc2. Swap 20 of them to dogecoin.

```sh
resim set-default-account $acc2 $pk2
resim call-method $fund trade_radiswap $usdt 20 $pool_doge_usdt --proofs 1,$fund_manager_badge
```

you will now have 80 usdt and 199.9 doge. you can verify by doing.

```sh
resim show $fund
```

switch to acc3 and deposit to the fund in ish the same ratio as the fund. for example 40usdt and 100doge
If your account does not hold the tokens needed you ca use the radiswap component to get the correct tokens.

```sh
resim set-default-account $acc3 $pk3
resim run transactions/deposit_usdt_and_doge_acc3.rtm
```

you hva noe deposited to the fund and recived 49 share tokens. One share tokens has bee taken as a fee. you can verify by doing.

```sh
resim show $acc3
resim show $fund
```

The fund manager can now do trades with all the funds, but he have not acces to witdraw them.
Lets do some trades with the fund manger, and chekd the fund again

```sh
resim set-default-account $acc2 $pk2
resim call-method $fund trade_radiswap $usdt 40 $pool_btc_usdt --proofs 1,$fund_manager_badge
resim call-method $fund trade_radiswap $usdt 40 $pool_eth_usdt --proofs 1,$fund_manager_badge
resim call-method $fund trade_radiswap $doge 100 $pool_doge_usdt --proofs 1,$fund_manager_badge
resim show $fund
```

acc3 hold 49 share tokens, and there exist in total 150 share tokens. When he call the witdraw function
he will get almost 1/3 of all the tokens in the pool. Test witdrawing the funds. and check the fund and the wallet

```sh
resim set-default-account $acc3 $pk3
resim call-method $fund withdraw_tokens_from_fund 49,$share_token
resim show $fund
resim show $acc3
```

fundmanager and defifund_admin can withtdraw the fee collected whenever they want. Try witdrawing them

```sh
resim set-default-account $acc1 $pk1
resim call-method $defifunds withdraw_collected_fee_defifunds_all --proofs 1,$defifunds_admin_badge
resim show $acc1

resim set-default-account $acc2 $pk2
resim call-method $fund withdraw_collected_fee_fund_manager --proofs 1,$fund_manager_badge
resim show $acc2
```

You have now hopefully a simple understading of how defifunds work. Try exploring with creating multiple funds forexample.
To get a better understaidng of how the components works you should check you the src files. Down below is an overview over functions
you can call to use the fund as you want. 


## Examples of methodcalls. You can change the paramters yourselves. 

Methodcalls for the defifunds_admin
```sh
resim call-method $defifunds new_pool_to_whitelist_all $pool_btc_usdt --proofs 1,$defifunds_admin_badge
resim call-method $defifunds withdraw_collected_fee_defifunds_all --proofs 1,$defifunds_admin_badge
resim call-method $defifunds change_deposit_fee_defifunds_all 1 --proofs 1,$defifunds_admin_badge
```


Methodcalls for the fundmanager
```sh
resim call-method $fund trade_radiswap $usdt 20 $pool_doge_usdt --proofs 1,$fund_manager_badge
resim call-method $fund change_deposit_fee_fund_manager 2 --proofs 1,$fund_manager_badge
resim call-method $fund withdraw_collected_fee_fund_manager --proofs 1,$fund_manager_badge
```


Methodcalls for everyone
```sh
resim call-method $defifunds new_fund 1000,$usdt 1000
resim call-method $defifunds get_fund_addresses
resim call-method $fund withdraw_tokens_from_fund 50,$share_token
resim run transactions/deposit_usdt_and_doge_acc3.rtm
```
This last method does not work with normal resim calls, beacuse of vec<Bucket>, so you need to change the .rtm file if you want to change paramters. When you deposit you need to deposit in about the same token ratio as the fund already has. This can be combined with the radiswap component if you don't have the other tokens in the fund. 


change account
```sh
resim set-default-account $acc1 $pk1
resim set-default-account $acc2 $pk2
resim set-default-account $acc3 $pk3
resim set-default-account $acc4 $pk4
```sh


show what the accounts contain.
```sh
resim show $defifunds
resim show $fund
resim show $acc1
resim show $acc2
resim show $acc3
resim show $acc4
```



