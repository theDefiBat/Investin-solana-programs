import { getterTree, mutationTree, actionTree } from 'typed-vuex'

import { ACCOUNT_LAYOUT, MINT_LAYOUT } from './layouts'
import { AMM_INFO_LAYOUT, AMM_INFO_LAYOUT_V3, AMM_INFO_LAYOUT_V4 } from './liquidity'
import { LIQUIDITY_POOLS, getAddressForWhat } from './pools'
import { commitment, getMultipleAccounts } from './web3'

import { OpenOrders } from '@project-serum/serum'
import { Connection, PublicKey } from '@solana/web3.js'
import { TokenAmount } from './safe-math'
import { cloneDeep } from 'lodash'
import logger from './logger'



export function loadInfo(conn: Connection){
  {
      const liquidityPools = {} as any
      const publicKeys = [] as any

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
    }
  }
)
