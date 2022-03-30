const { MsgExecuteContract } = require("@terra-money/terra.js");
const { walletAddress, sendTransaction } = require("./utils");

async function main() {
  let now = (Date.now() / 1000) | 0;
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra1wyjx8t64rswat0a6kyu0tvcvsu3a5hcpc0t683",
      {
        configure: {
          token: "terra1mt2ytlrxhvd5c4d4fshxxs3zcus3fkdmuv4mk2",
          start_time: 1648821600,
          end_deposit_time: 1648994400,
          end_withdraw_time: 1649080800,
          min_price: "50000",
          offering_amount: "10000000" + "000000",
          vesting_initial: "1000000",
          vesting_time: 1,
          finalized: true,
          merkle_root: "",
        },
        /*
        collect: {},
        */
      }
    ),
  ]);
  console.log("result", result);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
