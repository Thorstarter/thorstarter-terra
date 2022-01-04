const { MsgInstantiateContract } = require("@terra-money/terra.js");
const {
  terra,
  walletAddress,
  sendTransaction,
  uploadCode,
} = require("./utils");

const tokenAddress = "terra1td743l5k5cmfy7tqq202g7vkmdvq35q48u2jfm";

async function main() {
  const codeId = await uploadCode(
    "../tiers/artifacts/thorstarter_terra_tiers.wasm"
  );
  console.log("codeId", codeId);

  const tx = await sendTransaction([
    new MsgInstantiateContract(walletAddress, walletAddress, codeId, {
      token: tokenAddress,
      locked_period: 604800,
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
