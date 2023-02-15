import {
  configure,
  requestBuilder,
  requestItem,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
  ResourceAddress,
} from "@radixdlt/connect-button";
import axios from "axios";

const Addr = {
  XRD: "XRD",
  USDP: "BTC",
  WETH: "ETH",
  WBTC: "WBTC",
};

// ******************** Price stuff************
// cache all price datas somewhere?
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

// ************ Request pool info *************
export async function calculatePrice(poolInfo) {
  const price = poolInfo[0] / poolInfo[1];
  return price;
}

// ************ get fund amounts *************
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
