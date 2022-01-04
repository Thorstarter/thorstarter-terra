const fs = require("fs");
const path = require("path");
const terraSdk = require("@terra-money/terra.js");
const {
  isTxError,
  Coin,
  LCDClient,
  MnemonicKey,
  MsgInstantiateContract,
  MsgStoreCode,
  MsgExecuteContract,
  Fee,
} = terraSdk;

const chain = "mainnet";
const config = {
  local: {
    url: "http://localhost:1317",
    chainID: "localterra",
    mnemonic:
      "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius",
  },
  testnet: {
    url: "https://bombay-lcd.terra.dev",
    chainID: "bombay-12",
    mnemonic: process.env.TERRA_PK_TEST, // terra1uj2txzk03yq9exxqckuryqf548qjqlphyryjjy
  },
  mainnet: {
    url: "https://lcd.terra.dev",
    chainID: "columbus-5",
    mnemonic: process.env.TERRA_PK_MAIN, // terra16zrhr55k9syrlmeqtae9ahccpgld843gu04qp6
  },
};
const terra = new LCDClient({
  URL: config[chain].url,
  chainID: config[chain].chainID,
});
if (!config[chain].mnemonic) throw new Error("Missing mnemonic!");
const key = new MnemonicKey({ mnemonic: config[chain].mnemonic });
const wallet = terra.wallet(key);
const walletAddress = key.accAddress;

async function sendTransaction(msgs) {
  const tx = await wallet.createAndSignTx({
    msgs,
    // fee: new Fee(30000000, [
    //   new Coin("uluna", 4500000),
    //   new Coin("uusd", 4500000),
    // ]),
  });
  const result = await terra.tx.broadcast(tx);
  if (isTxError(result)) {
    throw new Error(
      `Error: ${result.code}: ${result.codespace}\n${result.raw_log}`
    );
  }
  return result;
}

async function uploadCode(path) {
  const code = fs.readFileSync(path).toString("base64");
  const storeCodeTx = await sendTransaction(terra, wallet, [
    new MsgStoreCode(walletAddress, code),
  ]);
  const codeId = parseInt(
    storeCodeTx.logs[0].eventsByType.store_code.code_id[0]
  );
  return codeId;
}

module.exports = { terra, wallet, walletAddress, sendTransaction, uploadCode };
