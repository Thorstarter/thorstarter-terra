const { MsgExecuteContract } = require("@terra-money/terra.js");
const { walletAddress, sendTransaction } = require("./utils");

async function main() {
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra10f7w8d5kdzwhlclyk73j887ws8r35972kgzusx",
      {
        configure: {
          token: "terra1vwz7t30q76s7xx6qgtxdqnu6vpr3ak3vw62ygk",
          start_time: 1641223800,
          end_time: 1646247600,
          raising_amount: "500000" + "000000",
          offering_amount: "20000000" + "000000",
          vesting_initial: "100000",
          vesting_time: 15552000,
          finalized: true,
          merkle_root:
            "49b51150f947ef3f986ed17ab3f9be42c641d7c4242bc6168c51f62c3da828a3",
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
