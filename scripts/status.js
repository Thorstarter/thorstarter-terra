const { Decimal } = require("decimal.js");
const { terra } = require("./utils");

const tiersAddress = "terra18s7n93ja9nh37mttu66rhtsw05dxrcpsmw0c45";
const saleAddress = "terra10f7w8d5kdzwhlclyk73j887ws8r35972kgzusx";

async function main() {
  let i = 0;
  console.log("number,address,tier,xrune,salexrune,mul,allo");
  for (let address of addresses()) {
    const tiersState = await terra.wasm.contractQuery(tiersAddress, {
      user_state: { user: address },
    });
    const saleState = await terra.wasm.contractQuery(saleAddress, {
      user_state: { user: address, now: 0 },
    });
    i++;
    const amount = Decimal(tiersState.balance).div("1000000");
    const tier = tierFromAmount(amount);
    console.log(
      [
        i,
        address,
        tier,
        amount.toFixed(2),
        Decimal(saleState.amount).div("1000000").toFixed(2),
        tierMultiplier(tier),
        tierMultiplier(tier) * 137,
      ].join(",")
    );
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

function tierMultiplier(tier) {
  switch (tier) {
    case 5:
      return 12;
    case 4:
      return 8;
    case 3:
      return 4;
    case 2:
      return 2;
    case 1:
      return 1;
    default:
      return 1;
  }
}

function tierFromAmount(amount) {
  if (amount.gte("150000")) return 5;
  if (amount.gte("75000")) return 4;
  if (amount.gte("25000")) return 3;
  if (amount.gte("7500")) return 2;
  if (amount.gte("2500")) return 1;
  return 0;
}

function addresses() {
  return `...`.split("\n");
}
