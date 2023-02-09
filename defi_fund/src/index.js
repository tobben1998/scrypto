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
const connectBtn = configure({
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

// Global states
let accountAddress; //User account address
let DefiFundsComponentAddress =
  "component_tdx_b_1qg8dfmej4trz4tw9h5u67phkvguxhkn50rgyttqjq7yschtwtp";
let DefiFundsAdminBadge =
  "resource_tdx_b_1qzsp6pdfvljg4sfjluhn75z370qfap6snc8gx42k8nwqk68jz2";
const xrdAddress =
  "resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x";

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

function showReceipt(commitReceipt, fieldId) {
  document.getElementById(fieldId).innerText = JSON.stringify(
    commitReceipt.details.receipt,
    null,
    2
  );
}

// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById("instantiateDefiFunds").onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let dexComponentAddress = document.getElementById(
    "dexComponentAddress"
  ).value;

  let manifest = new ManifestBuilder()
    .callFunction(packageAddress, "Defifunds", "instantiate_defifunds", [
      `ComponentAddress("${dexComponentAddress}")`,
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();
  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  // set componentAddress variable with gateway api commitReciept payload
  // componentAddress = commitReceipt.details.receipt.state_updates.new_global_entities[0].global_address <- long way -- shorter way below ->
  DefiFundsComponentAddress =
    commitReceipt.details.referenced_global_entities[0];
  DefiFundsAdminBadge = commitReceipt.details.referenced_global_entities[1];
  document.getElementById("DefiFundsComponentAddress").innerText =
    DefiFundsComponentAddress;
  document.getElementById("DefiFundsAdminBadge").innerText =
    DefiFundsAdminBadge;
};

// ************ New Fund *************
document.getElementById("btnNewFund").onclick = async function () {
  let fundName = document.getElementById("inpNewFundName").value;
  let initialSupply = document.getElementById("inpNewFundInitialSupply").value;
  let description = document.getElementById("inpNewFundDescription").value;
  let imagelink = document.getElementById("inpNewFundImageLink").value;
  let websitelink = document.getElementById("inpNewFundWebsiteLink").value;
  let manifest = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountAddress, initialSupply, xrdAddress)
    .takeFromWorktopByAmount(initialSupply, xrdAddress, "xrd_bucket")
    .callMethod(DefiFundsComponentAddress, "new_fund", [
      `"${fundName}"`,
      Bucket("xrd_bucket"),
      Decimal(initialSupply),
      `"${description}"`,
      `"${imagelink}"`,
      `"${websitelink}"`,
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusNewFund").innerText =
    commitReceipt.details.receipt.status;
  document.getElementById("FundComponentAddress").innerText =
    commitReceipt.details.referenced_global_entities[1];
  document.getElementById("FundManagerBadge").innerText =
    commitReceipt.details.referenced_global_entities[2];
  document.getElementById("ShareTokenAddress").innerText =
    commitReceipt.details.referenced_global_entities[4];
  //showReceipt(commitReceipt, "rcptNewFund");
};

// ************ New Pool To Whitelist *************
document.getElementById("btnNewPoolToWhitelist").onclick = async function () {
  let pool = document.getElementById("inpNewPoolAddress").value;
  let manifest = new ManifestBuilder()
    .createProofFromAccountByAmount(accountAddress, 1, DefiFundsAdminBadge)
    .callMethod(DefiFundsComponentAddress, "new_pool_to_whitelist", [
      `ComponentAddress("${pool}")`,
    ])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusNewPoolToWhitelist").innerText =
    commitReceipt.details.receipt.status;
};

// ************ Remove Pool From Whitelist *************
document.getElementById("btnRemovePoolFromWhitelist").onclick =
  async function () {
    let pool = document.getElementById("inpRemovePoolAddress").value;
    let manifest = new ManifestBuilder()
      .createProofFromAccountByAmount(accountAddress, 1, DefiFundsAdminBadge)
      .callMethod(DefiFundsComponentAddress, "remove_pool_from_whitelist", [
        `ComponentAddress("${pool}")`,
      ])
      .build()
      .toString();

    console.log("Manifest: ", manifest);

    const { commitReceipt } = await sendManifest(manifest);

    document.getElementById("StatusRemovePoolFromWhitelist").innerText =
      commitReceipt.details.receipt.status;
  };

// ************ Change deposit fee defifunds *************
document.getElementById("btnChangeDepositFeeDefifunds").onclick =
  async function () {
    let new_fee = document.getElementById("inpChangeDepositFeeDefifunds").value;
    let manifest = new ManifestBuilder()
      .createProofFromAccountByAmount(accountAddress, 1, DefiFundsAdminBadge)
      .callMethod(DefiFundsComponentAddress, "change_deposit_fee_defifunds", [
        Decimal(new_fee),
      ])
      .build()
      .toString();

    console.log("Manifest: ", manifest);

    const { commitReceipt } = await sendManifest(manifest);

    document.getElementById("StatusChangeDepositFeeDefifunds").innerText =
      commitReceipt.details.receipt.status;
  };

// ************ Withdraw collected fee defifunds *************
document.getElementById("btnWithdrawCollectedFeeDefifunds").onclick =
  async function () {
    let address = document.getElementById(
      "inpWithdrawCollectedFeeDefifunds"
    ).value;
    let manifest = new ManifestBuilder()
      .createProofFromAccountByAmount(accountAddress, 1, DefiFundsAdminBadge)
      .callMethod(
        DefiFundsComponentAddress,
        "withdraw_collected_fee_defifunds",
        [ResourceAddress(address)]
      )
      .callMethod(accountAddress, "deposit_batch", [
        Expression("ENTIRE_WORKTOP"),
      ])
      .build()
      .toString();

    console.log("Manifest: ", manifest);

    const { commitReceipt } = await sendManifest(manifest);

    document.getElementById("StatusWithdrawCollectedFeeDefifunds").innerText =
      commitReceipt.details.receipt.status;
  };

// ************ Withdraw collected fee defifunds all *************
document.getElementById("btnWithdrawCollectedFeeDefifundsAll").onclick =
  async function () {
    let manifest = new ManifestBuilder()
      .createProofFromAccountByAmount(accountAddress, 1, DefiFundsAdminBadge)
      .callMethod(
        DefiFundsComponentAddress,
        "withdraw_collected_fee_defifunds_all",
        []
      )
      .callMethod(accountAddress, "deposit_batch", [
        Expression("ENTIRE_WORKTOP"),
      ])
      .build()
      .toString();

    console.log("Manifest: ", manifest);

    const { commitReceipt } = await sendManifest(manifest);

    document.getElementById(
      "StatusWithdrawCollectedFeeDefifundsAll"
    ).innerText = commitReceipt.details.receipt.status;
  };
