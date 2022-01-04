const { MsgExecuteContract } = require("@terra-money/terra.js");
const { walletAddress, sendTransaction } = require("./utils");

async function main() {
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra10f7w8d5kdzwhlclyk73j887ws8r35972kgzusx",
      {
        /*
        configure: {
          token: "terra1uj2txzk03yq9exxqckuryqf548qjqlphyryjjy",
          start_time: 1641227400,
          end_time: 1641245400,
          raising_amount: "500000000000",
          offering_amount: "20000000000000",
          vesting_initial: "100000",
          vesting_time: 15552000,
          finalized: false,
          merkle_root: "85645aa5585433e517505710c35578e78da0ae57189bfc0dd47e2ef5fcb55806",
          //merkle_root: "e7a4101e22345b5280871e8d70bf50c986a3a738b9890ca4bb741937586a1daa",
        },
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
