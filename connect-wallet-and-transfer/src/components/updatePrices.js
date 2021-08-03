import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import { connection, platformStateAccount, priceStateAccount, programId } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { signAndSendTransaction } from '../utils/web3'
import { pools } from '../utils/pools';

const getPoolAccounts = (poolInfo) => {
    return poolInfo.map((p) => {
      return [
        { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
      ]
    })
  }

export async function updatePoolPrices (transaction,  poolInfo) {
    
    const dataLayout = struct([u8('instruction'), u8('count')])
    console.log("devnetpools length:: ", poolInfo.length)

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
    const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformStateAccount, isSigner: false, isWritable: true },
          { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
          ...getPoolAccounts(poolInfo).flat()
        ],
        programId: programId,
        data
    });
    transaction.add(instruction)
    
}