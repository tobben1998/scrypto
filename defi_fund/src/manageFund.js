import axios from "axios";
import {
  getFungibleTokens,
  getShareTokenAddressAndAmount,
  getFundAmounts,
} from "./helperFunctions.js";
import { getAllPricesUsd, getFundTvl } from "./pricesAndTvl";

//returns the funds that provided address are fundmanagers for.
export async function getFundManagerFunds(address) {
  const funds = await getAllFunds();
  const tokens = await getFungibleTokens(address);
  console.log(funds);
  console.log(tokens);

  const matchingFunds = funds.filter((fund) =>
    tokens.some((token) => fund[1] === token[0])
  );

  return matchingFunds.map((fund) => fund[0]);
}

export async function getFundNameImageBadge(fundAddress) {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: fundAddress,
    })
    .then((response) => {
      const data = response.data.details.state.data_json;
      const fundName = data[0];
      const imageLink = data[2];
      const fundManagerBadge = data[5];
      return [fundName, imageLink, fundManagerBadge];
    });
}

//Can optimize api calls on getYourShareandTvl and getMangeFundPortfolio.. in the first function i call get fund tvl, which calls getallprices usd.
//unecescarry to call getAllpriceUsdTwice. I guess also do not need to call on all, only the ones in the fund.
export async function getYourShareAndTvl(usrAddress, fundAddress) {
  const [sharetokenAddress, amount] = await getShareTokenAddressAndAmount(
    fundAddress
  );
  const [tokens, tvl] = await Promise.all([
    getFungibleTokens(usrAddress),
    getFundTvl(fundAddress),
  ]);
  const token = tokens.find((token) => token[0] === sharetokenAddress);
  const yourAmount = token ? token[1] : 0;
  const yourShare = (yourAmount * tvl) / amount;
  return [yourShare, tvl];
}

export async function getManageFundPortfolio(fundAddress) {
  const tokenPrices = await getAllPricesUsd();
  const fundAmounts = await getFundAmounts(fundAddress);
  let totalUsdValue = 0;

  const portfolio = new Map();
  for (const [tokenAddress, amount] of fundAmounts) {
    const usdValue = tokenPrices[tokenAddress] * amount;
    totalUsdValue += usdValue;
    portfolio.set(tokenAddress, {
      amount,
      usdValue,
    });
  }

  for (const [tokenAddress, data] of portfolio) {
    const percentage = (data.usdValue / totalUsdValue) * 100;
    data.percentage = percentage;
    portfolio.set(tokenAddress, data);
  }

  return portfolio;
}
