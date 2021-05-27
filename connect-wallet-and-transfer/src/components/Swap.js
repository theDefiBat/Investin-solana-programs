import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js'
import { nu64, struct, u8 } from 'buffer-layout'
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState'
import { connection, TOKEN_PROGRAM_ID } from '../utils/constants'
import { pools } from '../utils/pools'
import { TokenAmount } from '../utils/safe-math'
import { NATIVE_SOL, TOKENS } from '../utils/tokens'
import { createTokenAccountIfNotExist, findAssociatedTokenAddress, sendNewTransaction } from '../utils/web3'

export const Swap = () => {

  const swapInstruction = (
    programId,
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
    const dataLayout = struct([u8('instruction'), nu64('amountIn'), nu64('minAmountOut')])

    const keys = [
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
      { pubkey: userOwner, isSigner: true, isWritable: true }
    ]

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 9,
        amountIn,
        minAmountOut
      },
      data
    )

    return new TransactionInstruction({
      keys,
      programId,
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

    transaction.add(
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
        owner,
        Math.floor(amountIn.toWei().toNumber()),
        Math.floor(amountOut.toWei().toNumber())
      )
    )

    return await sendNewTransaction(connection, wallet, transaction, signers)
  }

  const walletProvider = GlobalState.useState(s => s.walletProvider);


  const [amountIn, setAmountIn] = useState(0);
  const [fundPDA, setFundPDA] = useState('');
  const [selectedFirstToken, setSelectedFirstToken] = useState('RAY-USDT');

  const handleBuy = async () => {
    const poolInfo = pools.find(p => p.name === selectedFirstToken);
    const fromCoinMint = poolInfo.coin.mintAddress;
    const toCoinMint = poolInfo.pc.mintAddress;
    console.log(`fundPDA :::: `, fundPDA)
    const fromTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(fromCoinMint));
    const toTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(toCoinMint));

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
    const toCoinMint = poolInfo.coin.mintAddress;
    const fromCoinMint = poolInfo.pc.mintAddress;
    console.log(`fundPDA :::: `, fundPDA)
    const fromTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(fromCoinMint));
    const toTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(toCoinMint));

    console.log(`poolInfo ::: `, poolInfo)
    console.log(`fromCoinMint ::: `, fromCoinMint)
    console.log(`toCoinMint ::: `, toCoinMint)
    console.log(`fromTokenAccount ::: `, fromTokenAccount)
    console.log(`toTokenAccount ::: `, fromTokenAccount)

    const txId = await swapTokens(connection, walletProvider, poolInfo, fromCoinMint, toCoinMint, fromTokenAccount, toTokenAccount, amountIn, 1, "sell");

    console.log(`txId :::: `, txId)
  }

  const handleFirstTokenSelect = (event) => {
    setSelectedFirstToken(`${event.target.value}-USDT`);
    console.log(`${event.target.value}-USDT :::: `, `${event.target.value}-USDT`)
  }

  return (
    <div>
      Swap ::: RAY TO USDT {' '}
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
      fundPDA : {' '}<input type="text" value={fundPDA} onChange={e => setFundPDA(e.target.value)} />
      <br />
      <button onClick={handleBuy} >Buy</button>
      <br />
      <button onClick={handleSell} >Sell</button>
    </div>
  )
}


