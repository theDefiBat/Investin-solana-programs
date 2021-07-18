import {
    getFeeRates,
    getFeeTier,
    Market,
    OpenOrders,
  } from '@project-serum/serum'
import { Order, ORDERBOOK_LAYOUT } from '@project-serum/serum/lib/market'
import { connection, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, SERUM_PROGRAM_ID_V3, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, MARGIN_ACCOUNT_KEY, priceStateAccount} from '../utils/constants';
import { nu64, struct, u8, u32, u16 } from 'buffer-layout';
import BN from 'bn.js';
import {
  NUM_MARKETS,
  NUM_TOKENS,
} from '@blockworks-foundation/mango-client/lib/layout'
import {
  MangoClient
} from '@blockworks-foundation/mango-client'
import {
  nativeToUi,
  uiToNative,
  zeroKey,
} from '@blockworks-foundation/mango-client/lib/utils'

import {
    Account,
    Connection,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    SYSVAR_CLOCK_PUBKEY,
    SYSVAR_RENT_PUBKEY,
    Transaction,
    TransactionInstruction,
    TransactionSignature,
  } from '@solana/web3.js'

import { MarginAccountLayout } from './MangoLayout';
import { createKeyIfNotExists, findAssociatedTokenAddress } from './web3';
import { INVESTOR_DATA } from '../utils/programLayouts';
import { MANGO_TOKENS } from './tokens';



export const calculateMarketPrice = (
  orderBook,
  size,
  side
) => {
  let acc = 0
  let selectedOrder
  for (const order of orderBook) {
    acc += order.size
    if (acc >= size) {
      selectedOrder = order
      break
    }
  }

  if (side === 'buy') {
    return selectedOrder.price * 1.05
  } else {
    return selectedOrder.price * 0.95
  }
}

