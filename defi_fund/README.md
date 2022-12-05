# DefiFunds

This is a proof of conept a dapp where traders can create decentralized
funds and everyday people can invest in them. Traders will effectivly show if they can trade
profitable or not, and do not need to go into the complex process of creating a centralized hedge
fund for getting everyday people to buy their fund. Everyday useres will have the convienice
of being able to invest in the defi ecosystem, without doing the trading themselves or go to a
centralized platform.

## Getting Started

If you haven't installed essentials for scrypto yet, look here first: https://docs.radixdlt.com/main/scrypto/getting-started/install-scrypto.html. If you haven't cloned the GitHub repo, you need to clone the repo and then move it into the defi_fund folder before you can follow the instructions below.

Start by reseting the radix egine simulator.

```sh
resim reset
```

You will then need to create some new accounts.

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

Create some new tokens, and send tokens to the different accounts you just created.
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

Publish the package containg the three blueprints.
```sh
pkg=$(resim publish ".")
export pkg=$(echo "$pkg" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
```

Create some new trading pools using the radiswap blueprint.
```sh
pools=$(resim run "./transactions/instantiate_radiswap_pools_acc1.rtm")
export pool_btc_usdt=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
export pool_eth_usdt=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '2q;d')
export pool_doge_usdt=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '3q;d')
```

Instantiate defifunds using the defifunds blueprint.
```sh
defifunds=$(resim run "./transactions/instantiate_defifunds_acc1.rtm")
export defifunds_admin_badge=$(echo "$defifunds" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
export defifunds=$(echo "$defifunds" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")
```

Finally add some radiswap pools to the whitelist in the defifunds component. Beacause of the 300
epoch delay feature explained in 4.2.2 you also need to move 300 epochs forward in time.
```sh
resim set-current-epoch 0
resim call-method $defifunds new_pool_to_whitelist $pool_btc_usdt --proofs 1,$defifunds_admin_badge
resim call-method $defifunds new_pool_to_whitelist $pool_eth_usdt --proofs 1,$defifunds_admin_badge
resim call-method $defifunds new_pool_to_whitelist $pool_doge_usdt --proofs 1,$defifunds_admin_badge
resim set-current-epoch 300
```
You have now created all the essistal components in the dapp, and are ready to start going through
some examples.



## Example - Basic features

In this example I will go through the basic features of the defifunds dapp. Creating a fund, depositing and witdrawing from a fund, trading with the tokens in a fund, and witdrawing the collected fee.

Start by creating a new fund using account 2 and change the deposit fee. Let the name be
"DegenFund", place a bucket with 100 usdt and set inital sharetokens to be 100. Account 2 will
now be the fundmanager of "DegenFund". He will recieve a badge used to get access to the methods
only available for the fundmanager and some sharetokens represting his share of the fund.
```sh
resim set-default-account $acc2 $pk2

fund=$(resim call-method $defifunds new_fund "DegenFund" 100,$usdt 100)
export fund_manager_badge=$(echo "$fund" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
export share_token=$(echo "$fund" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3q;d')
export fund=$(echo "$fund" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")

resim call-method $fund change_deposit_fee_fund_manager 1 --proofs 1,$fund_manager_badge
```

The fund you created now contains 100 usdt. Lets swap 20 of those usdt for dogecoin.
```sh
resim set-default-account $acc2 $pk2
resim call-method $fund trade_radiswap $usdt 20 $pool_doge_usdt --proofs 1,$fund_manager_badge
```

The fund now contains 80 usdt and 199.99 doge. you can verify by calling:
```sh
resim show $fund
```

Swicth to another user for example account 3 and deposit 40 usdt and 100 doge. The command
below uses a transaction manifest, and you can edit the parmeters by editing the ".rtm" file. The
method you call need tokens in about the same ratio as the fund. If you don’t have the tokens
needed you can use the radiswap component and swap to the correct amounts. Swaping and
depositing can be done in the same transaction using a transaction manifest.
```sh
resim set-default-account $acc3 $pk3
resim run transactions/deposit_usdt_and_doge_acc3.rtm
```

You have now deposited to the fund and received 49 share tokens. One share token has been taken
as a fee. You can verify by doing:
```sh
resim show $acc3
resim show $fund
resim show $defifunds
```

Let’s switch back to the fundmanager account and do some trades with the tokens in the fund.
```sh
resim set-default-account $acc2 $pk2
resim call-method $fund trade_radiswap $usdt 40 $pool_btc_usdt --proofs 1,$fund_manager_badge
resim call-method $fund trade_radiswap $usdt 40 $pool_eth_usdt --proofs 1,$fund_manager_badge
resim call-method $fund trade_radiswap $doge 100 $pool_doge_usdt --proofs 1,$fund_manager_badge
resim show $fund
```

