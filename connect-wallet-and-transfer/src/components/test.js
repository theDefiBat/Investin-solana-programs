
const { MangoClient, IDS, MangoGroup, sleep } = require('@blockworks-foundation/mango-client');
const { Account, Connection, PublicKey } = require('@solana/web3.js');
const { Market, TokenInstructions, OpenOrders, Orderbook } = require('@project-serum/serum');
const { Order } = require('@project-serum/serum/lib/market');

const fs = require('fs')

async function main() {
    const client = new MangoClient()
    const cluster = 'mainnet-beta'
    const clusterUrl = process.env.CLUSTER_URL || IDS.cluster_urls[cluster]
    const connection = new Connection(clusterUrl, 'singleGossip')
    const programId = new PublicKey(IDS[cluster].mango_program_id)
    const dexProgramId = new PublicKey(IDS[cluster].dex_program_id)
    const mangoGroupPk = new PublicKey(IDS[cluster].mango_groups['BTC_ETH_SOL_SRM_USDC'].mango_group_pk)
  
    const keyPairPath = '/home/muneeb/.config/solana/id.json'
  
    const payer = new Account(JSON.parse(fs.readFileSync(keyPairPath, 'utf-8')))
    const mangoGroup = await client.getMangoGroup(connection, mangoGroupPk)
    const prices = await mangoGroup.getPrices(connection)
    const marginAccounts = (await client.getMarginAccountsForOwner(connection, programId, mangoGroup, payer))
    marginAccounts.sort(
      (a, b) => (a.computeValue(mangoGroup, prices) > b.computeValue(mangoGroup, prices) ? -1 : 1)
    )
    let marginAccount = marginAccounts[0]
  
    const market = await Market.load(connection, mangoGroup.spotMarkets[2], { skipPreflight: true, commitment: 'singleGossip'}, mangoGroup.dexProgramId)
    console.log('placing order')
    const txid = await client.placeOrder(connection, programId, mangoGroup, marginAccount, market, payer, 'buy', 34.3, 0.25)
    console.log('order placed')
  
    await sleep(5000)

    const txid2 = await client.settleFunds(connection, programId, mangoGroup, marginAccount, payer, market);
    console.log('settle done')

    // marginAccount = await client.getMarginAccount(connection, marginAccount.publicKey, mangoGroup.dexProgramId)
    // const bids = await market.loadBids(connection)
    // const asks = await market.loadAsks(connection)
    // console.log('canceling orders')
    // await marginAccount.cancelAllOrdersByMarket(connection, client, programId, mangoGroup, market, bids, asks, payer)
    // console.log('orders canceled')
  }

  main()