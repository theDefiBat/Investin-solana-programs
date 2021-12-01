import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState , useEffect} from 'react'
import { GlobalState } from '../store/globalState';
import { signAndSendTransaction } from '../utils/web3'
import { connection, programId, priceStateAccount, platformStateAccount } from '../utils/constants';
import { struct, u8 } from 'buffer-layout';
import { TOKENS } from '../utils/tokens'
import { PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools';
import { IDS } from '@blockworks-foundation/mango-client';

const priceProgramId = new PublicKey('CB6oEYpfSsrF3oWG41KQxwfg4onZ38JMj1hk17UNe1Fn')


export const GetPrices = () => {
  let ids;
  if(process.env.REACT_APP_NETWORK==='devnet'){
     ids = IDS['groups'][2]
  } else {
     ids = IDS['groups'][0]
  }

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const [tokenList, setTokenList] = useState([ids.tokens[5]]) // SOL-4 SRM-5
    const [priceAccount, setPriceAccount] = useState('');
    const [poolName, setPoolName] = useState('');
    const [platformData, setPlatformData] = useState(0)
   const [tokenPrice, setTokenPrice] = useState(0)
   const [selectedTokenSymbol, setSelectedTokenSymbol] = useState('')


    useEffect(  ()=> {
      (async () => {
        const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
          const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
          // console.log("platformData::",platformData);
          setPlatformData(platformData)
          const platformTokens = platformData?.token_list;
          // console.log("platformTokens::",platformTokens);

          let t = []; 
          if(platformTokens?.length){
            t = platformTokens.map( (i) => {
              return {
                symbol: ids.tokens.find( k => k.mintAddress ===i.mint.toBase58()),
                mintAddress: i.mint.toBase58(),
                decimals: i.decimals?.toString()
              }
            })
          } 

          setTokenList(t)
      })()
      
    },[walletProvider])
    
    const handleGetAllPlatformTokens = async () => {
          console.log("platform tokens::",tokenList);

    }

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
      console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

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
      console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 
        
    }

    const handleGetPrices = async () => {
        // let acc = connection.getAccountInfo(platformStateAccount)
        // let data = PLATFORM_DATA.decode((await acc).data)
        // console.log("price data::: ", data)
      if(selectedTokenSymbol){
        const getMint = tokenList?.find( i => i.symbol === selectedTokenSymbol);
        const p = platformData?.token_list.find( i => i.mint.toBase58() === getMint.mintAddress)
        // console.log("price of selected token :",p)

        const selectedToken = {
          mint : p.mint.toBase58(),
          symbol : getMint.symbol,
          price : p.pool_price.toString()
        }
        console.log("price of selectedToken **:",selectedToken)

        setTokenPrice( p.pool_price.toString() );
      } else {
        alert("select token first")
      }
      
    }
    const handleTokenSelect = async(event) => {
        setPoolName(`${event.target.value}-USDC`)
       setSelectedTokenSymbol(event.target.value)
      }

    return (
        <div className="form-div">
          <h4>Get Token Prices</h4>

          <button onClick={handleGetAllPlatformTokens}>Get ALL platform Tokens in log</button>

          <label htmlFor="funds">Select a platform Token:</label>

            <select name="funds" width = "100px"  onChange={handleTokenSelect}>
            { tokenList && 
                tokenList.map((token,index) => {
                    return (<option key={index} value={token.symbol}>{token.symbol}</option>)
                })
            }
            </select>
          <br />
          <button onClick={handleGetPrices}>Get Price</button>

          <p> Selcted Token price: {tokenPrice}</p>
            
          <button onClick={handleAddToken}>Add Token to fund</button>

          <button onClick={handleUpdatePrices}>Update token Price</button>

        </div>
      )
}