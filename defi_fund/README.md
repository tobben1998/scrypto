# DefiFunds on Betanet V\2

## Pre-requisites

1. Node >= 12.17.0
2. The Betanet wallet & Radix-connector browser extenstion installed. Instructions [here](https://docs-babylon.radixdlt.com/main/getting-started-developers/wallet-and-connector.html)

## Interacting with DeFiFunds

1. In a terminal go to the to the root of this project (defi_fund)
2. Install the npm dependencies: `npm install`
3. Start the local server with `npm start`
4. Open up your browser at the provided url if it doesn't open automatically.
5. Make sure you created an account on the wallet and added funds via the faucet by clicking on account name and then the three dots a button to get XRD from faucet should open.
6. Click on the connect button to fetch your wallet address. Confirm login with your phone. You should see your address appearing.
7. You can now start testing the different functions and methods on the blueprint

### New Fund

If you want to test being a fundmanager you can create a new fund using the new Fund method. Fill in the fields, and press the Create Fund button.

### Choose what fund you interact on

Scroll down to "Get Funds in DefiFunds" press the get button to see all the created funds.
Select the fund you want to interact on by setting Fund, fundManger and Sharetoken.

### Trade Beakerfi

This method is only for the fundmanger. When you use this method you will trade with the tokens in the fund.
Select the amount you want to trade. The token you want to trade from, and the pool you want to trade with. You trade using the pools from https://beaker.fi/

### Deposit tokens to fund

If you want to buy part of a fund you can call this method. The method will buy up tokens equal to the ratio in the fund using the beaker.fi and depsoit those tokens into the fund.
You will recive sharetokens in retunrn

### Whitdraw tokens from fund

When you want to sell you share of the fund you can use this method. By calling this method you will take out your share of all the different tokens, and then use beaker.fi to swap those tokens into the specific token you want.