export async function placeAndSettle(
  connection,

  marginAcc,
  fundStateAccount,
  fundPDA,
  wallet,

  serumMarket,

  side,
  size,
  clientId,
  transaction
) {
  const client = new MangoClient()

  let marginAccount = await client.getMarginAccount(connection, marginAcc, SERUM_PROGRAM_ID_V3)
  let mangoGroup = await client.getMangoGroup(connection, MANGO_GROUP_ACCOUNT)
  console.log("mango group::", mangoGroup)

  console.log("margin acc::", marginAccount)

  let spotMarket = await Market.load(connection, serumMarket, {}, SERUM_PROGRAM_ID_V3)
  console.log("spot market:: ", spotMarket)
  console.log("margin acc:: ", marginAccount)

  let orderType = 'limit'
  let orderbook
  if (side === 'buy') {
    orderbook = await spotMarket.loadAsks(connection)
  }
  else {
    orderbook = await spotMarket.loadBids(connection)
  }
  console.log("orderbook", orderbook)
  let price = calculateMarketPrice(orderbook, size, side)
  console.log("price:: ", price)

  const limitPrice = spotMarket.priceNumberToLots(price)
  const maxBaseQuantity = spotMarket.baseSizeNumberToLots(size)

  const feeTier = getFeeTier(
    0,
    nativeToUi(mangoGroup.nativeSrm || 0, 6)
  )
  const rates = getFeeRates(feeTier)
  const maxQuoteQuantity = new BN(
    maxBaseQuantity
      .mul(limitPrice)
      .mul(spotMarket['_decoded'].quoteLotSize)
      .toNumber() *
    (1 + rates.taker)
  )

  console.log(maxBaseQuantity.toString(), maxQuoteQuantity.toString())

  if (maxBaseQuantity.lte(new BN(0))) {
    throw new Error('size too small')
  }
  if (limitPrice.lte(new BN(0))) {
    throw new Error('invalid price')
  }
  const selfTradeBehavior = 'decrementTake'
  const marketIndex = mangoGroup.getMarketIndex(spotMarket)
  // const vaultIndex = side === 'buy' ? mangoGroup.vaults.length - 1 : marketIndex


  // Specify signers in addition to the wallet
  const signers = []

  const dexSigner = await PublicKey.createProgramAddress(
    [
      spotMarket.publicKey.toBuffer(),
      spotMarket['_decoded'].vaultSignerNonce.toArrayLike(Buffer, 'le', 8),
    ],
    spotMarket.programId
  )

  // Create a Solana account for the open orders account if it's missing
  const openOrdersKeys = []

  const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
  const openOrdersLamports =
    await connection.getMinimumBalanceForRentExemption(
      openOrdersSpace,
      'singleGossip'
    )
  // const accInstr = await createKeyIfNotExists(
  //   wallet,
  //   "",
  //   mangoGroup.dexProgramId,
  //   "seed",
  //   openOrdersSpace,
  //   transaction
  // )
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)

  for (let i = 0; i < marginAccount.openOrders.length; i++) {
    if (
      i === marketIndex &&
      marginAccount.openOrders[marketIndex].equals(zeroKey)
    ) {
      // open orders missing for this market; create a new one now
      const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
      const openOrdersLamports =
        await connection.getMinimumBalanceForRentExemption(
          openOrdersSpace,
          'singleGossip'
        )
      const accInstr = await createKeyIfNotExists(
        wallet,
        "",
        mangoGroup.dexProgramId,
        "seed2",
        openOrdersSpace,
        transaction
      )
      openOrdersKeys.push(accInstr)
    } else {
      openOrdersKeys.push(marginAccount.openOrders[i])
    }
  }

  //Only send a pre-settle instruction if open orders account already exists
  // if (!marginAccount.openOrders[marketIndex].equals(zeroKey)) {
  //   const settleFundsInstr = makeSettleFundsInstruction(
  //     programId,
  //     mangoGroup.publicKey,
  //     wallet.publicKey,
  //     marginAccount.publicKey,
  //     spotMarket.programId,
  //     spotMarket.publicKey,
  //     openOrdersKeys[marketIndex],
  //     mangoGroup.signerKey,
  //     spotMarket['_decoded'].baseVault,
  //     spotMarket['_decoded'].quoteVault,
  //     mangoGroup.vaults[marketIndex],
  //     mangoGroup.vaults[NUM_TOKENS - 1],
  //     dexSigner
  //   )
  //   transaction.add(settleFundsInstr)
  // }

  const keys = [
    { isSigner: false, isWritable: true, pubkey: fundStateAccount },
    { isSigner: true, isWritable: true, pubkey: wallet?.publicKey },
    { isSigner: false, isWritable: true, pubkey: fundPDA },
    { isSigner: false, isWritable: true, pubkey: MANGO_PROGRAM_ID_V2 },

    { isSigner: false, isWritable: true, pubkey: mangoGroup.publicKey },
    //   { isSigner: true, isWritable: false, pubkey: wallet.publicKey },
    { isSigner: false, isWritable: true, pubkey: marginAccount.publicKey },
    { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
    { isSigner: false, isWritable: false, pubkey: spotMarket.programId },
    { isSigner: false, isWritable: true, pubkey: spotMarket.publicKey },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].requestQueue,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].eventQueue,
    },
    { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].bids },
    { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].asks },
    {
      isSigner: false,
      isWritable: true,
      pubkey: mangoGroup.vaults[marketIndex],
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: mangoGroup.vaults[NUM_TOKENS - 1],
    },
    { isSigner: false, isWritable: false, pubkey: mangoGroup.signerKey },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].baseVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].quoteVault,
    },
    { isSigner: false, isWritable: false, pubkey: TOKEN_PROGRAM_ID },
    { isSigner: false, isWritable: false, pubkey: SYSVAR_RENT_PUBKEY },
    { isSigner: false, isWritable: true, pubkey: mangoGroup.srmVault },
    { isSigner: false, isWritable: false, pubkey: dexSigner },
    ...openOrdersKeys.map((pubkey) => ({
      isSigner: false,
      isWritable: true,
      pubkey,
    })),
    ...mangoGroup.oracles.map((pubkey) => ({
      isSigner: false,
      isWritable: false,
      pubkey,
    })),
  ]

  const dataLayout = struct([
    u8('instruction'),
    u32('side'),
    nu64('limitPrice'),
    nu64('maxBaseQuantity'),
    nu64('maxQuoteQuantity'),
    u32('selfTradeBehavior'),
    u32('orderType'),
    nu64('clientId'),
    u16('limit'),
  ])
  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction: 9,
      side: (side == 'buy') ? 0 : 1,
      limitPrice,
      maxBaseQuantity,
      maxQuoteQuantity,
      selfTradeBehavior: 0,
      orderType: 0,
      clientId: 0,
      limit: 65535,
    },
    data
  )

  const placeAndSettleInstruction = new TransactionInstruction({
    keys,
    data,
    programId: programId,
  })
  transaction.add(placeAndSettleInstruction)
}

