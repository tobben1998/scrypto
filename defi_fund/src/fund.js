import axios from "axios";
import {
  getFungibleTokens,
  getShareTokenAddressAndAmount,
  getFundAmounts,
} from "./helperFunctions.js";
import { getAllPricesUsd, getFundTvl } from "./pricesAndTvl";

export async function getFundInfo(fundAddress) {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: fundAddress,
    })
    .then((response) => {
      const data = response.data.details.state.data_json;
      const fundName = data[0];
      const imageLink = data[2];
      const websitelink = data[3];
      const depositFee = data[10];
      const fundStrategy = data[1];
      const sharetokenAddress = data[7];
      return [
        fundName,
        imageLink,
        websitelink,
        depositFee,
        fundStrategy,
        sharetokenAddress,
      ];
    });
}

//maybe not nescerayy to get the next to function from here. maybe get them from mongo db instead to not need to call getAllpricesUsd more then nesecarry?
//or maybe it does not matter that much?

//Can optimize api calls on getYourShareandTvl and getFundPortfolio.. in the first function i call get fund tvl, which calls getallprices usd.
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

export async function getFundPortfolio(fundAddress) {
  const tokenPrices = await getAllPricesUsd();
  const fundAmounts = await getFundAmounts(fundAddress);
  let totalUsdValue = 0;

  const portfolio = new Map();
  for (const [tokenAddress, amount] of fundAmounts) {
    const usdValue = tokenPrices[tokenAddress] * amount;
    totalUsdValue += usdValue;
    portfolio.set(tokenAddress, usdValue);
  }

  for (const [tokenAddress, usdValue] of portfolio) {
    const percentage = (usdValue / totalUsdValue) * 100;
    portfolio.set(tokenAddress, percentage);
  }

  return portfolio;
}
