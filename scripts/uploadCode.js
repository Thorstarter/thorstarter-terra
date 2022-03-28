const { uploadCode } = require("./utils");

async function main() {
  const codeId = await uploadCode("../saleCommit/artifacts/saleCommit.wasm");
  console.log("codeId", codeId);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
