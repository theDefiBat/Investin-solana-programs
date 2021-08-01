import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { signAndSendTransaction } from '../utils/web3'
import { connection, programId, priceStateAccount, platformStateAccount } from '../utils/constants';
import { struct, u8 } from 'buffer-layout';
import { MANGO_TOKENS } from '../utils/tokens'
import { PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools';

const priceProgramId = new PublicKey('CB6oEYpfSsrF3oWG41KQxwfg4onZ38JMj1hk17UNe1Fn')
const tokensList = [
    MANGO_TOKENS['SRM']
]

export const GetPrices = () => {
    const [priceAccount, setPriceAccount] = useState('');
    const [poolName, setPoolName] = useState('');

    const walletProvider = GlobalState.useState(s => s.walletProvider);


    const handleAddToken = async () => {
       
        let transaction = new Transaction()
        if (!poolName)
        {
            alert("no token pool found")
            return
        }
        const poolInfo = devnet_pools.find(p => p.name === poolName);
        console.log(poolInfo)
        const dataLayout = struct([u8('instruction')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
                instruction: 17,
            },
            data
        )
        const instruction = new TransactionInstruction({
            keys: [
              { pubkey: platformStateAccount, isSigner: false, isWritable: true },
              { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
              { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
              { pubkey: new PublicKey(poolInfo.coin.mintAddress), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolCoinTokenAccount), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolPcTokenAccount), isSigner: false, isWritable: true },
            ],
            programId: programId,
            data
        });
        transaction.add(instruction)
        transaction.feePayer = walletProvider?.publicKey;
        console.log("trnsaction:: ", transaction)
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
    }

    const handleUpdatePrices = async () => {
       
        if (!poolName)
        {
            alert("no token pool found")
            return
        }
        const poolInfo = devnet_pools.find(p => p.name === poolName);
        console.log(poolInfo)
        const dataLayout = struct([u8('instruction'), u8('count')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
                instruction: 18,
                count: 1
            },
            data
        )
        const instruction = new TransactionInstruction({
            keys: [
              { pubkey: platformStateAccount, isSigner: false, isWritable: true },
              { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolCoinTokenAccount), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolPcTokenAccount), isSigner: false, isWritable: true },
            ],
            programId: programId,
            data
        });
        let transaction = new Transaction()
        transaction.add(instruction)
        transaction.feePayer = walletProvider?.publicKey;
        console.log("trnsaction:: ", transaction)
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
    }

    const handleGetPrices = async () => {
        let acc = connection.getAccountInfo(platformStateAccount)
        let data = PLATFORM_DATA.decode((await acc).data)

        console.log("price data::: ", data)
    }
    const handleTokenSelect = async(event) => {
        setPoolName(`${event.target.value}-USDC`)
      }

    return (
        <div className="form-div">
          <h4>Get Token Prices</h4>
          <label htmlFor="funds">Select Token:</label>

            <select name="funds" width = "100px" onClick={handleTokenSelect}>
            {
                tokensList.map((token) => {
                    return (<option key={token.symbol} value={token.symbol}>{token.symbol}</option>)
                })
            }
            </select>
          <br />
          <button onClick={handleGetPrices}>Get Prices</button>
            
          <button onClick={handleAddToken}>Add Token</button>

          <button onClick={handleUpdatePrices}>Update Price</button>

        </div>
      )
}