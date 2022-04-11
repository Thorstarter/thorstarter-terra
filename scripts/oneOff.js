const { MsgExecuteContract, MsgSend } = require("@terra-money/terra.js");
const { terra, walletAddress, sendTransaction } = require("./utils");

async function main() {
  let now = (Date.now() / 1000) | 0;
  const result = await sendTransaction([
    new MsgExecuteContract(
      walletAddress,
      "terra16ewzuu492jt9nvxruhjtt554f4au9r6j05qa76",
      {
        //collect: {},
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

// refund
// https://columbus-fcd.terra.dev/v1/txs?offset=259634049&limit=100&account=terra16ewzuu492jt9nvxruhjtt554f4au9r6j05qa76
// setTimeout(async () => {await navigator.clipboard.writeText(JSON.parse(document.body.innerText).txs.filter(t => { console.log(t); return t.tx.value.msg.length > 0 && t.tx.value.msg[0].value.coins && t.tx.value.msg[0].value.coins.length > 0; }).map(t => ({s: t.tx.value.msg[0].value.sender, v: t.tx.value.msg[0].value.coins[0].amount})).filter(t => t.s !== "terra16ewzuu492jt9nvxruhjtt554f4au9r6j05qa76" && t.s !== "terra16zrhr55k9syrlmeqtae9ahccpgld843gu04qp6").map(t => t.s+','+t.v).join('\n'));console.log('ok')}, 1000);
/*
async function main() {
  const rows = require("fs")
    .readFileSync("1.csv", "utf-8")
    .split("\n")
    .slice(1)
    .filter((r) => r);
  let total = 0;
  let messages = [];
  for (let i in rows) {
    const [address, amount] = rows[i].split(",");
    total += parseInt(amount) / 1000000;
    messages.push(new MsgSend(walletAddress, address, { uusd: amount }));
    console.log(
      [i, (parseInt(amount) / 1000000).toFixed(2), address].join("\t")
    );
  }
  console.log(total);
  const result = await sendTransaction(messages);
  console.log("result", result);
}
*/

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
