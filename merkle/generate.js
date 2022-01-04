const { MerkleTree } = require("merkletreejs");
const keccak256 = require("keccak256");

const participantsStr = `
0x0000,terra1uj2txzk03yq9exxqckuryqf548qjqlphyryjjy,0,0,1
`;

const totals = {};
for (let p of participantsStr
  .split("\n")
  .map((a) => a.trim())
  .filter((a) => a)) {
  const parts = p.split(",");
  totals[parts[1]] = (totals[parts[1]] || 0.0) + parseFloat(parts[4]);
}
const participants = Object.keys(totals).map((k) => ({
  address: k,
  allocation:
    (totals[k] * 100) | (0 === 0)
      ? "0"
      : String((totals[k] * 100) | 0) + "0000",
}));
const elements = participants.map((p) =>
  keccak256(p.address + "," + p.allocation)
);
const merkleTree = new MerkleTree(elements, keccak256, { sort: true });

console.log("merkle root", merkleTree.getHexRoot().slice(2));

const result = participants.map((a, i) => {
  a.proof = merkleTree.getHexProof(elements[i]).map((p) => p.slice(2));
  return a;
});

require("fs").writeFileSync("addresses.json", JSON.stringify(result, null, 2));
