
import { Account, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js'
// @ts-ignore
import { u8, nu64, struct } from 'buffer-layout'
// eslint-disable-next-line
import { NATIVE_SOL, TOKENS, getTokenByMintAddress } from './tokens'
import {
  createProgramAccountIfNotExist,
  createTokenAccountIfNotExist,
  mergeTransactions,
  sendTransaction
} from './web3'
import { TokenAmount } from './safe-math'
// eslint-disable-next-line

import { closeAccount } from '@project-serum/serum/lib/token-instructions'
import { TOKEN_PROGRAM_ID, MEMO_PROGRAM_ID, SERUM_PROGRAM_ID_V3 } from './ids'
import logger from './logger'

import { ACCOUNT_LAYOUT, MINT_LAYOUT } from './layouts'
import { AMM_INFO_LAYOUT, AMM_INFO_LAYOUT_V3, AMM_INFO_LAYOUT_V4 } from './liquidity'
import { LIQUIDITY_POOLS, getAddressForWhat } from './pools'
import { commitment, getMultipleAccounts } from './web3'

import { OpenOrders } from '@project-serum/serum'
import { cloneDeep } from 'lodash'

export async function loadInfo (conn: Connection) {
      const liquidityPools = {} as any
      const publicKeys = [] as any
      console.log("loading liquidity pools")
      LIQUIDITY_POOLS.forEach((pool) => {
        const { poolCoinTokenAccount, poolPcTokenAccount, ammOpenOrders, ammId, coin, pc, lp } = pool
        publicKeys.push(
          new PublicKey(poolCoinTokenAccount),
          new PublicKey(poolPcTokenAccount),
          new PublicKey(ammOpenOrders),
          new PublicKey(ammId),
          new PublicKey(lp.mintAddress)
        )

        const poolInfo = cloneDeep(pool)

        poolInfo.coin.balance = new TokenAmount(0, coin.decimals)
        poolInfo.pc.balance = new TokenAmount(0, pc.decimals)

        liquidityPools[lp.mintAddress] = poolInfo
      })
      console.log("reading accounts infos ")
      const multipleInfo = await getMultipleAccounts(conn, publicKeys, commitment)

      multipleInfo.forEach((info) => {
        if (info) {
          const address = info.publicKey.toBase58()
          const data = Buffer.from(info.account.data)

          const { key, lpMintAddress, version } = getAddressForWhat(address)

          if (key && lpMintAddress) {
            const poolInfo = liquidityPools[lpMintAddress]

            switch (key) {
              case 'poolCoinTokenAccount': {
                const parsed = ACCOUNT_LAYOUT.decode(data)
                // quick fix: Number can only safely store up to 53 bits
                poolInfo.coin.balance.wei = poolInfo.coin.balance.wei.plus(parsed.amount.toString())
                break
              }
              case 'poolPcTokenAccount': {
                const parsed = ACCOUNT_LAYOUT.decode(data)

                poolInfo.pc.balance.wei = poolInfo.pc.balance.wei.plus(parsed.amount.toNumber())

                break
              }
              case 'ammOpenOrders': {
                const OPEN_ORDERS_LAYOUT = OpenOrders.getLayout(new PublicKey(poolInfo.serumProgramId))
                const parsed = OPEN_ORDERS_LAYOUT.decode(data)

                const { baseTokenTotal, quoteTokenTotal } = parsed
                poolInfo.coin.balance.wei = poolInfo.coin.balance.wei.plus(baseTokenTotal.toNumber())
                poolInfo.pc.balance.wei = poolInfo.pc.balance.wei.plus(quoteTokenTotal.toNumber())

                break
              }
              case 'ammId': {
                let parsed
                if (version === 2) {
                  parsed = AMM_INFO_LAYOUT.decode(data)
                } else if (version === 3) {
                  parsed = AMM_INFO_LAYOUT_V3.decode(data)
                } else {
                  parsed = AMM_INFO_LAYOUT_V4.decode(data)

                  const { swapFeeNumerator, swapFeeDenominator } = parsed
                  poolInfo.fees = {
                    swapFeeNumerator: swapFeeNumerator.toNumber(),
                    swapFeeDenominator: swapFeeDenominator.toNumber()
                  }
                }

                const { needTakePnlCoin, needTakePnlPc } = parsed
                poolInfo.coin.balance.wei = poolInfo.coin.balance.wei.minus(needTakePnlCoin.toNumber())
                poolInfo.pc.balance.wei = poolInfo.pc.balance.wei.minus(needTakePnlPc.toNumber())

                break
              }
              // getLpSupply
              case 'lpMintAddress': {
                const parsed = MINT_LAYOUT.decode(data)

                poolInfo.lp.totalSupply = new TokenAmount(parsed.supply.toNumber(), poolInfo.lp.decimals)

                break
              }
            }
          }
        }
      })
      console.log('Liquidity pool infomations updated')
      return liquidityPools
  }

export function getSwapOutAmount(
  poolInfo: any,
  fromCoinMint: string,
  toCoinMint: string,
  amount: string,
  slippage: number
) {
  let { coin, pc, fees } = poolInfo
  console.log(poolInfo)
  const { swapFeeNumerator, swapFeeDenominator } = fees
  
  if (fromCoinMint === coin.mintAddress && toCoinMint === pc.mintAddress) {
    // coin2pc
    const fromAmount = new TokenAmount(amount, coin.decimals, false)
    const denominator = coin.balance.wei.plus(fromAmount.wei)
    const amountOut = pc.balance.wei.multipliedBy(fromAmount.wei).dividedBy(denominator)
    const amountOutWithFee = amountOut.dividedBy(swapFeeDenominator).multipliedBy(swapFeeDenominator - swapFeeNumerator)
    const amountOutWithSlippage = amountOutWithFee.dividedBy(100).multipliedBy(100 - slippage)
    return { amountIn: fromAmount, amountOut: new TokenAmount(amountOutWithSlippage, pc.decimals) }
  } else {
    // pc2coin
    const fromAmount = new TokenAmount(amount, pc.decimals, false)
    const denominator = pc.balance.wei.plus(fromAmount.wei)
    const amountOut = coin.balance.wei.multipliedBy(fromAmount.wei).dividedBy(denominator)
    const amountOutWithFee = amountOut.dividedBy(swapFeeDenominator).multipliedBy(swapFeeDenominator - swapFeeNumerator)
    const amountOutWithSlippage = amountOutWithFee.dividedBy(100).multipliedBy(100 - slippage)
    return { amountIn: fromAmount, amountOut: new TokenAmount(amountOutWithSlippage, coin.decimals) }
  }
}


export async function createTokenAccount(connection: Connection, wallet: any, mintAddress: string) {
  const transaction = new Transaction()
  const signers: Account[] = []

  const owner = wallet.publicKey

  await createTokenAccountIfNotExist(connection, '', owner, mintAddress, null, transaction, signers)

  return await sendTransaction(connection, wallet, transaction, signers)
}

// export async function wrap(
//   axios: any,
//   connection: Connection,
//   wallet: any,
//   fromCoinMint: string,
//   toCoinMint: string,
//   fromTokenAccount: string,
//   toTokenAccount: string,
//   amount: string
// ) {
//   const transaction = new Transaction()
//   const signers: Account[] = []

//   const owner = wallet.publicKey

//   const fromCoin = getTokenByMintAddress(fromCoinMint)
//   const amountOut = new TokenAmount(amount, fromCoin?.decimals, false)

//   const newFromTokenAccount = await createTokenAccountIfNotExist(
//     connection,
//     fromTokenAccount,
//     owner,
//     fromCoinMint,
//     null,
//     transaction,
//     signers
//   )
//   const newToTokenAccount = await createTokenAccountIfNotExist(
//     connection,
//     toTokenAccount,
//     owner,
//     toCoinMint,
//     null,
//     transaction,
//     signers
//   )

//   const solletRes = await axios.post('https://swap.sollet.io/api/swap_to', {
//     address: newToTokenAccount.toString(),
//     blockchain: 'sol',
//     coin: toCoinMint,
//     size: 1,
//     wusdtToUsdt: true
//   })
//   const { address, maxSize } = solletRes.result

//   if (!address) {
//     throw new Error('Unwrap not available now')
//   }

//   if (parseFloat(amount) > maxSize) {
//     throw new Error(`Max allow ${maxSize}`)
//   }

//   transaction.add(transfer(newFromTokenAccount, new PublicKey(address), owner, amountOut.toWei().toNumber()))
//   transaction.add(memoInstruction(newToTokenAccount.toString()))

//   return await sendTransaction(connection, wallet, transaction, signers)
// }

export async function swap(
  connection: Connection,
  wallet: any,
  poolInfo: any,
  fromCoinMint: string,
  toCoinMint: string,
  fromTokenAccount: string,
  toTokenAccount: string,
  amount: string,
  slippage: number
) {
  const transaction = new Transaction()
  const signers: Account[] = []

  const owner = wallet.publicKey
  console.log(poolInfo)
  //const { amountIn, amountOut } = getSwapOutAmount(poolInfo, fromCoinMint, toCoinMint, amount, slippage)


  // let fromMint = fromCoinMint
  // let toMint = toCoinMint

  // if (fromMint === NATIVE_SOL.mintAddress) {
  //   fromMint = TOKENS.WSOL.mintAddress
  // }
  // if (toMint === NATIVE_SOL.mintAddress) {
  //   toMint = TOKENS.WSOL.mintAddress
  // }

  // let wrappedSolAccount: PublicKey | null = null
  // let wrappedSolAccount2: PublicKey | null = null

  // if (fromCoinMint === NATIVE_SOL.mintAddress) {
  //   wrappedSolAccount = await createTokenAccountIfNotExist(
  //     connection,
  //     wrappedSolAccount,
  //     owner,
  //     TOKENS.WSOL.mintAddress,
  //     amountIn.wei.toNumber() + 1e7,
  //     transaction,
  //     signers
  //   )
  // }

  // if (toCoinMint === NATIVE_SOL.mintAddress) {
  //   wrappedSolAccount2 = await createTokenAccountIfNotExist(
  //     connection,
  //     wrappedSolAccount2,
  //     owner,
  //     TOKENS.WSOL.mintAddress,
  //     1e7,
  //     transaction,
  //     signers
  //   )
  // }

  // const newFromTokenAccount = await createTokenAccountIfNotExist(
  //   connection,
  //   fromTokenAccount,
  //   owner,
  //   fromMint,
  //   null,
  //   transaction,
  //   signers
  // )
  // const newToTokenAccount = await createTokenAccountIfNotExist(
  //   connection,
  //   toTokenAccount,
  //   owner,
  //   toMint,
  //   null,
  //   transaction,
  //   signers
  // )

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
      new PublicKey(fromTokenAccount),
      new PublicKey(toTokenAccount),
      //wrappedSolAccount ?? newFromTokenAccount,
      //wrappedSolAccount2 ?? newToTokenAccount,
      owner,
      Math.floor((+amount)*1000000),
      Math.floor(100)
    )
  )
    
  // if (wrappedSolAccount) {
  //   transaction.add(
  //     closeAccount({
  //       source: wrappedSolAccount,
  //       destination: owner,
  //       owner
  //     })
  //   )
  // }
  // if (wrappedSolAccount2) {
  //   transaction.add(
  //     closeAccount({
  //       source: wrappedSolAccount2,
  //       destination: owner,
  //       owner
  //     })
  //   )
  // }

  return await connection.sendTransaction(transaction, [wallet])

  //return await sendTransaction(connection, wallet, transaction, signers)
}


