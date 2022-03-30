const { MsgExecuteContract } = require("@terra-money/terra.js");
const { walletAddress, sendTransaction } = require("./utils");

async function main() {
  let now = (Date.now() / 1000) | 0;
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra1yajdgg725s49kazrvss74p3q9hex69tln3wqdp",
      {
        /*
        configure: {
          token: "terra1td743l5k5cmfy7tqq202g7vkmdvq35q48u2jfm",
          start_time: now,
          end_deposit_time: now + 300,
          end_withdraw_time: now + 600,
          min_price: "100000",
          offering_amount: "10000000" + "000000",
          vesting_initial: "1000000",
          vesting_time: 1,
          finalized: false,
          merkle_root: "",
        },
        */
        collect: {},
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
