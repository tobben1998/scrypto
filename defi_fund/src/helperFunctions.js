import {
  RadixDappToolkit,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
  ResourceAddress,
} from "@radixdlt/radix-dapp-toolkit";
import axios from "axios";
import { DefiFundsComponentAddress } from "./index.js";

export const addr = {
  XRD: "resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp",
  BUSD: "resource_tdx_b_1qpev6f8v2su68ak5p2fswd6gqml3u7q0lkrtfx99c4ts3zxlah",
  WETH: "resource_tdx_b_1qps68awewmwmz0az7cxd86l7xhq6v3pez355wq8gra3qw2v7kp",
  WBTC: "resource_tdx_b_1qre9sv98scqut4k9g3j6kxuvscczv0lzumefwgwhuf6qdu4c3r",
};

//(resource addr, number)
export let price = {};

//Simple function that does not care about slippage
//need to make a maximation function that uses liquidity in the pool to calculate what the ratio shoudl be.
export async function getRatios(FundAddress) {
  let amounts = await getFundAmounts(FundAddress);
  //console.log(amounts);
  let totalValue = 0;
  let values = new Map();
  await updateAllPrices();
  //console.log(getAllPrices());
  for (let [address, amount] of amounts.entries()) {
    let value = amount * price[address];
    values.set(address, value);
    totalValue += value;
  }
  let ratios = new Map();
  for (let [address, value] of values.entries()) {
    let ratio = value / totalValue;
    ratios.set(address, ratio);
  }
  return ratios;
}

//må finne en constrained optimalization algorithm som fungerer.
//denne fungerer ikke er bare inpirasjon fra chatgpt
export function getOptimizedRatios(amount, x, y, addresses, fundaddress) {
  const fm = 1; //fee multiplier 1=no fee, 0=100% fee
  const n = x.length;

  //calculate the fund ratio
  const reserveAmounts = getFundAmounts(fundaddress);
  const prices = x.map((_, i) => x[i] / y[i]);
  const values = prices.map((price, i) => price * reserveAmounts[addresses[i]]);
  const totalValue = values.reduce((total, value) => total + value, 0);
  const p = values.map((value) => value / totalValue); //perfect fund ratio. The ratio without slippage.
  // Define the objective function
  //r=ratio I will input when slippage is considered
  //p=perfect fund ratio
  //(dy1*price1/totalAmount)-p1)
  //dy=(r*dx*y)/(x-r*dx)
  //price1=y1*dx1/x1
  //r=totalAmount*y1*y1*dx1*dx1/((x1+dx1)*x1)
  //minimize sqrt(abs(r1-p1)+..+sqrt(abs(rn-pn))
  const f = (v) => {
    const dx = v.slice(0, n);
    const sumOfSquareRoots = p.reduce(
      (a, b, i) =>
        a +
        Math.sqrt(
          Math.abs(
            (amount * fm * y[i] * y[i] * dx[i] * dx[i]) /
              ((x[i] + fm * dx[i]) * x[i]) -
              b
          )
        ),
      0
    );
    return sumOfSquareRoots;
  };

  // Define the constraint: the sum of the dx values must be equal to the input amount
  const constraint = (v) => {
    return v.slice(0, n).reduce((a, b) => a + b, 0) - amount;
  };

  // Solve the constrained optimization problem using the Newton-Raphson algorithm
  const solution = Solve(f, constraint, Array(n + 1).fill(0), {
    method: "newton",
  });

  // Extract the solution
  const dx = solution.x.slice(0, n);
  const r = dx.map((_, i) => [addresses[i], dx[i] / amount]);

  return r;
}

// ************ Request pool info for beakrfi *************
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

export async function updatePrice(tokenXAddress) {
  const data = await requestPoolInfo(tokenXAddress, addr.XRD);
  price[tokenXAddress] = calculatePrice(data);
}

//call this regularly (can optimizes it by running it in parallell)
export async function updateAllPrices() {
  for (const [tokenSymbol, tokenAddress] of Object.entries(addr)) {
    if (tokenSymbol !== "XRD") {
      const data = await requestPoolInfo(tokenAddress, addr.XRD);
      price[tokenAddress] = calculatePrice(data);
    } else {
      price[tokenAddress] = 1;
    }
  }
}

export function getAllPrices() {
  return price;
}

export function calculatePrice(poolInfo) {
  const price = poolInfo[1] / poolInfo[0];
  return price;
}

//will likly dot his another way when rc net is coming. can chenck resource amount i vault directly
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
    });
}

export async function getAllSharetokens() {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: DefiFundsComponentAddress,
    })
    .then((response) => {
      let vector = response.data.details.state.data_json[0];
      return vector.map((arr) => arr[2]);
    });
}

export async function getAllFunds() {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: DefiFundsComponentAddress,
    })
    .then((response) => {
      let vector = response.data.details.state.data_json[0];
      return vector;
    });
}

export async function getFungibleTokens(address) {
  return axios
    .post("https://betanet.radixdlt.com/entity/fungibles", {
      address: address,
    })
    .then((response) => {
      let vector = response.data.fungibles.items;
      return vector.map((item) => [
        item.address,
        parseFloat(item.amount.value),
      ]);
    });
}

export async function getShareTokenAddressAndAmount(fundAddress) {
  return axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: fundAddress,
    })
    .then((response) => {
      const data = response.data.details.state.data_json;
      const address = data[7];
      const amount = data[8];
      return [address, amount];
    });
}

export async function getSharetokensWallet(address) {
  const sharetokens = await getAllSharetokens();
  const tokens = await getFungibleTokens(address);

  const matchingTokens = tokens.filter((token) =>
    sharetokens.includes(token[0])
  );

  return matchingTokens;
}

//Kan requeste fundAmounts fra tidligere state også. basert på feks timestamp.
//ide:
//Ha en server som hele tiden kjører og kaller price på de forskjellige tokensene.
//legge til og multiplisere for å finne pris på de forskjellige fondene. Dytter dette inn in en database.
//om serverern går ned så kan du requeste tidligere state og tidligere price data.
//så når programmet starter leser du først en gnag, og sjekker om sist gang var for feks mindre enn 5 minutter siden.
