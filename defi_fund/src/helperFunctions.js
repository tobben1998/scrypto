import {
  RadixDappToolkit,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
  ResourceAddress,
} from "@radixdlt/radix-dapp-toolkit";
import axios from "axios";

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
  let data = await getFundAmounts(FundAddress);
  //(the more accurate the prices the less dust will occur, and the more share tokens you will get)
  //loop through data and cacluate value=Fundamount*price
  //values=(resource address, value)
  //totalvalue=value+value+value...
  //loop thourgh a caluclate value/total value and put into vec with (resourceaddress, ratio)
}

// ************ Request pool info *************
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

//this is not looping over the vault, but over all resource in the component. Need to wait for RCnet before
//per vault balance will be callable by api. Just set the value of the other resources to 0,
//when using this for getRatios.
export async function getFundAmounts(FundAddress) {
  axios
    .post("https://betanet.radixdlt.com/entity/resources", {
      address: FundAddress,
    })
    .then((response) => {
      let vector = response.data.fungible_resources.items;
      let map = new Map();
      for (let e of vector) {
        map.set(e.address, e.amount[0]);
      }
      return map;
    });
}
