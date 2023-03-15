import {
  getFundAmounts,
  getFundTvl,
  getFunds,
  getTokensInWallet,
  getShareTokenAddress,
  getShareTokenAmount,
  getTokenPrice,
} from "./apiDataFetcher";

//NB! call updatefunctions in apiDataFecther before you use these

//returns the funds that logged in user (if you have updatetd tokens in wallet) are fundmanagers for.
export function getFundManagerFunds() {
  const funds = getFunds();
  const tokens = getTokensInWallet();
  const matchingFunds = funds.filter((fund) => tokens.has(fund[1]));
  return matchingFunds.map((fund) => fund[0]);
}

export function getYourShareAndTvl(fundAddress) {
  const shareTokenAddress = getShareTokenAddress(fundAddress);
  const amount = getShareTokenAmount(fundAddress);
  const tokenBalances = getTokensInWallet();
  const tvl = getFundTvl(fundAddress);
  const yourAmount = tokenBalances.get(shareTokenAddress) || 0;
  const yourShare = (yourAmount * tvl) / amount;
  return [yourShare, tvl];
}

export function getManageFundPortfolio(fundAddress) {
  const fundAmounts = getFundAmounts(fundAddress);
  let totalUsdValue = 0;

  const portfolio = new Map();
  for (const [tokenAddress, amount] of fundAmounts) {
    const usdValue = getTokenPrice(tokenAddress) * amount;
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
