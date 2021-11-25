import { Account, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, FUND_ACCOUNT_KEY, programId, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID } from '../utils/constants';
import { struct, u32, u8, u16, ns64, nu64 } from 'buffer-layout';
import { createAssociatedTokenAccountIfNotExist, signAndSendTransaction } from '../utils/web3';
import { FUND_DATA } from '../utils/programLayouts';
import { Card, Col, Row ,Table} from 'reactstrap';



import { IDS, MangoClient, I80F48,
   NodeBankLayout, PerpAccountLayout, PerpMarket, PerpMarketLayout,
   Config ,getAllMarkets, getMarketByPublicKey, getMultipleAccounts,
   BookSide, BookSideLayout
  } from '@blockworks-foundation/mango-client';

export const MangoPlaceOrder = () => {
    const [size, setSize] = useState(0);
    const [price, setPrice] = useState(0);
    const [index, setIndex] = useState(0);
    const [side, setSide] = useState('');

    const [openOrders, setOpenOrders] = useState([])
    const [perpPositions, setPerpPositions] = useState([])
    
    const walletProvider = GlobalState.useState(s => s.walletProvider);
    const ids = IDS['groups'][0]
    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
  
    const handleMangoDeposit = async () => {
    
    const key = walletProvider?.publicKey;

    if (!key ) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    let fundStateInfo = await connection.getAccountInfo((fundStateAccount))
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
      let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
      console.log("mango group:: ", mangoGroup)

      let mangoAcc = await client.getMangoAccount(new PublicKey('7BLzTNvjNjaCnZ2Nnpu1aFYqTBsL8Lz2FUxknSAZ8tDX'), ids.serumProgramId)
      console.log("manogACc:: ", mangoAcc)
      let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].nodeKeys[0]))
      let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
      console.log("nodebank:: ", nodeBank)

    const dataLayout = struct([u32('instruction'), nu64('quantity')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 6,
        quantity: size * 10** ids.tokens[0].decimals
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: fundStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },

        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: false },
        { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
        { pubkey: fundState.fund_pda, isSigner: false, isWritable: false },
        { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.tokens[0].rootKey), isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.tokens[0].nodeKeys[0]), isSigner: false, isWritable: true },
        { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
        { pubkey: fundState.vault_key, isSigner: false, isWritable: true },
      ],
      programId,
      data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)
    
  }

    const handleMangoPlace = async () => {

      console.log("---handleMangoPlace side::", side)

      console.log("---handleMangoPlace size,price::", size,price)
        
      const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const fundStateAccount = await PublicKey.createWithSeed(
        key,
        FUND_ACCOUNT_KEY,
        programId,
      );
  
    let fundStateInfo = await connection.getAccountInfo(fundStateAccount)
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)
    console.log("vault_balance:: ", fundState.vault_balance.toNumber()/ 10 ** ids.tokens[0].decimals)

    let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].nodeKeys[0]))
    let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
    console.log("nodebank:: ", nodeBank)

    
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    let mangoAcc = await client.getMangoAccount(fundState.mango_account, ids.serumProgramId)
    console.log("mangoAcc:: ", mangoAcc)
    console.log("mangogroup:: ", mangoGroup)

    let mangoCache = await mangoGroup.loadCache(connection)
    console.log("mangocache:: ", mangoCache)

    let perpMarket = await client.getPerpMarket( mangoGroup.perpMarkets[fundState.perp_market_index].perpMarket,  mangoGroup.tokens[1].decimals, mangoGroup.tokens[15].decimals )
    console.log("perpmarket:: ", perpMarket)

      
    const idsPerpmarket = fundState.perp_market_index == 1 ? 0 : 1

    const market =  mangoGroup.perpMarkets[fundState.perp_market_index]
    const lotSizerPrice = market.baseLotSize/ market.quoteLotSize;
    console.log("lotSizerPrice:",lotSizerPrice)
    // const lotSizerPrice = fundState.perp_market_index === 1 ? 10 : 100000;
    const decimals = 6 //for BTC
    const sizeMultiplier = decimals/market.baseLotSize;  // BTC = 4

    let cachePrice = (mangoCache.priceCache[fundState.perp_market_index].price * lotSizerPrice)
    let price_adj = side == 1 ? cachePrice * 0.95 : cachePrice*1.05
     console.log("price_adj:",price_adj)

    const transaction = new Transaction()

    const dataLayout = struct([u32('instruction'), ns64('price'), ns64('quantity'), nu64('client_order_id'), u8('side'), u8('order_type')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 8,
            price: price_adj,
            quantity: Math.abs(size* sizeMultiplier),
            client_order_id: 21343,
            side: 0,
            order_type: 0
        },
        data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: fundStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        
        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
        { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
        { pubkey: fundPDA[0], isSigner: false, isWritable: true },
        { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: true },

        { pubkey: new PublicKey(ids.perpMarkets[1].publicKey) , isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.perpMarkets[1].bidsKey) , isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.perpMarkets[1].asksKey) , isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.perpMarkets[1].eventsKey) , isSigner: false, isWritable: true },

        { pubkey: SYSTEM_PROGRAM_ID , isSigner: false, isWritable: false },

      ],
      programId,
      data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)

    }

    const handleMangoClosePosition = async () => {
        
    
      const key = walletProvider?.publicKey;

    if (!key ) {
      alert("connect wallet")
      return;
    };
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    let fundStateInfo = await connection.getAccountInfo(fundStateAccount)
  let fundState = FUND_DATA.decode(fundStateInfo.data)
  console.log("fundState:: ", fundState)

  let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
  let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
  let mangoAcc = await client.getMangoAccount(fundState.mango_account, ids.serumProgramId)
  // let mangoAcc = await client.getMangoAccount(new PublicKey('9rzuDYREjQ1UoiXgU2gJmixik5J2vSn5DoWitzKAmeJm'), ids.serumProgramId)

  let mangoCache = await mangoGroup.loadCache(connection)
  let price = (mangoCache.priceCache[1].price * 10)
  let price_adj = mangoAcc.perpAccounts[1].basePosition > 0 ? price * 0.95 : price * 1.05
  console.log("mangoCAche:: ", mangoCache)

  console.log("mangoAcc:: ", mangoAcc)
  console.log("mangogroup:: ", mangoGroup)
 
  const transaction = new Transaction()

  const dataLayout = struct([u32('instruction'), ns64('price'), ns64('quantity'), nu64('client_order_id'), u8('side'), u8('order_type')])
  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction: 8,
      price: price_adj,
      quantity: Math.abs(mangoAcc.perpAccounts[1].basePosition),
      client_order_id: 333,
      side: mangoAcc.perpAccounts[1].basePosition < 0 ? 0: 1,
      order_type: 0
    },
    data
  )

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: fundStateAccount, isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      
      { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
      { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
      { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: true },

      { pubkey: new PublicKey(ids.perpMarkets[0].publicKey) , isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.perpMarkets[0].bidsKey) , isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.perpMarkets[0].asksKey) , isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.perpMarkets[0].eventsKey) , isSigner: false, isWritable: true },

      { pubkey: SYSTEM_PROGRAM_ID , isSigner: false, isWritable: false },

    ],
    programId,
    data
  });

  transaction.add(instruction);
  console.log(`transaction ::: `, transaction)
  console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
  transaction.feePayer = key;
  let hash = await connection.getRecentBlockhash("finalized");
  console.log("blockhash", hash);
  transaction.recentBlockhash = hash.blockhash;

  const sign = await signAndSendTransaction(walletProvider, transaction);
  console.log("tx::: ", sign)

  }

    const handleMangoWithdraw = async () => {
        
    
      const key = walletProvider?.publicKey;

    if (!key ) {
      alert("connect wallet")
      return;
    };
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );
    let fundStateInfo = await connection.getAccountInfo((fundStateAccount))
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    console.log("mango group:: ", mangoGroup)

    let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].nodeKeys[0]))
    let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
    console.log("nodebank:: ", nodeBank)

    const transaction = new Transaction()

    const dataLayout = struct([u32('instruction'), nu64('quantity')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 7,
        quantity: size * 10**ids.tokens[0].decimals
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: fundStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },

        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
        { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
        { pubkey: fundPDA[0], isSigner: false, isWritable: false },
        { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.tokens[0].rootKey), isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.tokens[0].nodeKeys[0]), isSigner: false, isWritable: true },
        { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
        { pubkey: fundState.vault_key, isSigner: false, isWritable: true }, // Fund Vault
        { pubkey: mangoGroup.signerKey, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
        { pubkey: PublicKey.default, isSigner: false, isWritable: true },
      ],
      programId,
      data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)

  }

  const handleMangoCancelPerp = async () => {
        
    
    const key = walletProvider?.publicKey;

  if (!key ) {
    alert("connect wallet")
    return;
  };
  const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
  const fundStateAccount = await PublicKey.createWithSeed(
    key,
    FUND_ACCOUNT_KEY,
    programId,
  );
  let fundStateInfo = await connection.getAccountInfo((fundStateAccount))
  let fundState = FUND_DATA.decode(fundStateInfo.data)
  console.log("fundState:: ", fundState)

  let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
  let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
  console.log("mango group:: ", mangoGroup)

  let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].nodeKeys[0]))
  let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
  console.log("nodebank:: ", nodeBank)

  const transaction = new Transaction()

  const dataLayout = struct([u32('instruction'), nu64('client_order_id'), u8('invalid_ok')])
  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction: 9,
      client_order_id: 333,
      invalid_ok: 0,
    },
    data
  )

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: fundStateAccount, isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },

      { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
      { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: false },

      { pubkey: new PublicKey(ids.perpMarkets[0].publicKey) , isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.perpMarkets[0].bidsKey) , isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.perpMarkets[0].asksKey) , isSigner: false, isWritable: true },
    ],
    programId,
    data
  });

  transaction.add(instruction);
  console.log(`transaction ::: `, transaction)
  console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
  transaction.feePayer = key;
  let hash = await connection.getRecentBlockhash("finalized");
  console.log("blockhash", hash);
  transaction.recentBlockhash = hash.blockhash;

  const sign = await signAndSendTransaction(walletProvider, transaction);
  console.log("tx::: ", sign)

  }

  const handleMangoRedeem = async () => {
        
    
    const key = walletProvider?.publicKey;

  if (!key ) {
    alert("connect wallet")
    return;
  };
  const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
  const fundStateAccount = await PublicKey.createWithSeed(
    key,
    FUND_ACCOUNT_KEY,
    programId,
  );
  let fundStateInfo = await connection.getAccountInfo((fundStateAccount))
  let fundState = FUND_DATA.decode(fundStateInfo.data)
  console.log("fundState:: ", fundState)

  let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
  let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
  console.log("mango group:: ", mangoGroup)

  let perpAcc = await connection.getAccountInfo(new PublicKey(ids.perpMarkets[0].publicKey))
  let perpMkt = PerpMarketLayout.decode(perpAcc.data)
  console.log("perpmkt:: ", perpMkt)

  let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[1].nodeKeys[0]))
  let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
  console.log("nodebank:: ", nodeBank)

  const transaction = new Transaction()

  const managerMngoAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[1].mintKey), key, transaction);

  const dataLayout = struct([u32('instruction')])
  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction: 4
    },
    data
  )

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: fundStateAccount, isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
      { pubkey: fundState.mngo_vault_key, isSigner: false, isWritable: true },
      { pubkey: managerMngoAccount , isSigner: false, isWritable: true },

      { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
      { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: false },
      { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: false },
      { pubkey: new PublicKey(ids.perpMarkets[0].publicKey), isSigner: false, isWritable: false },
      { pubkey: perpMkt.mngoVault, isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.tokens[1].rootKey), isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.tokens[1].nodeKeys[0]), isSigner: false, isWritable: true },
      { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
      { pubkey: mangoGroup.signerKey, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
      { pubkey: PublicKey.default, isSigner: false, isWritable: true },
    ],
    programId,
    data
  });

  transaction.add(instruction);
  console.log(`transaction ::: `, transaction)
  console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
  transaction.feePayer = key;
  let hash = await connection.getRecentBlockhash("finalized");
  console.log("blockhash", hash);
  transaction.recentBlockhash = hash.blockhash;

  const sign = await signAndSendTransaction(walletProvider, transaction);
  console.log("tx::: ", sign)

  }

  const  parsePerpOpenOrders = async(
    market,
    config,
    mangoAccount,
    accountInfos,
    fundState
  ) => {
    const idsPerpmarket = fundState.perp_market_index == 1 ? 0 : 1

    const bidData = (await connection.getAccountInfo(new PublicKey(ids.perpMarkets[idsPerpmarket].bidsKey))).data
    
    const askData = (await connection.getAccountInfo(new PublicKey(ids.perpMarkets[idsPerpmarket].asksKey))).data
  
    const bidOrderBook =
      market && bidData
        ? new BookSide(market.bids, market, BookSideLayout.decode(bidData))
        : []
    const askOrderBook =
      market && askData
        ? new BookSide(market.asks, market, BookSideLayout.decode(askData))
        : [] 

    const openOrdersForMarket = [...bidOrderBook, ...askOrderBook].filter((o) =>
      o.owner.equals(mangoAccount.publicKey)
    ) 
  
    return openOrdersForMarket.map((order) => ({
      order,
      market: { account: market, config: config },
    }))
  }

  const getOrdersandPositions = async () => {

    const key = walletProvider?.publicKey;
    if (!key ) {
      alert("connect wallet")
      return;
    };
    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );
  let fundStateInfo = await connection.getAccountInfo(fundStateAccount)
  if(!fundStateInfo){
    alert("NOT A MM FUND")
      return;
  }
  let fundState = FUND_DATA.decode(fundStateInfo.data)
    
    let mangoAcc = await client.getMangoAccount(fundState.mango_account, ids.serumProgramId)
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    const cache = await mangoGroup.loadCache(connection);

    // setBalances([{
    //   asset: 'USDC',
    //   balance: roundDownTo4Decimals(mangoAcc.getUiDeposit(cache.rootBankCache[15], mangoGroup, 15).toNumber()),
    //   fundBalance: roundDownTo4Decimals(fundState.vault_balance.toNumber() / 10 ** TOKENS.USDC.decimals)
    // }])

    const positions = [];
    const mangoPerpMarkets = ids.perpMarkets;

    for(let i=0; i<mangoPerpMarkets.length;i++){
      const perpMarket = mangoGroup.perpMarkets[i]
      const perpMarketCache = cache.perpMarketCache[i];
      const price = cache.priceCache[i].price;

      positions.push({
        market: mangoPerpMarkets[i].name,
        side: mangoAcc.perpAccounts[i].basePosition.toNumber() < 0 ? 'SHORT' : 'LONG',
        positionSize: (mangoAcc.perpAccounts[i].basePosition.toNumber() / 10 ** 4),
        pnl: ((mangoAcc.perpAccounts[i].getPnl(perpMarket, perpMarketCache, price)).toNumber() / 10 ** 6),
        mngoAccrued: mangoAcc.perpAccounts[i].mngoAccrued.toNumber()
      })
    }
    setPerpPositions(positions);

    // get open orders ::: 
     function zipDict( keys, values) {
        const result = {}
        keys.forEach((key, index) => {
            result[key] = values[index]
        })
        return result
     }
    const DEFAULT_MANGO_GROUP_CONFIG = Config.ids().getGroup(
      'mainnet',
      'mainnet.1'
    )
    const allMarketConfigs = getAllMarkets(DEFAULT_MANGO_GROUP_CONFIG)
    const allMarketPks = allMarketConfigs.map((m) => m.publicKey)

    const resp = await Promise.all([
      getMultipleAccounts(connection, allMarketPks),
      mangoGroup.loadCache(connection),
      mangoGroup.loadRootBanks(connection),
    ])
    const allMarketAccountInfos = resp[0]
    const allMarketAccounts = allMarketConfigs.map((config, i) => {
      if (config.kind == 'perp') {
        const decoded = PerpMarketLayout.decode(
          allMarketAccountInfos[i].accountInfo.data
        )
        return new PerpMarket(
          config.publicKey,
          config.baseDecimals,
          config.quoteDecimals,
          decoded
        )
      }
    })
    const markets = zipDict(
      allMarketPks.map((pk) => pk.toBase58()),
      allMarketAccounts
    )

    let slot = 0;
    const accountInfos = {};
    for (const { publicKey, context, accountInfo } of allMarketAccountInfos) {
      if (context.slot >= slot) {
        slot = context.slot
        accountInfos[publicKey.toBase58()] = accountInfo
      }
    }

    allMarketAccountInfos
      .forEach(({ publicKey, context, accountInfo }) => {
        if (context.slot >= slot) {
          slot = context.slot
          accountInfos[publicKey.toBase58()] = accountInfo
        }
      })

    let openOrders = Object.entries(markets).filter(([a, b]) => b != undefined).map(([address, market]) => {
      if (market) {
        const marketConfig = getMarketByPublicKey(DEFAULT_MANGO_GROUP_CONFIG, address)
        return parsePerpOpenOrders(
          market,
          marketConfig,
          mangoAcc,
          accountInfos,
          fundState
        )
      }
    })

    const orders = [];

    for (const order of openOrders) {
      orders.push(await order)
    }

    if (orders.length) {
      console.log(`orders ::: `, ...orders)
      setOpenOrders(...orders);
    }


  }

    return (
        <div className="form-div">
            <h4>Mango Place</h4>
            Size ::: {' '}
            <input type="number" value={size} onChange={(event) => setSize(event.target.value)} />
            <br />
            Price ::: {' '}
            <input type="number" value={price} onChange={(event) => setPrice(event.target.value)} />
            <br />
            <label htmlFor="side">Buy/Sell</label>

            <select name="side" width = "100px" onClick={(event) => setSide(event.target.value)}>
            <option value="buy">Buy</option>
            <option value="sell">Sell</option>
            </select>

            <select name="side" width = "100px" onClick={(event) => setIndex(event.target.value)}>
            <option value={0}>BTC</option>
            <option value={1}>ETH</option>
            <option value={2}>SOL</option>
            <option value={3}>SRM</option>

            </select>

          <button onClick={handleMangoPlace}>Mango Open Position</button>

          <br />
          <button onClick={handleMangoDeposit}>Deposit </button>
          <button onClick={handleMangoWithdraw}>Withdraw </button>

          <br />
          <button onClick={handleMangoClosePosition}>Mango Close </button>

          <br />
          <button onClick={handleMangoRedeem}>Redeem Mngo </button>

          <br />
          <button onClick={handleMangoCancelPerp}>Mango Cancel perp </button>

          <br/>
          <hr/>
          <br/>
          <button onClick={getOrdersandPositions}>GET ALL PERPS POSITIONS AND ORDERS </button>
         
                 <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}>
                        <thead className="text-primary">
                                        <tr>
                                          <th style={{ width: "15%" }}>index</th>
                                          <th style={{ width: "15%" }}>market</th>
                                          <th style={{ width: "15%" }}>side</th>
                                          <th style={{ width: "15%" }}>positionSize</th>
                                          <th style={{ width: "15%" }}>pnl</th>
                                          <th style={{ width: "15%" }}>mngoAccrued</th>
                                        </tr>
                        </thead>
                        <tbody>
                          {
                            perpPositions.length && 
                            perpPositions.map((i,x)=>{
                              return <tr key={x}>
                                <td >{x}</td>
                                <td >{i?.market}</td>
                                <td >{i?.side}</td>
                                <td >{i?.positionSize}</td>
                                <td >{i?.pnl}</td>
                                <td >{i?.mngoAccrued}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>


                <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}>
                        <thead className="text-primary">
                                        <tr>
                                          <th style={{ width: "15%" }}>index</th>
                                          <th scope="col">Market</th>
                                          <th scope="col">Side</th>
                                          <th scope="col">Position size</th>
                                          <th scope="col">Price</th>
                                        </tr>
                        </thead>
                        <tbody>
                          {
                            openOrders.length && 
                            openOrders.map((p,x)=>{
                              return <tr key={x}>
                                <td >{x}</td>
                                <td >{"FIND"}</td>
                                <td>{`${p.order.side}`.toUpperCase()}</td>
                                <td>{p.order.size}</td>
                                <td>{p.order.price}</td>
                                <td >{p?.mngoAccrued}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>
            


        </div>
    )
}