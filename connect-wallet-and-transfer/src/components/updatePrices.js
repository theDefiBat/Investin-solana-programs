import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import { connection, priceStateAccount } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { signAndSendTransaction } from '../utils/web3'
import { devnet_pools } from '../utils/pools';

const priceProgramId = new PublicKey('HACWxPihVtqqvyf9T6i77VDAT7s9hFse8vHRVsinP3NG')

const getPoolAccounts = () => {
    return devnet_pools.map((p) => {
      return [
        { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
      ]
    })
  }

export async function updatePoolPrices (transaction,  poolInfo) {
    
    const dataLayout = struct([u8('instruction'), u8('count')])
    console.log("devnetpools length:: ", devnet_pools.length)

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 1,
            count: devnet_pools.length
        },
        data
    )
    const instruction = new TransactionInstruction({
        keys: [
          { pubkey: priceStateAccount, isSigner: false, isWritable: true },
          { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
          ...getPoolAccounts().flat()
        ],
        programId: priceProgramId,
        data
    });
    transaction.add(instruction)
    
}