const { MsgExecuteContract } = require("@terra-money/terra.js");
const { walletAddress, sendTransaction } = require("./utils");

async function main() {
  let now = (Date.now() / 1000) | 0;
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra16ewzuu492jt9nvxruhjtt554f4au9r6j05qa76",
      {
        configure: {
          token: "terra1td743l5k5cmfy7tqq202g7vkmdvq35q48u2jfm",
          start_time: 1649777400,
          end_time: 1649863800,
          raising_amount: "300000" + "000000",
          offering_amount: "5000000" + "000000",
          vesting_initial: "100000000000",
          vesting_time: 15552000,
          merkle_root: "",
          finalized: false,
        },
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
