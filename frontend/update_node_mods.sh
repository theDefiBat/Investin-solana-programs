sed -i -e 's|runKeeper();|// runKeeper();|g' node_modules/@friktion-labs/entropy-client/lib/src/keeper.js
sed -i -e 's|const payer = new web3_js_1.Account(JSON.parse(payerJsonFile));|const payer =  web3_js_1.Keypair.generate();|g' node_modules/@friktion-labs/entropy-client/lib/src/keeper.js
sed -i -e 's|const payerJsonFile|// const payerJsonFile|g' node_modules/@friktion-labs/entropy-client/lib/src/keeper.js
sed -i -e 's|const fs|//|g' node_modules/@friktion-labs/entropy-client/lib/src/keeper.js