export async function placeOrder(
  connection,

  marginAcc,
  fundStateAccount,
  fundPDA,
  wallet,

  serumMarket,

  side,
  size,
  clientId,
  transaction,
  investor
) {
  const client = new MangoClient()

  let marginAccount = await client.getMarginAccount(connection, marginAcc, SERUM_PROGRAM_ID_V3)
  let mangoGroup = await client.getMangoGroup(connection, MANGO_GROUP_ACCOUNT)
  console.log("mango group::", mangoGroup)

  console.log("margin acc::", marginAccount)

  let spotMarket = await Market.load(connection, serumMarket, {}, SERUM_PROGRAM_ID_V3)
  console.log("spot market:: ", spotMarket)
  console.log("margin acc:: ", marginAccount)

  let orderType = 'limit'
  let orderbook
  if (side === 'buy') {
    orderbook = await spotMarket.loadAsks(connection)
  }
  else {
    orderbook = await spotMarket.loadBids(connection)
  }
  console.log("orderbook", orderbook)
  let price = calculateMarketPrice(orderbook, size, side)
  console.log("price:: ", price)

  const limitPrice = spotMarket.priceNumberToLots(price)
  const maxBaseQuantity = spotMarket.baseSizeNumberToLots(size)

  const feeTier = getFeeTier(
    0,
    nativeToUi(mangoGroup.nativeSrm || 0, 6)
  )
  const rates = getFeeRates(feeTier)
  const maxQuoteQuantity = new BN(
    maxBaseQuantity
      .mul(limitPrice)
      .mul(spotMarket['_decoded'].quoteLotSize)
      .toNumber() *
    (1 + rates.taker)
  )

  console.log(maxBaseQuantity, maxQuoteQuantity.toString())

  if (maxBaseQuantity.lte(new BN(0))) {
    throw new Error('size too small')
  }
  if (limitPrice.lte(new BN(0))) {
    throw new Error('invalid price')
  }
  const selfTradeBehavior = 'decrementTake'
  const marketIndex = mangoGroup.getMarketIndex(spotMarket)
  // const vaultIndex = side === 'buy' ? mangoGroup.vaults.length - 1 : marketIndex


  // Specify signers in addition to the wallet
  const signers = []

  const dexSigner = await PublicKey.createProgramAddress(
    [
      spotMarket.publicKey.toBuffer(),
      spotMarket['_decoded'].vaultSignerNonce.toArrayLike(Buffer, 'le', 8),
    ],
    spotMarket.programId
  )

  // Create a Solana account for the open orders account if it's missing
  const openOrdersKeys = []

  const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
  const openOrdersLamports =
    await connection.getMinimumBalanceForRentExemption(
      openOrdersSpace,
      'singleGossip'
    )
  // const accInstr = await createKeyIfNotExists(
  //   wallet,
  //   "",
  //   mangoGroup.dexProgramId,
  //   "seed",
  //   openOrdersSpace,
  //   transaction
  // )
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)

  for (let i = 0; i < marginAccount.openOrders.length; i++) {
    if (
      i === marketIndex &&
      marginAccount.openOrders[marketIndex].equals(zeroKey)
    ) {
      // open orders missing for this market; create a new one now
      const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
      const openOrdersLamports =
        await connection.getMinimumBalanceForRentExemption(
          openOrdersSpace,
          'singleGossip'
        )
      const accInstr = await createKeyIfNotExists(
        wallet,
        "",
        mangoGroup.dexProgramId,
        "seed2",
        openOrdersSpace,
        transaction
      )
      openOrdersKeys.push(accInstr)
    } else {
      openOrdersKeys.push(marginAccount.openOrders[i])
    }
  }
  let keys
  if (investor) {
    const invStateAccount = await createKeyIfNotExists(wallet, null, programId, fundPDA.toBase58().substr(0, 32), INVESTOR_DATA.span, transaction)
    keys = [
      { isSigner: false, isWritable: true, pubkey: fundStateAccount },
      { isSigner: false, isWritable: true, pubkey: invStateAccount },

      { isSigner: true, isWritable: true, pubkey: wallet?.publicKey },
      { isSigner: false, isWritable: true, pubkey: fundPDA },
      { isSigner: false, isWritable: true, pubkey: MANGO_PROGRAM_ID_V2 },

      { isSigner: false, isWritable: true, pubkey: mangoGroup.publicKey },
      //   { isSigner: true, isWritable: false, pubkey: wallet.publicKey },
      { isSigner: false, isWritable: true, pubkey: marginAccount.publicKey },
      { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
      { isSigner: false, isWritable: false, pubkey: spotMarket.programId },
      { isSigner: false, isWritable: true, pubkey: spotMarket.publicKey },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].requestQueue,
      },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].eventQueue,
      },
      { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].bids },
      { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].asks },
      {
        isSigner: false,
        isWritable: true,
        pubkey: side == 'sell' ? mangoGroup.vaults[marketIndex] : mangoGroup.vaults[NUM_TOKENS - 1],
      },
      { isSigner: false, isWritable: false, pubkey: mangoGroup.signerKey },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].baseVault,
      },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].quoteVault,
      },
      { isSigner: false, isWritable: false, pubkey: TOKEN_PROGRAM_ID },
      { isSigner: false, isWritable: false, pubkey: SYSVAR_RENT_PUBKEY },
      { isSigner: false, isWritable: true, pubkey: mangoGroup.srmVault },
      //{ isSigner: false, isWritable: false, pubkey: dexSigner },
      ...openOrdersKeys.map((pubkey) => ({
        isSigner: false,
        isWritable: true,
        pubkey,
      })),
      ...mangoGroup.oracles.map((pubkey) => ({
        isSigner: false,
        isWritable: false,
        pubkey,
      })),
    ]
  }
  else {
    keys = [
      { isSigner: false, isWritable: true, pubkey: fundStateAccount },
      { isSigner: true, isWritable: true, pubkey: wallet?.publicKey },
      { isSigner: false, isWritable: true, pubkey: fundPDA },
      { isSigner: false, isWritable: true, pubkey: MANGO_PROGRAM_ID_V2 },

      { isSigner: false, isWritable: true, pubkey: mangoGroup.publicKey },
      //   { isSigner: true, isWritable: false, pubkey: wallet.publicKey },
      { isSigner: false, isWritable: true, pubkey: marginAccount.publicKey },
      { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
      { isSigner: false, isWritable: false, pubkey: spotMarket.programId },
      { isSigner: false, isWritable: true, pubkey: spotMarket.publicKey },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].requestQueue,
      },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].eventQueue,
      },
      { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].bids },
      { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].asks },
      {
        isSigner: false,
        isWritable: true,
        pubkey: side == 'sell' ? mangoGroup.vaults[marketIndex] : mangoGroup.vaults[NUM_TOKENS - 1],
      },
      { isSigner: false, isWritable: false, pubkey: mangoGroup.signerKey },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].baseVault,
      },
      {
        isSigner: false,
        isWritable: true,
        pubkey: spotMarket['_decoded'].quoteVault,
      },
      { isSigner: false, isWritable: false, pubkey: TOKEN_PROGRAM_ID },
      { isSigner: false, isWritable: false, pubkey: SYSVAR_RENT_PUBKEY },
      { isSigner: false, isWritable: true, pubkey: mangoGroup.srmVault },
      //{ isSigner: false, isWritable: false, pubkey: dexSigner },
      ...openOrdersKeys.map((pubkey) => ({
        isSigner: false,
        isWritable: true,
        pubkey,
      })),
      ...mangoGroup.oracles.map((pubkey) => ({
        isSigner: false,
        isWritable: false,
        pubkey,
      })),
    ]
  }

  const dataLayout = struct([
      u8('instruction'),
      u8('side'),
      nu64('price'),
      nu64('quote_size'),
      nu64('base_size')
  ])
  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
      {
        instruction: investor ? 13 : 9,
        side: (side == 'buy') ? 0 : 1,
        price: limitPrice,
        quote_size: maxBaseQuantity,
        base_size: maxQuoteQuantity
      },
      data
  )

  const placeAndSettleInstruction = new TransactionInstruction({
    keys,
    data,
    programId: programId,
  })
  transaction.add(placeAndSettleInstruction)

  const baseTokenIndex = marketIndex;
  const quoteTokenIndex = NUM_TOKENS - 1;
  const tokenIndex = side === 'buy' ? baseTokenIndex : quoteTokenIndex;
  const quantity = marginAccount.getUiBorrow(mangoGroup, tokenIndex);
  const nativeQuantity = uiToNative(quantity, mangoGroup.mintDecimals[tokenIndex]);

  const settle_keys = [
    { isSigner: false, isWritable: true, pubkey: fundStateAccount },
    { isSigner: true, isWritable: true, pubkey: wallet?.publicKey },
    { isSigner: false, isWritable: true, pubkey: fundPDA },
    { isSigner: false, isWritable: true, pubkey: MANGO_PROGRAM_ID_V2 },

    { isSigner: false, isWritable: true, pubkey: mangoGroup.publicKey },
    { isSigner: false, isWritable: true, pubkey: marginAccount.publicKey },
    { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
    { isSigner: false, isWritable: false, pubkey: spotMarket.programId },
    { isSigner: false, isWritable: true, pubkey: spotMarket.publicKey },
    { isSigner: false, isWritable: true, pubkey: openOrdersKeys[marketIndex] },
    { isSigner: false, isWritable: false, pubkey: mangoGroup.signerKey },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].baseVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].quoteVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: mangoGroup.vaults[marketIndex],
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: mangoGroup.vaults[NUM_MARKETS],
    },
    { isSigner: false, isWritable: false, pubkey: dexSigner },
    { isSigner: false, isWritable: false, pubkey: TOKEN_PROGRAM_ID },
  ];

  const datLayout = struct([u8('instruction'), nu64('settle_amount'), nu64('token_index')])

  console.log("settle_amount", nativeQuantity.toNumber())
  const fundBaseTokenAccount = await findAssociatedTokenAddress(fundPDA, new PublicKey(MANGO_TOKENS['USDC'].mintAddress));


      const dat = Buffer.alloc(datLayout.span)
      datLayout.encode(
        {
          instruction: 10,
          settle_amount: nativeQuantity,
          token_index: tokenIndex
        },
        dat
      )
      const SettleInstruction = new TransactionInstruction({
        keys: settle_keys,
        data: dat,
        programId: programId,
      })
      console.log("settle intr")
  transaction.add(SettleInstruction)
  
  const withdraw_keys = [
    { isSigner: false, isWritable: true, pubkey: fundStateAccount },
    { isSigner: false, isWritable: true, pubkey: priceStateAccount },


    { isSigner: true, isWritable: true, pubkey: wallet?.publicKey },
    { isSigner: false, isWritable: true, pubkey: fundPDA },
    { isSigner: false, isWritable: true, pubkey: MANGO_PROGRAM_ID_V2 },

  { isSigner: false, isWritable: true, pubkey: mangoGroup.publicKey },
//   { isSigner: true, isWritable: false, pubkey: wallet.publicKey },
  { isSigner: false, isWritable: true, pubkey: marginAccount.publicKey },

  { isSigner: false, isWritable: true, pubkey: fundBaseTokenAccount },
  { isSigner: false, isWritable: true, pubkey: mangoGroup.vaults[NUM_MARKETS] },
  { isSigner: false, isWritable: false, pubkey: mangoGroup.signerKey },

  { isSigner: false, isWritable: false, pubkey: TOKEN_PROGRAM_ID },
  { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
 
  ...openOrdersKeys.map((pubkey) => ({
    isSigner: false,
    isWritable: true,
    pubkey,
  })),
  ...mangoGroup.oracles.map((pubkey) => ({
    isSigner: false,
    isWritable: false,
    pubkey,
  })),
]

  const dataL = struct([u8('instruction'), nu64('quantity')])
      const data2 = Buffer.alloc(dataL.span)
      dataL.encode(
        {
          instruction: 11,
          quantity: 20 * ( 10 ** MANGO_TOKENS['USDC'].decimals)
        },
        data2
      )
      const instr = new TransactionInstruction({
        keys: withdraw_keys,
        data: data2,
        programId: programId,
      })
      transaction.add(instr)
}