Account 3 holds 49 share tokens, and there exists in total 150 share tokens. When he calls the
withdraw function, he will get almost 1/3 of tokens in the fund. If you want to get one token
instead of many different when you withdraw, you can use the radiswap component to swap them
into one. You can use the transaction manifest and do it all in a single transaction.
```sh
resim set-default-account $acc3 $pk3
resim call-method $fund withdraw_tokens_from_fund 49,$share_token
resim show $fund
resim show $acc3
```

Some fees have been collected from useres depositing to the fund. You can collect the fees by
calling the methods below:
```sh
resim set-default-account $acc1 $pk1
resim call-method $defifunds withdraw_collected_fee_defifunds_all --proofs 1,$defifunds_admin_badge
resim show $acc1

resim set-default-account $acc2 $pk2
resim call-method $fund withdraw_collected_fee_fund_manager --proofs 1,$fund_manager_badge
resim show $acc2
```
You have now gone through a simple example of how defifunds work. If you want to explore more
you can forexample create multiple funds. To get a better understanding of how the components
and methods work, you should check the source files.


## Examples - Misusing methods

### 1. Withdraw from the fund

The first example is just to give a clearer explenation of how componets work. The only method
that lets you withdraw tokens from the fund directly is the "witdraw tokens from fund" method.
There is no "owner" of the fund component, and all the rules for who can do what with the
component are defined in the blueprint. You can for example try to call the "withdraw tokens
from fund" method with more share tokens than you have.
```sh
resim set-default-account $acc2 $pk2
resim call-method $fund withdraw_tokens_from_fund 120,$share_token
```
The method will obvisoly fail because you don’t have enough share tokens. There is no other way
to directly take tokens from the fund to your account. The other way tokens is moved from the
fund is through the "trade radiswap" method. A potenial misuse of this method is covered in the
last example.


### 2. Withdraw collected fee

Lets test to call the "withdraw collected fee fund manager" method with another user than the
fundmanager.
```sh
resim set-default-account $acc1 $pk1
resim call-method $fund withdraw_collected_fee_fund_manager
```
If you do so you will get an AuthorizationError. It failes because of the acces rule defined in the
instantiate fund function. The only one that can call the function is the account holding the fund
manager badge.


### 3. Adding a malicious pool to the whitelist


Lets first explain how a tradingpool can be malicious and then go into two scenarios. When doing
a trade between token A and token B you are supposed to get the same dollar amount of token
B as you swap with token A minus some fees. A pool can be malicous if that is not the case. A
person can create a pool with 100 tokens he created him selves and pool them with 1 usdt. If a
trader decides to use that pool he would basically give the creater of that pool usdt and getting a
token worth nothing in return.

A scenario where this can be missused is if a fundmanager is able to compromise the defifunds
admin wallet. He will then add a maliciuos pool to the whitelist and try to trade with it.
```sh
resim set-default-account $acc1 $pk1
pools=$(resim run "./transactions/instantiate_malicious_pool_acc1.rtm")
export pool_malicious=$(echo "$pools" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '1q;d')
resim call-method $defifunds new_pool_to_whitelist $pool_malicious --proofs 1,$defifunds_admin_badge

resim set-default-account $acc2 $pk2
resim call-method $fund trade_radiswap $btc 20 $pool_malicious --proofs 1,$fund_manager_badge
```
He wont have acces to trade on that pool before the 300 epocs has occured. The creator of defifunds
can then call the "remove pool from whitelist" method and warn useres to withdraw their funds.
As long as the owner still has access to his wallet, like he most likely has, he can continue to call
the "remove pool from whitelist" method if the pool is re-added. The users will then get in practie
idenfently time to withdraw their funds.

Another scenario that could occur is that the admin of defifunds decides to add a malicious pool
him selves, beacuse he controlls a large fund and want to empty it. Some useres will likley monitor
this whitelist and warn other users to withdraw. To expect that all users would be able to withdraw
their tokens from the fund that the admin of defifunds controll within 7 days is optimstic, however
a huge portin will likely be able to do so. To further increase the safty of this unlikly event,
as incentives most likely are going against the admin to act dishonest, could be to extend the
timedealy.

To totally avoid the trust for an admin to controll a whitelist with timedelays a whitelist can
be added when the Defifunds component is created. The incovincie here is that a new Defifunds
component would need to be made if new tradingpools should be added. Another complex solution
that would minimize the chance of maliciuos pools beeing added could be to create a DAO that
controls the whitelist.
