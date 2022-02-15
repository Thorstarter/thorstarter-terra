const { MsgInstantiateContract } = require("@terra-money/terra.js");
const {
  terra,
  walletAddress,
  sendTransaction,
  uploadCode,
} = require("./utils");

async function main() {
  /*
  const codeId = await uploadCode("../sale/artifacts/sale.wasm");
  console.log("codeId", codeId);
  /**/
  const codeId = 2090;

  const tx = await sendTransaction([
    new MsgInstantiateContract(walletAddress, walletAddress, codeId, {
      token: "terra16zrhr55k9syrlmeqtae9ahccpgld843gu04qp6",
      start_time: 1644939000,
      end_time: 1645025400,
      raising_amount: "300000"+"000000",
      offering_amount: "3529411"+"764705",
      vesting_initial: "0",
      vesting_time: 10368000,
      merkle_root:
        "49b51150f947ef3f986ed17ab3f9be42c641d7c4242bc6168c51f62c3da828a3",
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