export async function placeOrder2(
  connection,

  marginAcc,
  fundStateAccount,
  fundPDA,
  wallet,

  serumMarket,

  side,
  size,
  clientId,
  transaction,
  token_i
) {
  const client = new MangoClient()

  let marginAccount = await client.getMarginAccount(connection, marginAcc, SERUM_PROGRAM_ID_V3)
  let mangoGroup = await client.getMangoGroup(connection, MANGO_GROUP_ACCOUNT)
  console.log("mango group::", mangoGroup)

  console.log("margin acc::", marginAccount)

  let spotMarket = await Market.load(connection, serumMarket, {}, SERUM_PROGRAM_ID_V3)
  console.log("spot market:: ", spotMarket)
  console.log("margin acc:: ", marginAccount)

  let orderType = 'limit'
  let orderbook
  if (side === 'buy') {
    orderbook = await spotMarket.loadAsks(connection)
  }
  else {
    orderbook = await spotMarket.loadBids(connection)
  }
  console.log("orderbook", orderbook)
  let price = calculateMarketPrice(orderbook, size, side)
  console.log("price:: ", price)

  const limitPrice = spotMarket.priceNumberToLots(price)
  const maxBaseQuantity = spotMarket.baseSizeNumberToLots(size)

  const feeTier = getFeeTier(
    0,
    nativeToUi(mangoGroup.nativeSrm || 0, 6)
  )
  const rates = getFeeRates(feeTier)
  const maxQuoteQuantity = new BN(
    maxBaseQuantity
      .mul(limitPrice)
      .mul(spotMarket['_decoded'].quoteLotSize)
      .toNumber() *
    (1 + rates.taker)
  )

  console.log(maxBaseQuantity, maxQuoteQuantity.toString())

  if (maxBaseQuantity.lte(new BN(0))) {
    throw new Error('size too small')
  }
  if (limitPrice.lte(new BN(0))) {
    throw new Error('invalid price')
  }
  const selfTradeBehavior = 'decrementTake'
  const marketIndex = mangoGroup.getMarketIndex(spotMarket)
  // const vaultIndex = side === 'buy' ? mangoGroup.vaults.length - 1 : marketIndex


  // Specify signers in addition to the wallet
  const signers = []

  const dexSigner = await PublicKey.createProgramAddress(
    [
      spotMarket.publicKey.toBuffer(),
      spotMarket['_decoded'].vaultSignerNonce.toArrayLike(Buffer, 'le', 8),
    ],
    spotMarket.programId
  )

  // Create a Solana account for the open orders account if it's missing
  const openOrdersKeys = []

  const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
  const openOrdersLamports =
    await connection.getMinimumBalanceForRentExemption(
      openOrdersSpace,
      'singleGossip'
    )
  // const accInstr = await createKeyIfNotExists(
  //   wallet,
  //   "",
  //   mangoGroup.dexProgramId,
  //   "seed",
  //   openOrdersSpace,
  //   transaction
  // )
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)
  // openOrdersKeys.push(accInstr)

  for (let i = 0; i < marginAccount.openOrders.length; i++) {
    if (
      i === marketIndex &&
      marginAccount.openOrders[marketIndex].equals(zeroKey)
    ) {
      // open orders missing for this market; create a new one now
      const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
      const openOrdersLamports =
        await connection.getMinimumBalanceForRentExemption(
          openOrdersSpace,
          'singleGossip'
        )
      const accInstr = await createKeyIfNotExists(
        wallet,
        "",
        mangoGroup.dexProgramId,
        "seed",
        openOrdersSpace,
        transaction
      )
      openOrdersKeys.push(accInstr)
    } else {
      openOrdersKeys.push(marginAccount.openOrders[i])
    }
  }

  const invStateAccount = await createKeyIfNotExists(wallet, null, programId, fundPDA.toBase58().substr(0, 32), INVESTOR_DATA.span, transaction)
  let keys = [
    { isSigner: false, isWritable: true, pubkey: fundStateAccount },
    { isSigner: false, isWritable: true, pubkey: invStateAccount },

    { isSigner: true, isWritable: true, pubkey: wallet?.publicKey },
    { isSigner: false, isWritable: true, pubkey: fundPDA },
    { isSigner: false, isWritable: true, pubkey: MANGO_PROGRAM_ID_V2 },

    { isSigner: false, isWritable: true, pubkey: mangoGroup.publicKey },
    //   { isSigner: true, isWritable: false, pubkey: wallet.publicKey },
    { isSigner: false, isWritable: true, pubkey: marginAccount.publicKey },
    { isSigner: false, isWritable: false, pubkey: SYSVAR_CLOCK_PUBKEY },
    { isSigner: false, isWritable: false, pubkey: spotMarket.programId },
    { isSigner: false, isWritable: true, pubkey: spotMarket.publicKey },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].requestQueue,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].eventQueue,
    },
    { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].bids },
    { isSigner: false, isWritable: true, pubkey: spotMarket['_decoded'].asks },
    {
      isSigner: false,
      isWritable: true,
      pubkey: side == 'sell' ? mangoGroup.vaults[marketIndex] : mangoGroup.vaults[NUM_TOKENS - 1],
    },
    { isSigner: false, isWritable: false, pubkey: mangoGroup.signerKey },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].baseVault,
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: spotMarket['_decoded'].quoteVault,
    },
    { isSigner: false, isWritable: false, pubkey: dexSigner },
    { isSigner: false, isWritable: false, pubkey: TOKEN_PROGRAM_ID },
    { isSigner: false, isWritable: false, pubkey: SYSVAR_RENT_PUBKEY },
    {
      isSigner: false,
      isWritable: true,
      pubkey: mangoGroup.vaults[marketIndex],
    },
    {
      isSigner: false,
      isWritable: true,
      pubkey: mangoGroup.vaults[NUM_MARKETS],
    },
    { isSigner: false, isWritable: true, pubkey: mangoGroup.srmVault },
    ...openOrdersKeys.map((pubkey) => ({
      isSigner: false,
      isWritable: true,
      pubkey,
    })),
    ...mangoGroup.oracles.map((pubkey) => ({
      isSigner: false,
      isWritable: false,
      pubkey,
    })),
  ]

  const dataLayout = struct([
    u8('instruction'),
    u32('side'),
    nu64('limitPrice'),
    nu64('maxBaseQuantity'),
    nu64('maxQuoteQuantity'),
    u32('selfTradeBehavior'),
    u32('orderType'),
    nu64('clientId'),
    u16('limit'),
    nu64('token_index')
  ])
  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
      {
        instruction: 15,
        side: (side == 'buy') ? 0 : 1,
        limitPrice,
        //maxBaseQuantity,
        //maxQuoteQuantity,
        //selfTradeBehavior: 0,
        orderType: 0,
        //clientId: 0,
        limit: 65535,
        token_index: token_i
      },
      data
  )

  const placeAndSettleInstruction = new TransactionInstruction({
    keys,
    data,
    programId: programId,
  })
  transaction.add(placeAndSettleInstruction)
}