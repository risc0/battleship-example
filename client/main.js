const nearAPI = require("near-api-js");
const fs = require("fs");
const axios = require('axios')

const user_acct = "melvinz.testnet";
const contract_acct = "dev-1650087915725-26749817965169";
const game_name = "my_fun_game";
const init_state = {
  "ships": [
    {"pos":{"x":2,"y":3},"dir":"Vertical","hit_mask":0},
    {"pos":{"x":3,"y":1},"dir":"Horizontal","hit_mask":0},
    {"pos":{"x":4,"y":7},"dir":"Vertical","hit_mask":0},
    {"pos":{"x":7,"y":5},"dir":"Horizontal","hit_mask":0},
    {"pos":{"x":7,"y":7},"dir":"Horizontal","hit_mask":0}
  ],"salt":3735928559
};
const turn_state = {
  "state": init_state,
  "shot":{ "x":5,"y":5}
};

async function main() {
  console.log("Hello world");

  // Either init for new/join or turn for normal turns
  //res = await axios.post('http://127.0.0.1:3000/prove/init', init_state);
  res = await axios.post('http://127.0.0.1:3000/prove/turn', turn_state);
  receipt = res.data;

  const { keyStores } = nearAPI;
  const homedir = require("os").homedir();
  const CREDENTIALS_DIR = ".near-credentials";
  const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
  const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

  const config = {
      networkId: "testnet",
      keyStore: keyStore,
      nodeUrl: "https://rpc.testnet.near.org",
      walletUrl: "https://wallet.testnet.near.org",
      helperUrl: "https://helper.testnet.near.org",
      explorerUrl: "https://explorer.testnet.near.org",
  };

  // connect to NEAR
  const { connect } = nearAPI;
  const { utils } = nearAPI;
  near = await connect(config);
  console.log("Connected");
  account = await near.account(user_acct);
  console.log(await account.getAccountBalance());

  const MAX_GAS = "300000000000000";
  // One example of each call, uncomment to pick
  /*
  const result = await account.functionCall({
    contractId : contract_acct, 
    methodName : "new_game",
    args: {
      "name": game_name,
      "receipt_str": receipt
    },
    gas: MAX_GAS
  });
  */
  /*
  const result = await account.functionCall({
    contractId : contract_acct, 
    methodName : "join_game",
    args: {
      "name": game_name,
      "shot_x": 5,
      "shot_y": 5,
      "receipt_str": receipt
    },
    gas: MAX_GAS
  });
  */
  const result = await account.functionCall({
    contractId : contract_acct, 
    methodName : "turn",
    args: {
      "name": game_name,
      "shot_x": 5,
      "shot_y": 5,
      "receipt_str": receipt
    },
    gas: MAX_GAS
  });

  console.log(result);
  const { totalGasBurned, totalTokensBurned } = result.receipts_outcome.reduce(
    (acc, receipt) => {
      acc.totalGasBurned += receipt.outcome.gas_burnt;
      acc.totalTokensBurned += utils.format.formatNearAmount(
        receipt.outcome.tokens_burnt
      );
      return acc;
    },
    {
      totalGasBurned: result.transaction_outcome.outcome.gas_burnt,
      totalTokensBurned: utils.format.formatNearAmount(
        result.transaction_outcome.outcome.tokens_burnt
      ),
    }
  );
  console.log("Total GAS: " + totalGasBurned);
  console.log("Total Tokens: " + totalTokensBurned);
}

if (require.main === module) {
    main();
}

