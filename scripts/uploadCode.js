const { uploadCode } = require("./utils");

async function main() {
  const codeId = await uploadCode("../saleDeposit/artifacts/saleDeposit.wasm");
  console.log("codeId", codeId);
}
