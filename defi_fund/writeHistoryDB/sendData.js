import {
  getFundTvl,
  getFundPrice,
  getFunds,
  updateAll,
  getFundName,
} from "./apiData.js";

import { MongoClient } from "mongodb";
import { config } from "dotenv";
config();

const username = process.env.USERNAME;
const password = process.env.PASSWORD;
const uri = `mongodb+srv://${username}:${password}@cluster0.ss8pcty.mongodb.net/?retryWrites=true&w=majority`;
const client = new MongoClient(uri);

function createNewCollection(databaseName, collectioName) {
  const db = client.db(databaseName);
  db.createCollection(collectioName, {
    timeseries: {
      timeField: "date",
      metaField: "meta",
      granularity: "minutes",
    },
  });
  console.log("New collection made");
}
//createNewCollection("DefiFunds", "FundPricesAndTvls");

async function insertData() {
  try {
    await client.connect();
    await updateAll();
    const funds = getFunds();
    const database = client.db("DefiFunds");
    const collection = database.collection("FundPricesAndTvls");

    // Insert data for each fund
    for (const fund of funds) {
      const fundAddr = fund[0];
      const name = getFundName(fundAddr);
      const shareTokenAddr = fund[2];
      const price = getFundPrice(fundAddr);
      const tvl = getFundTvl(fundAddr);
      const meta = {
        name: name,
        shareTokenAddress: shareTokenAddr,
        fundAddress: fundAddr,
      };
      await collection.insertOne({
        date: new Date(),
        meta: meta,
        price: price,
        tvl: tvl,
      });
      console.log(`Inserted price and tvls for "${name}"`);
    }
  } catch (err) {
    console.log(err.stack);
  }
}

//insertData();
//setInterval(insertData, 10 * 10 * 1000); // Insert data every 10th minute
