const nearAPI = require("near-api-js");
const fs = require("fs");

async function main() {
  console.log("Hello world");

  const content = fs.readFileSync("seal.bin").toString('base64')

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

  dev_acct = "dev-1649632081005-14076690372915";
  // connect to NEAR
  const { connect } = nearAPI;
  const { utils } = nearAPI;
  near = await connect(config);
  console.log("Connected");
  account = await near.account(dev_acct);
  console.log(await account.getAccountBalance());

  const MAX_GAS = "300000000000000";
  const result = await account.functionCall({
    contractId : dev_acct, 
    methodName : "verify",
    args: {seal_str : content },
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