export function swapInstruction(
  programId: PublicKey,
  // tokenProgramId: PublicKey,
  // amm
  ammId: PublicKey,
  ammAuthority: PublicKey,
  ammOpenOrders: PublicKey,
  ammTargetOrders: PublicKey,
  poolCoinTokenAccount: PublicKey,
  poolPcTokenAccount: PublicKey,
  // serum
  serumProgramId: PublicKey,
  serumMarket: PublicKey,
  serumBids: PublicKey,
  serumAsks: PublicKey,
  serumEventQueue: PublicKey,
  serumCoinVaultAccount: PublicKey,
  serumPcVaultAccount: PublicKey,
  serumVaultSigner: PublicKey,
  // user
  userSourceTokenAccount: PublicKey,
  userDestTokenAccount: PublicKey,
  userOwner: PublicKey,

  amountIn: number,
  minAmountOut: number
): TransactionInstruction {
  const dataLayout = struct([u8('instruction1'), u8('instruction'), nu64('amountIn'), nu64('minAmountOut')])

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
      instruction1: 3,
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

export function transfer(source: PublicKey, destination: PublicKey, owner: PublicKey, amount: number) {
  const dataLayout = struct([u8('instruction'), nu64('amount')])

  const keys = [
    { pubkey: source, isSigner: false, isWritable: true },
    { pubkey: destination, isSigner: false, isWritable: true },
    { pubkey: owner, isSigner: true, isWritable: false }
  ]

  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction: 3,
      amount
    },
    data
  )

  return new TransactionInstruction({
    keys,
    programId: TOKEN_PROGRAM_ID,
    data
  })
}

export function memoInstruction(memo: string) {
  return new TransactionInstruction({
    keys: [],
    data: Buffer.from(memo, 'utf-8'),
    programId: MEMO_PROGRAM_ID
  })
}
