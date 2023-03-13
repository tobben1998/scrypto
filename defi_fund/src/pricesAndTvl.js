import axios from "axios";
import { DefiFundsComponentAddress } from "./index.js";

export const addr = {
  XRD: "resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp",
  BUSD: "resource_tdx_b_1qpev6f8v2su68ak5p2fswd6gqml3u7q0lkrtfx99c4ts3zxlah",
  WETH: "resource_tdx_b_1qps68awewmwmz0az7cxd86l7xhq6v3pez355wq8gra3qw2v7kp",
  WBTC: "resource_tdx_b_1qre9sv98scqut4k9g3j6kxuvscczv0lzumefwgwhuf6qdu4c3r",
};

// ************ API Calls*************
export async function requestPoolInfo(tokenX, tokenY) {
  const apiUrl = "https://beaker.fi:8888/pool_info_price?";
  const params = `token_x=${tokenX}&token_y=${tokenY}`;

  try {
    const response = await axios.get(apiUrl + params);
    const poolInfo = response.data;
    return poolInfo;
  } catch (error) {
    console.error(error);
  }
}

export async function getRadixPrice() {
  return axios
    .get(
      "https://api.coingecko.com/api/v3/simple/price?ids=radix&vs_currencies=usd"
    )
    .then((response) => {
      const price = response.data.radix.usd;
      return price;
    })
    .catch((error) => {
      console.error(error);
    });
}

//will likly do this another way when rc net is coming. can chenck resource amount in vault directly
export async function getFundAmounts(FundAddress) {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: FundAddress,
    })
    .then((response) => {
      let vector = response.data.details.state.data_json[4];
      let map = new Map();
      for (let e of vector) {
        map.set(e[0], e[1][1]);
      }
      return map;
    })
    .catch((error) => {
      console.error(error);
    });
}

export async function getAllFunds() {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: DefiFundsComponentAddress,
    })
    .then((response) => {
      let vector = response.data.details.state.data_json[0];
      return vector.map((arr) => arr[0]);
    });
}

//will likly do this another way when rc net is coming. amount directly then
export async function getShareTokenAmount(FundComponentAddress) {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: FundComponentAddress,
    })
    .then((response) => {
      return response.data.details.state.data_json[8];
    })
    .catch((error) => {
      console.error(error);
    });
}

// ************ functions for calcualting TVL and prices*************
export function calculatePrice(poolInfo) {
  const price = poolInfo[1] / poolInfo[0];
  return price;
}

export async function getFundTvl(FundAddress) {
  const [tokenPrices, fundAmounts] = await Promise.all([
    getAllPricesUsd(),
    getFundAmounts(FundAddress),
  ]);

  let totalValue = 0;

  for (let [tokenAddress, amount] of fundAmounts) {
    if (tokenPrices[tokenAddress]) {
      const price = tokenPrices[tokenAddress];
      const value = price * amount;
      totalValue += value;
    }
  }

  return totalValue;
}

export async function getFundPrice(FundAddress) {
  const [tvl, amount] = await Promise.all([
    getFundTvl(FundAddress),
    getShareTokenAmount(FundAddress),
  ]);
  return tvl / amount;
}

export async function getAllPricesXrd() {
  //fetch prices in paralell
  const promises = Object.entries(addr)
    .filter(([tokenSymbol, tokenAddress]) => tokenSymbol !== "XRD")
    .map(([tokenSymbol, tokenAddress]) =>
      requestPoolInfo(tokenAddress, addr.XRD).then((data) => ({
        tokenAddress,
        price: calculatePrice(data),
      }))
    );

  const results = await Promise.all(promises);

  const prices = {};
  for (const { tokenAddress, price } of results) {
    prices[tokenAddress] = price;
  }
  prices[addr.XRD] = 1;

  return prices;
}

export async function getAllPricesUsd() {
  const [xrdPrice, prices] = await Promise.all([
    getRadixPrice(),
    getAllPricesXrd(),
  ]);

  const usdPrices = {};

  for (const [tokenAddress, price] of Object.entries(prices)) {
    usdPrices[tokenAddress] = price * xrdPrice;
  }

  return usdPrices;
}

//get fund amounts of all the fund addresses that is passed in. input paramter is a vec<componentaddresses>
export async function getAllFundAmounts(funds) {
  const amounts = await Promise.all(
    funds.map(async (fund) => {
      const fundAmounts = await getFundAmounts(fund);
      return [fund, fundAmounts];
    })
  );

  return new Map(amounts);
}

//get shartoken amount of all fund addresses that is passed in. input paramter is a vec<componentaddresses>
export async function getAllShareTokenAmounts(funds) {
  const amounts = await Promise.all(
    funds.map(async (fund) => {
      const shareTokenAmount = await getShareTokenAmount(fund);
      return [fund, shareTokenAmount];
    })
  );

  return new Map(amounts);
}

export async function getAllFundPricesAndTvl() {
  const funds = await getAllFunds();
  const [fundAmounts, shareTokenAmounts, tokenPrices] = await Promise.all([
    getAllFundAmounts(funds),
    getAllShareTokenAmounts(funds),
    getAllPricesUsd(),
  ]);

  const fundPrices = new Map();
  const fundTvl = new Map();

  for (const fund of funds) {
    const amounts = fundAmounts.get(fund);
    let totalValue = 0;

    for (let [tokenAddress, amount] of amounts.entries()) {
      if (tokenPrices[tokenAddress]) {
        const price = tokenPrices[tokenAddress];
        const value = price * amount;
        totalValue += value;
      }
    }

    const shareTokenAmount = shareTokenAmounts.get(fund);
    const price = shareTokenAmount ? totalValue / shareTokenAmount : 0;
    fundPrices.set(fund, price);
    fundTvl.set(fund, totalValue);
  }

  const result = [];

  for (const fund of funds) {
    const price = fundPrices.get(fund);
    const tvl = fundTvl.get(fund);
    result.push([fund, price, tvl]);
  }

  return result;
}
