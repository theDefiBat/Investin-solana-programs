import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import { connection, platformStateAccount, priceStateAccount, programId } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { signAndSendTransaction } from '../utils/web3'
import { devnet_pools, raydiumPools } from '../utils/pools';

const getPoolAccounts = () => {
    return raydiumPools.map((p) => {
      return [
        { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
      ]
    })
  }

export async function updatePoolPrices (transaction,  poolInfo) {
    
    const dataLayout = struct([u8('instruction'), u8('count')])
    console.log("raydiumPools length:: ", raydiumPools.length)

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 19,
            count: raydiumPools.length
        },
        data
    )
    const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformStateAccount, isSigner: false, isWritable: true },
          { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
          ...getPoolAccounts().flat()
        ],
        programId: programId,
        data
    });
    transaction.add(instruction)
    
}