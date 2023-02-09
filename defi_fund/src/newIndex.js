import {
  configure,
  requestBuilder,
  requestItem,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
} from "@radixdlt/connect-button";

// Configure the connect button
const connectBtn = configure({
  dAppId: "Gumball",
  networkId: 0x0b,
  onConnect: ({ setState, getWalletData }) => {
    getWalletData(
      requestBuilder(requestItem.oneTimeAccounts.withoutProofOfOwnership(1))
    ).map(({ oneTimeAccounts }) => {
      setState({ connected: true, loading: false });
      document.getElementById("accountAddress").innerText =
        oneTimeAccounts[0].address;
      accountAddress = oneTimeAccounts[0].address;
    });
  },
  onDisconnect: ({ setState }) => {
    setState({ connected: false });
  },
});
console.log("connectBtn: ", connectBtn);

// There are four classes exported in the Gateway-SDK These serve as a thin wrapper around the gateway API
// API docs are available @ https://betanet-gateway.redoc.ly/
import {
  TransactionApi,
  StateApi,
  StatusApi,
  StreamApi,
} from "@radixdlt/babylon-gateway-api-sdk";

// Instantiate Gateway SDK
const transactionApi = new TransactionApi();
const stateApi = new StateApi();
const statusApi = new StatusApi();
const streamApi = new StreamApi();

// Global states
let accountAddress; //User account address
let DefiFundsComponentAddress; // component address for DefiFunds

//functions to simplyfy code:

async function sendManifest(manifest) {
  // Send manifest to extension for signing
  const result = await connectBtn.sendTransaction({
    transactionManifest: manifest,
    version: 1,
  });
  if (result.isErr()) throw result.error;
  console.log("Result: ", result.value);

  // Fetch the transaction status from the Gateway API
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash,
    },
  });
  console.log(" TransactionApi transaction/status:", status);

  // fetch component address from gateway api and set componentAddress variable
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      transaction_identifier: {
        type: "intent_hash",
        value_hex: result.value.transactionIntentHash,
      },
    },
  });
  console.log("Committed Details Receipt", commitReceipt);

  return { status, commitReceipt };
}

// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById("instantiateDefiFunds").onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let dexComponentAddress = document.getElementById(
    "dexComponentAddress"
  ).value;

  let manifest = new ManifestBuilder()
    .callFunction(
      packageAddress,
      "GumballMachine",
      "instantiate_gumball_machine",
      [Decimal("10"), `"${dexComponentAddress}"`]
    )
    .build()
    .toString();
  console.log("Instantiate Manifest: ", manifest);
  // Send manifest to extension for signing

  const { commitReceipt: crInstantiateDefiFunds } = sendManifest(manifest);

  // set componentAddress variable with gateway api commitReciept payload
  // componentAddress = commitReceipt.details.receipt.state_updates.new_global_entities[0].global_address <- long way -- shorter way below ->
  DefiFundsComponentAddress =
    crInstantiateDefiFunds.details.referenced_global_entities[0];
  document.getElementById("DefiFundsComponentAddress").innerText =
    DefiFundsComponentAddress;
};

// ************ Buy gumball *************
document.getElementById("buyGumball").onclick = async function () {
  let manifest = new ManifestBuilder()
    .withdrawFromAccountByAmount(
      accountAddress,
      10,
      "resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x"
    )
    .takeFromWorktopByAmount(
      10,
      "resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x",
      "xrd_bucket"
    )
    .callMethod(DefiFundsComponentAddress, "buy_gumball", [
      Bucket("xrd_bucket"),
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();

  console.log("buy_gumball manifest: ", manifest);

  // Send manifest to extension for signing
  const result = await connectBtn.sendTransaction({
    transactionManifest: manifest,
    version: 1,
  });

  if (result.isErr()) throw result.error;

  console.log("Buy Gumball getMethods Result: ", result);

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash,
    },
  });
  console.log("Buy Gumball TransactionAPI transaction/status: ", status);

  // fetch commit reciept from gateway api
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      transaction_identifier: {
        type: "intent_hash",
        value_hex: result.value.transactionIntentHash,
      },
    },
  });
  console.log("Buy Gumball Committed Details Receipt", commitReceipt);

  // Show the receipt on the DOM
  document.getElementById("receipt").innerText = JSON.stringify(
    commitReceipt.details.receipt,
    null,
    2
  );
};
