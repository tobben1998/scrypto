import { MongoClient } from "mongodb";
import { config } from "dotenv";
config();

//readOnlyUser. Feel free to use it, but please do not spam
const username = "DefiFundsRead";
const password = "kTJhuQ0bc8aUe2D5";

const uri = `mongodb+srv://${username}:${password}@cluster0.ss8pcty.mongodb.net/`;
const client = new MongoClient(uri);

export async function getGraphData(shareTokenAddress, startDate, endDate) {
  try {
    await client.connect();
    const database = client.db("DefiFunds");
    const collection = database.collection("FundPricesAndTvls");

    const query = {
      "meta.shareTokenAddress": shareTokenAddress,
      date: {
        $gte: new Date(startDate),
        $lte: new Date(endDate),
      },
    };

    const cursor = collection.find(query).sort({ date: 1 });

    const graphData = {
      priceHistory: [],
      tvlHistory: [],
    };

    await cursor.forEach((doc) => {
      const date = doc.date;
      const price = doc.price;
      const tvl = doc.tvl;

      graphData.priceHistory.push({ date, price });
      graphData.tvlHistory.push({ date, tvl });
    });

    console.log("Graph data retrieved successfully");
    return graphData;
  } catch (err) {
    console.log(err.stack);
  }
}

async function getSinglePriceData(
  shareTokenAddress,
  startDate,
  endDate,
  collection
) {
  const query = {
    "meta.shareTokenAddress": shareTokenAddress,
    date: {
      $gte: new Date(startDate),
      $lte: new Date(endDate),
    },
  };

  const [firstDoc, lastDoc] = await Promise.all([
    collection.findOne(query, { sort: { date: 1 } }),
    collection.findOne(query, { sort: { date: -1 } }),
  ]);

  if (!firstDoc || !lastDoc) {
    console.log(`No data found for shareTokenAddress: ${shareTokenAddress}`);
    return { priceChange: null, priceHistory: null };
  }

  const fourhours = 4 * 60 * 60 * 1000; // 4 hours in milliseconds
  const timeDiffStart = Math.abs(firstDoc.date - new Date(startDate));
  const timeDiffEnd = Math.abs(lastDoc.date - new Date(endDate));
  //returns null if provided dates are more than for hours away from dates in database.
  const priceChange =
    timeDiffStart <= fourhours && timeDiffEnd <= fourhours
      ? (((lastDoc.price - firstDoc.price) / firstDoc.price) * 100).toFixed(2)
      : null;

  const cursor = collection.find(query).sort({ date: 1 });

  const priceHistory = [];
  await cursor.forEach((doc) => {
    const date = doc.date;
    const price = doc.price;
    priceHistory.push({ date, price });
  });

  return { priceChange, priceHistory };
}

export async function getMultiplePriceData(
  shareTokenAddresses,
  startDate,
  endDate
) {
  try {
    await client.connect();
    const database = client.db("DefiFunds");
    const collection = database.collection("FundPricesAndTvls");

    const results = await Promise.all(
      shareTokenAddresses.map(async (shareTokenAddress) => {
        const data = await getSinglePriceData(
          shareTokenAddress,
          startDate,
          endDate,
          collection
        );
        return { [shareTokenAddress]: data };
      })
    );

    const mergedResults = Object.assign({}, ...results);

    console.log(
      "Price data for multiple shareTokenAddresses retrieved successfully"
    );
    return mergedResults;
  } catch (err) {
    console.log(err.stack);
  }
}

const shareTokenAddress =
  "resource_tdx_b_1qprygcdvf0ad6ug63errw7c5t3ul6xapet773z0txqkqm203e8";
const shareTokenAddress1 =
  "resource_tdx_b_1qrwt6uf8h0pvvnmm0fg2859ez2m9wq5gk3tjyc64perqhxsz6k";
const shareTokenAddress2 =
  "resource_tdx_b_1qqnldn0qvffwfye5r84ucjm0v7plcdtnczdm9ljatpyshyr0et";
const addresses = [shareTokenAddress, shareTokenAddress1, shareTokenAddress2];
const startDate = "2023-03-17T14:00";
const endDate = new Date();
const graphData = await getGraphData(shareTokenAddress, startDate, endDate);
const result = await getMultiplePriceData(addresses, startDate, endDate);
const priceChange = result[shareTokenAddress1].priceChange;
const priceHistory = result[shareTokenAddress1].priceHistory;

console.log(priceChange, priceHistory);
console.log(priceHistory[priceHistory.length - 1]);
