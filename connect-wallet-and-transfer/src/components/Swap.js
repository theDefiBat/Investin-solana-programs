import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js'
import { nu64, struct, u8 } from 'buffer-layout'
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState'
import { connection, programId, TOKEN_PROGRAM_ID, FUND_ACCOUNT_KEY, LIQUIDITY_POOL_PROGRAM_ID_V4 } from '../utils/constants'
import { pools } from '../utils/pools'
import { TokenAmount } from '../utils/safe-math'
import { NATIVE_SOL, TOKENS } from '../utils/tokens'
import { createTokenAccountIfNotExist, findAssociatedTokenAddress, sendNewTransaction, signAndSendTransaction } from '../utils/web3'

export const Swap = () => {

  const swapInstruction = async (
    poolProgramId,
    // tokenProgramId,
    // amm
    ammId,
    ammAuthority,
    ammOpenOrders,
    ammTargetOrders,
    poolCoinTokenAccount,
    poolPcTokenAccount,
    // serum
    serumProgramId,
    serumMarket,
    serumBids,
    serumAsks,
    serumEventQueue,
    serumCoinVaultAccount,
    serumPcVaultAccount,
    serumVaultSigner,
    // user
    userSourceTokenAccount,
    userDestTokenAccount,
    userOwner,

    amountIn,
    minAmountOut
  ) => {

    const key = walletProvider?.publicKey;  
    if (!key ) {
      alert("connect wallet")
      return;
    }

    const fundStateAcc = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    const dataLayout = struct([u8('instruction1'), u8('instruction'), nu64('amountIn'), nu64('minAmountOut')])

    const keys = [
      { pubkey: fundStateAcc, isSigner: false, isWritable: true },
      { pubkey: poolProgramId, isSigner: false, isWritable: true },

      // spl token
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
      // amm
      { pubkey: ammId, isSigner: false, isWritable: true },
      { pubkey: ammAuthority, isSigner: false, isWritable: true },
      { pubkey: ammOpenOrders, isSigner: false, isWritable: true },
      { pubkey: ammTargetOrders, isSigner: false, isWritable: true },
      { pubkey: poolCoinTokenAccount, isSigner: false, isWritable: true },
      { pubkey: poolPcTokenAccount, isSigner: false, isWritable: true },
      // serum
      { pubkey: serumProgramId, isSigner: false, isWritable: true },
      { pubkey: serumMarket, isSigner: false, isWritable: true },
      { pubkey: serumBids, isSigner: false, isWritable: true },
      { pubkey: serumAsks, isSigner: false, isWritable: true },
      { pubkey: serumEventQueue, isSigner: false, isWritable: true },
      { pubkey: serumCoinVaultAccount, isSigner: false, isWritable: true },
      { pubkey: serumPcVaultAccount, isSigner: false, isWritable: true },
      { pubkey: serumVaultSigner, isSigner: false, isWritable: true },

      { pubkey: userSourceTokenAccount, isSigner: false, isWritable: true },
      { pubkey: userDestTokenAccount, isSigner: false, isWritable: true },
      { pubkey: userOwner, isSigner: false, isWritable: true }
    ]

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction1: 4, 
        instruction: 9,
        amountIn,
        minAmountOut
      },
      data
    )

    console.log("prog_id:: ", programId)

    return new TransactionInstruction({
      keys,
      programId: programId,
      data
    })
  }

  const swapTokens = async (
    connection,
    wallet,
    poolInfo,
    fromCoinMint,
    toCoinMint,
    fromTokenAccount,
    toTokenAccount,
    amount,
    slippage,
    tradeSide
  ) => {
    const transaction = new Transaction()
    const signers = []

    const owner = wallet.publicKey
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    // const { amountIn, amountOut } = getSwapOutAmount(poolInfo, fromCoinMint, toCoinMint, amount, slippage)
    let amountIn = new TokenAmount(amount, poolInfo.coin.decimals, false)
    let amountOut = new TokenAmount(1, poolInfo.coin.decimals)
    if (tradeSide === "sell") {
      amountIn = new TokenAmount(amount, poolInfo.coin.decimals, false)
      amountOut = new TokenAmount(1, poolInfo.coin.decimals)
    } else {
      amountIn = new TokenAmount(amount, poolInfo.pc.decimals, false)
      amountOut = new TokenAmount(1, poolInfo.pc.decimals)
    }

    let fromMint = fromCoinMint
    let toMint = toCoinMint

    if (fromMint === NATIVE_SOL.mintAddress) {
      fromMint = TOKENS.WSOL.mintAddress
    }
    if (toMint === NATIVE_SOL.mintAddress) {
      toMint = TOKENS.WSOL.mintAddress
    }

    const newFromTokenAccount = fromTokenAccount
    const newToTokenAccount = toTokenAccount

    let instruction = await
      swapInstruction(
        new PublicKey(poolInfo.programId),
        new PublicKey(poolInfo.ammId),
        new PublicKey(poolInfo.ammAuthority),
        new PublicKey(poolInfo.ammOpenOrders),
        new PublicKey(poolInfo.ammTargetOrders),
        new PublicKey(poolInfo.poolCoinTokenAccount),
        new PublicKey(poolInfo.poolPcTokenAccount),
        new PublicKey(poolInfo.serumProgramId),
        new PublicKey(poolInfo.serumMarket),
        new PublicKey(poolInfo.serumBids),
        new PublicKey(poolInfo.serumAsks),
        new PublicKey(poolInfo.serumEventQueue),
        new PublicKey(poolInfo.serumCoinVaultAccount),
        new PublicKey(poolInfo.serumPcVaultAccount),
        new PublicKey(poolInfo.serumVaultSigner),
        newFromTokenAccount,
        newToTokenAccount,
        fundPDA[0],
        Math.floor(amountIn.toWei().toNumber()),
        Math.floor(amountOut.toWei().toNumber())
      )
    transaction.add(instruction)
    transaction.feePayer = owner;
    console.log("trnsaction:: ", transaction)
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    //return await sendNewTransaction(connection, wallet, transaction, signers)
  }

  const walletProvider = GlobalState.useState(s => s.walletProvider);


  const [amountIn, setAmountIn] = useState(0);
  //const [selectedFirstToken, setSelectedFirstToken] = useState('RAY-USDT');
  const [selectedFirstToken, setSelectedFirstToken] = useState('');

  const handleBuy = async () => {
    const poolInfo = pools.find(p => p.name === selectedFirstToken);
    const fromCoinMint = poolInfo.pc.mintAddress;
    const toCoinMint = poolInfo.coin.mintAddress;
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    console.log(`fundPDA :::: `, fundPDA)
    const fromTokenAccount = await findAssociatedTokenAddress( fundPDA[0], new PublicKey(fromCoinMint));
    const toTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(toCoinMint));

    console.log(`poolInfo ::: `, poolInfo)
    console.log(`fromCoinMint ::: `, fromCoinMint)
    console.log(`toCoinMint ::: `, toCoinMint)
    console.log(`fromTokenAccount ::: `, fromTokenAccount)
    console.log(`toTokenAccount ::: `, fromTokenAccount)

    const txId = await swapTokens(connection, walletProvider, poolInfo, fromCoinMint, toCoinMint, fromTokenAccount, toTokenAccount, amountIn, 1, "buy");

    console.log(`txId :::: `, txId)
  }

  const handleSell = async () => {
    const poolInfo = pools.find(p => p.name === selectedFirstToken);

    console.log("pool info:: ", poolInfo)
    const toCoinMint = poolInfo.pc.mintAddress;
    const fromCoinMint = poolInfo.coin.mintAddress;
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    console.log(`fundPDA :::: `, fundPDA)
    const fromTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(fromCoinMint));
    const toTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(toCoinMint));

    console.log(`poolInfo ::: `, poolInfo)
    console.log(`fromCoinMint ::: `, fromCoinMint)
    console.log(`toCoinMint ::: `, toCoinMint)
    console.log(`fromTokenAccount ::: `, fromTokenAccount)
    console.log(`toTokenAccount ::: `, fromTokenAccount)

    const txId = await swapTokens(connection, walletProvider, poolInfo, fromCoinMint, toCoinMint, fromTokenAccount, toTokenAccount, amountIn, 1, "sell");

    console.log(`txId :::: `, txId)
  }

  const handleFirstTokenSelect = (event) => {
    setSelectedFirstToken(`${event.target.value}-USDC`);
    console.log(`${event.target.value}-USDC :::: `, `${event.target.value}-USDC`)
  }

  return (
    <div className="form-div">
      <h4>Swap</h4>
      Swap ::: {selectedFirstToken}  
      <br />

      <label htmlFor="tokens">From Token:</label>

      <select name="tokens" onChange={handleFirstTokenSelect}>
        {
          pools.map((pool) => {
            return (<option key={pool.coin.name} value={pool.coin.symbol}>{pool.coin.name}</option>)
          })
        }
      </select>
      <br />

      amount : {' '}<input type="number" value={amountIn} onChange={(e) => setAmountIn(e.target.value)} />
      <br />
    
      <button margin-right="10px" onClick={handleBuy} >Buy</button>
      <button onClick={handleSell} >Sell</button>
    </div>
  )
}


