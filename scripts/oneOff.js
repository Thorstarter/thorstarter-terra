const { MsgExecuteContract } = require("@terra-money/terra.js");
const { walletAddress, sendTransaction } = require("./utils");

async function main() {
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra1dvlkmlfa5j0sdzj3f99a4dlhguu9k4acdt5nzx",
      {
        configure: {
          token: "terra1tnxg0hpk6rgsk2kjx8mtrqv5lnn38fhk3styuh",
          start_time: 1644939000,
          end_time: 1645025400,
          raising_amount: "300000"+"000000",
          offering_amount: "3529411"+"764705",
          vesting_initial: "125000",
          vesting_time: 10368001,
          finalized: false,
          merkle_root: "5d2c6ad6ee9205b48cdbd85ad13538f66cd8f8781462e4c23f9e4b40ccd3362b",
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
