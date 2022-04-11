const { MsgInstantiateContract } = require("@terra-money/terra.js");
const {
  terra,
  walletAddress,
  sendTransaction,
  uploadCode,
} = require("./utils");

async function main() {
  const codeId = 2090; // Sale
  //const codeId = 4107; // Sale Commit

  const tx = await sendTransaction([
    new MsgInstantiateContract(walletAddress, walletAddress, codeId, {
      token: "terra1td743l5k5cmfy7tqq202g7vkmdvq35q48u2jfm",
      start_time: 1649777400,
      end_time: 1649863800,
      raising_amount: "300000" + "000000",
      offering_amount: "5000000" + "000000",
      vesting_initial: "100000000000",
      vesting_time: 15552000,
      merkle_root: "",
    }),
  ]);
  console.log("tx", tx);
  const address = JSON.parse(tx.raw_log)[0].events[1].attributes[3].value;
  console.log("address", address);
  console.log("state", await terra.wasm.contractQuery(address, { state: {} }));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
