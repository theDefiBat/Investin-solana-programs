import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { createKeyIfNotExists, signAndSendTransaction } from '../utils/web3'
import { adminAccount, connection, TOKEN_PROGRAM_ID, PRICE_ACCOUNT_KEY, programId, priceStateAccount } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { TEST_TOKENS, TOKENS } from '../utils/tokens'
import { PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools';

const priceProgramId = new PublicKey('HACWxPihVtqqvyf9T6i77VDAT7s9hFse8vHRVsinP3NG')
const tokensList = [
    TOKENS['WSOL'],
    TOKENS['SRM'],
    TOKENS['STEP'],
    TOKENS['ALEPH'],
    TOKENS['ROPE'],
    TOKENS['MEDIA'],
    TOKENS['MER'],
    TOKENS['COPE'],
    TOKENS['TULIP']
]
export const GetPrices = () => {
    const [priceAccount, setPriceAccount] = useState('');
    const [poolName, setPoolName] = useState('');

    const walletProvider = GlobalState.useState(s => s.walletProvider);


    const handleAddToken = async () => {
       
        let transaction = new Transaction()

        //const priceAccount = priceStateAccount;
        const priceAccount = await createKeyIfNotExists(walletProvider, "", priceProgramId, PRICE_ACCOUNT_KEY, PRICE_DATA.span, transaction)
        console.log('pool ', poolName)
        console.log('account size:: ', PRICE_DATA.span)
        console.log('priceAccount::', priceAccount.toBase58())
        if (!poolName)
        {
            alert("no token pool found")
            return
        }
        const poolInfo = pools.find(p => p.name === poolName);
        console.log(poolInfo)
        const dataLayout = struct([u8('instruction'), u8('count')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
                instruction: 0,
                count: 1
            },
            data
        )
        const instruction = new TransactionInstruction({
            keys: [
              { pubkey: priceAccount, isSigner: false, isWritable: true },
              { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
              { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
              { pubkey: new PublicKey(poolInfo.coin.mintAddress), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolCoinTokenAccount), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolPcTokenAccount), isSigner: false, isWritable: true },
            ],
            programId: priceProgramId,
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
       

        const priceAccount = priceStateAccount;
        //const priceAccount = await createKeyIfNotExists(walletProvider, "", priceProgramId, PRICE_ACCOUNT_KEY, PRICE_DATA.span)
        console.log('pool ', poolName)
        console.log('priceAccount::', priceAccount.toBase58())
        if (!poolName)
        {
            alert("no token pool found")
            return
        }
        const poolInfo = pools.find(p => p.name === poolName);
        console.log(poolInfo)
        const dataLayout = struct([u8('instruction'), u8('count')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
                instruction: 1,
                count: 1
            },
            data
        )
        const instruction = new TransactionInstruction({
            keys: [
              { pubkey: priceAccount, isSigner: false, isWritable: true },
              { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolCoinTokenAccount), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(poolInfo.poolPcTokenAccount), isSigner: false, isWritable: true },
            ],
            programId: priceProgramId,
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
        let acc = connection.getAccountInfo(priceStateAccount)
        let data = PRICE_DATA.decode((await acc).data)

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