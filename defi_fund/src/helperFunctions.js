import {
  RadixDappToolkit,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
  ResourceAddress,
} from "@radixdlt/radix-dapp-toolkit";
import axios from "axios";
import pkg from "numeric";
const { Solve } = pkg;
import * as math from "mathjs";

export const addr = {
  XRD: "resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp",
  USDP: "USDP",
  WETH: "WETH",
  WBTC: "WBTC",
};

//(resource addr, number)
export let price = {};

//will not be updated, just for testing
export let priceTest = {
  WBTC: 50000,
  WETH: 2000,
  USDP: 1,
  resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp: 1,
};

//Simple function that does not care about slippage
//need to make a maximation function that uses liquidity in the pool to calculate what the ratio shoudl be.
export async function getRatios(FundAddress) {
  let amounts = await getFundAmounts(FundAddress);
  let totalValue = 0;
  let values = new Map();
  //update prices before here, so you get the most uptodate ones.
  for (let [address, amount] of amounts.entries()) {
    let value = amount * priceTest[address];
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

//call this regularly
export async function updatePrice(tokenX) {
  const data = await requestPoolInfo(tokenX, addr.XRD);
  price[tokenX] = calculatePrice(data);
}

export function calculatePrice(poolInfo) {
  const price = poolInfo[0] / poolInfo[1];
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

//Kan requeste fundAmounts fra tidligere state også. basert på feks timestamp.
//ide:
//Ha en server som hele tiden kjører og kaller price på de forskjellige tokensene.
//legge til og multiplisere for å finne pris på de forskjellige fondene. Dytter dette inn in en database.
//om serverern går ned så kan du requeste tidligere state og tidligere price data.
//så når programmet starter leser du først en gnag, og sjekker om sist gang var for feks mindre enn 5 minutter siden.
