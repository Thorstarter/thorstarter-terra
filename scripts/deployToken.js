const { MsgInstantiateContract } = require("@terra-money/terra.js");
const {
  terra,
  walletAddress,
  sendTransaction,
  uploadCode,
} = require("./utils");

async function main() {
  /*
  const codeId = await uploadCode("./cw20_base.wasm");
  console.log("codeId", codeId);
  /**/
  const codeId = 1;

  const tx = await sendTransaction([
    new MsgInstantiateContract(walletAddress, walletAddress, codeId, {
      name: "XRUNE Token",
      symbol: "XRUNE",
      decimals: 6,
      initial_balances: [{ address: walletAddress, amount: "10000000000000" }],
    }),
  ]);
  console.log("tx", tx);
  const address = JSON.parse(tx.raw_log)[0].events[1].attributes[3].value;
  console.log("address", address);
  console.log(
    "state",
    await terra.wasm.contractQuery(address, { token_info: {} })
  );
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
