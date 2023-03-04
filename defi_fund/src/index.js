import {
  RadixDappToolkit,
  ManifestBuilder,
  Decimal,
  Array,
  Tuple,
  Bucket,
  Expression,
  ResourceAddress,
  ComponentAddress,
} from "@radixdlt/radix-dapp-toolkit";

import axios from "axios";
import {
  requestPoolInfo,
  calculatePrice,
  addr,
  getFundAmounts,
  getRatios,
  getAllSharetokens,
  getSharetokensWallet,
  getFungibleTokens,
} from "./helperFunctions.js";
import { accountAddress, sendManifest, showReceipt } from "./radixConnect.js";

// Global states
export let DefiFundsComponentAddress =
  "component_tdx_b_1qtscpaffn2p5d7sakqre965ns28twejlm3letuf67hqs2w83we";
let DefiFundsAdminBadge =
  "resource_tdx_b_1qrscpaffn2p5d7sakqre965ns28twejlm3letuf67hqsq04ltz";

let FundComponentAddress;
let FundManagerBadge;
let ShareTokenAddress;

document.getElementById("test").onclick = async function () {
  //let noe = await getSharetokensWallet(accountAddress);
  let noe = await getRatios(FundComponentAddress);
  console.log(noe);
};

// ************************************
// ************ DefiFunds *************
// ************************************

// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById("instantiateDefiFunds").onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let dexComponentAddress = document.getElementById(
    "dexComponentAddress"
  ).value;

  let manifest = new ManifestBuilder()
    .callFunction(packageAddress, "Defifunds", "instantiate_defifunds", [
      ComponentAddress(dexComponentAddress),
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
    .withdrawFromAccountByAmount(accountAddress, initialSupply, addr.XRD)
    .takeFromWorktopByAmount(initialSupply, addr.XRD, "xrd_bucket")
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
  document.getElementById("FundComponentAddressNewFund").innerText =
    commitReceipt.details.referenced_global_entities[1];
  document.getElementById("FundManagerBadgeNewFund").innerText =
    commitReceipt.details.referenced_global_entities[2];
  document.getElementById("ShareTokenAddressNewFund").innerText =
    commitReceipt.details.referenced_global_entities[4];
  //showReceipt(commitReceipt, "rcptNewFund");
};

// ************ New Pool To Whitelist *************
document.getElementById("btnNewPoolToWhitelist").onclick = async function () {
  let pool = document.getElementById("inpNewPoolAddress").value;
  let manifest = new ManifestBuilder()
    .createProofFromAccountByAmount(accountAddress, 1, DefiFundsAdminBadge)
    .callMethod(DefiFundsComponentAddress, "new_pool_to_whitelist", [
      ComponentAddress(pool),
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
        ComponentAddress(pool),
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

// ************************************
// ************ Fund ******************
// ************************************

// ************ Get Fund Addresses *************
document.getElementById("btnGetFundAddresses").onclick = async function () {
  axios
    .post("https://betanet.radixdlt.com/entity/details", {
      address: DefiFundsComponentAddress,
    })
    .then((response) => {
      let vector = response.data.details.state.data_json[0];
      document.getElementById("rcptFunds").innerText = vector
        .map((arr) => arr.join("\n"))
        .join("\n\n");
    });
};

// ************ Set Fund Address *************
document.getElementById("btnSetFundAddress").onclick = async function () {
  FundComponentAddress = document.getElementById("inpSetFundAddress").value;
  FundManagerBadge = document.getElementById("inpSetFundManagerBadge").value;
  ShareTokenAddress = document.getElementById("inpSetShareToken").value;
};

// ************ Get pool info *************
document.getElementById("btnGetPoolInfo").onclick = async function () {
  console.log("Need to update the pool addreses to correct ones");
  let selectElement = document.getElementById("selGetPoolInfo");
  let value = selectElement.options[selectElement.selectedIndex].value;
  let addresses = value.split(",");
  let address1 = addresses[0];
  let address2 = addresses[1];
  let noe = await requestPoolInfo(address1, address2);
  //let noe = await request_pool_info();
  console.log(noe);
  console.log("Price: ", calculatePrice(noe));
};

// ************ Deposit tokens to fund *************
document.getElementById("btnDeposit").onclick = async function () {
  document.getElementById("StatusDeposit").innerText =
    "not working yet. Ratios need to be made";
  //The ratios should be on this format Vec<(ResourceAddress, Decimal)>
  let ratios = await getRatios(FundComponentAddress);
  let ratioTuples = [];
  for (let [address, ratio] of ratios) {
    ratioTuples.push(Tuple(ResourceAddress(address), Decimal(ratio)));
  }
  console.log(ratios);
  let amount = document.getElementById("inpDepositFromNumber").value;
  let selectElement = document.getElementById("selDepositFromAddress");
  let address = selectElement.options[selectElement.selectedIndex].value;
  let manifest = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountAddress, amount, address)
    .takeFromWorktopByAmount(amount, address, "bucket")
    .callMethod(FundComponentAddress, "swap_token_for_tokens", [
      Bucket("bucket"),
      Array("Tuple", ...ratioTuples),
    ])
    .callMethod(FundComponentAddress, "deposit_tokens_to_fund", [
      Expression("ENTIRE_WORKTOP"), //this is a vec of all buckets on worktop
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusDeposit").innerText =
    commitReceipt.details.receipt.status;
};

// ************ Withdraw tokens from fund *************
//problemer her med at sharetoken bucket blir en del av entire worktop?
document.getElementById("btnWithdraw").onclick = async function () {
  let amount = document.getElementById("inpWithdrawFromNumber").value;
  let selectElement = document.getElementById("selWithdrawToAddress");
  let address = selectElement.options[selectElement.selectedIndex].value;

  let manifest = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountAddress, amount, ShareTokenAddress)
    .takeFromWorktop(ShareTokenAddress, "bucket")
    .callMethod(FundComponentAddress, "withdraw_tokens_from_fund", [
      Bucket("bucket"),
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    ///////////////////////////////////////////////////////////
    .withdrawFromAccountByAmount(accountAddress, amount, addr.XRD)
    .callMethod(FundComponentAddress, "swap_tokens_for_token", [
      Expression("ENTIRE_WORKTOP"),
      ResourceAddress(address),
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusWithdraw").innerText =
    commitReceipt.details.receipt.status;
};

// ************ Withdraw collected fee Fund Manager *************
document.getElementById("btnWithdrawCollectedFeeFundManager").onclick =
  async function () {
    let manifest = new ManifestBuilder()
      .createProofFromAccountByAmount(accountAddress, 1, FundManagerBadge)
      .callMethod(
        FundComponentAddress,
        "withdraw_collected_fee_fund_manager",
        []
      )
      .callMethod(accountAddress, "deposit_batch", [
        Expression("ENTIRE_WORKTOP"),
      ])
      .build()
      .toString();

    console.log("Manifest: ", manifest);

    const { commitReceipt } = await sendManifest(manifest);

    document.getElementById("StatusWithdrawCollectedFeeFundManager").innerText =
      commitReceipt.details.receipt.status;
  };

// ************ Change Deposit fee fundmanager *************
document.getElementById("btnChangeDepositFeeFundManager").onclick =
  async function () {
    let newFee = document.getElementById("inpChangeDepositFundManager").value;
    let manifest = new ManifestBuilder()
      .createProofFromAccountByAmount(accountAddress, 1, FundManagerBadge)
      .callMethod(FundComponentAddress, "change_deposit_fee_fund_manager", [
        Decimal(newFee),
      ])
      .build()
      .toString();

    console.log("Manifest: ", manifest);

    const { commitReceipt } = await sendManifest(manifest);

    document.getElementById("StatusChangeDepositFeeFundManager").innerText =
      commitReceipt.details.receipt.status;
  };

// ************ Change Description *************
document.getElementById("btnChangeDescription").onclick = async function () {
  let text = document.getElementById("inpChangeDescription").value;
  let manifest = new ManifestBuilder()
    .createProofFromAccountByAmount(accountAddress, 1, FundManagerBadge)
    .callMethod(FundComponentAddress, "change_short_description", [`"${text}"`])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusChangeDescription").innerText =
    commitReceipt.details.receipt.status;
};

// ************ Change Image *************
document.getElementById("btnChangeImage").onclick = async function () {
  let text = document.getElementById("inpChangeImage").value;
  let manifest = new ManifestBuilder()
    .createProofFromAccountByAmount(accountAddress, 1, FundManagerBadge)
    .callMethod(FundComponentAddress, "change_image_link", [`"${text}"`])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusChangeImage").innerText =
    commitReceipt.details.receipt.status;
};

// ************ Change Website *************
document.getElementById("btnChangeWebsite").onclick = async function () {
  let text = document.getElementById("inpChangeWebsite").value;
  let manifest = new ManifestBuilder()
    .createProofFromAccountByAmount(accountAddress, 1, FundManagerBadge)
    .callMethod(FundComponentAddress, "change_website_link", [`"${text}"`])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusChangeWebsite").innerText =
    commitReceipt.details.receipt.status;
};

//remeber to whitelist the pool before testing

//pool
//xrd
// ************ Trade Beakerfi *************
document.getElementById("btnTrade").onclick = async function () {
  let amount = document.getElementById("inpTradeAmount").value;
  let selectElement1 = document.getElementById("selTradeFromAddress");
  let address = selectElement1.options[selectElement1.selectedIndex].value;
  let selectElement2 = document.getElementById("selTradeComponentAddress");
  let componentAddress =
    selectElement2.options[selectElement2.selectedIndex].value;

  let manifest = new ManifestBuilder()
    .createProofFromAccountByAmount(accountAddress, 1, FundManagerBadge)
    .callMethod(FundComponentAddress, "trade_beakerfi", [
      ResourceAddress(address),
      Decimal(amount),
      ComponentAddress(componentAddress),
    ])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();

  console.log("Manifest: ", manifest);

  const { commitReceipt } = await sendManifest(manifest);

  document.getElementById("StatusTrade").innerText =
    commitReceipt.details.receipt.status;
};
