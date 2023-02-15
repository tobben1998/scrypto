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

// Configure the connect button
export let accountAddress;
export const connectBtn = configure({
  dAppId: "DefiFunds",
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

// ************ Send Manifest*************
export async function sendManifest(manifest) {
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

// ************ Show Recript*************
export function showReceipt(commitReceipt, fieldId) {
  document.getElementById(fieldId).innerText = JSON.stringify(
    commitReceipt.details.receipt,
    null,
    2
  );
}
