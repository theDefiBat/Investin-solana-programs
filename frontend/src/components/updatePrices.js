import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import { connection, LIQUIDITY_POOL_PROGRAM_ID_V4, platformStateAccount, priceStateAccount, programId } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { signAndSendTransaction } from '../utils/web3'
import { devnet_pools, raydiumPools } from '../utils/pools';


const getPoolAccounts = (poolInfo) => {
    return poolInfo.map((p) => {
      if(p.programId == LIQUIDITY_POOL_PROGRAM_ID_V4) {
        return [
          { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(p.ammOpenOrders), isSigner: false, isWritable: false},
          { pubkey: new PublicKey(p.ammId), isSigner: false, isWritable: false}
        ]
      } else {
        return [
          { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
        ]
      }
    })
  }

export async function updatePoolPrices (transaction,  poolInfo) {
    
    const dataLayout = struct([u8('instruction'), u8('count')])
    console.log("poolInfo length:: ", poolInfo.length)

    if (poolInfo.length == 0) {
      return
    }

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 19,
            count: poolInfo.length
        },
        data
    )
    const keys = [
      { pubkey: platformStateAccount, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
      ...getPoolAccounts(poolInfo).flat()
    ]

    for(let i=0; i<keys.length;i++) {
      console.log("updatePoolPrices key:",i, keys[i].pubkey.toBase58())
    }
    const instruction = new TransactionInstruction({
        keys,
        programId: programId,
        data
    });
    transaction.add(instruction)
    
}