import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState , useEffect} from 'react'
import { GlobalState } from '../store/globalState';
import { createAssociatedTokenAccountIfNotExist, signAndSendTransaction } from '../utils/web3'
import { connection, programId, priceStateAccount, platformStateAccount, idsIndex, FUND_ACCOUNT_KEY } from '../utils/constants';
import { struct, u8 } from 'buffer-layout';
import { TOKENS } from '../utils/tokens'
import { FUND_DATA, PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, DEV_TOKENS, pools, raydiumPools } from '../utils/pools';
import { IDS } from '@blockworks-foundation/mango-client';

const priceProgramId = new PublicKey('CB6oEYpfSsrF3oWG41KQxwfg4onZ38JMj1hk17UNe1Fn')


export const GetPrices = () => {
  
  const ids= IDS['groups'][idsIndex];

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const [tokenList, setTokenList] = useState([ids.tokens[0]]) // SOL-4 SRM-5
    const [priceAccount, setPriceAccount] = useState('');
    const [poolName, setPoolName] = useState('');
    const [platformData, setPlatformData] = useState(0)
   const [tokenPrice, setTokenPrice] = useState(0)
   const [selectedTokenSymbol, setSelectedTokenSymbol] = useState('')


    useEffect(  ()=> {
      (async () => {
        const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
        if(!platformDataAcc){
          alert('platform state not initilaized');
          return;
        }
          const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
          // console.log("platformData::",platformData);
          setPlatformData(platformData)
          const platformTokens = platformData?.token_list;
          console.log("platformTokens::",platformTokens);

          let t = []; 
          if(platformTokens?.length){
            t = platformTokens.map( (i) => {
              return {
                symbol: ids.tokens.find( k => k.mintKey ===i.mint.toBase58())?.symbol ?? 'NONE',
                mintAddress: i.mint.toBase58(),
                decimals: i.decimals?.toString(),
                pool_coin_account: i.pool_coin_account.toBase58(),
                pool_pc_account: i.pool_pc_account.toBase58(),
                pool_price : i.pool_price?.toString(),
              }
            })
          } 
          console.log("t:",t)

          setTokenList(t)
      })()
      
    },[walletProvider])
    
    const handleGetAllPlatformTokens = async () => {
          console.log("platform tokens::",tokenList);

    }

    const handleAddToken = async () => {
       
      console.log("**handleAddToken  selectedTokenSymbol::",selectedTokenSymbol)
      const tokenMintAddr = DEV_TOKENS[selectedTokenSymbol.toUpperCase()]?.mintKey
      console.log(" tokenMintAddr::",tokenMintAddr);

      const transaction = new Transaction()
  
      // const toCoinMint = poolInfo.pc.mintAddress;
      // const fromCoinMint = poolInfo.coin.mintAddress;
      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const associatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(tokenMintAddr), fundPDA[0], transaction);
  
      const fundStateAcc = await PublicKey.createWithSeed(
        walletProvider?.publicKey,
          FUND_ACCOUNT_KEY,
          programId,
      );
  
      let fund_info = await connection.getAccountInfo(fundStateAcc);
      const fund_data = FUND_DATA.decode(fund_info.data); 
      console.log("fund_data:",fund_data)
  
      let unUsedTokenIndex = 0 ;
      for (let i = 0; i < fund_data.tokens.length; i++) {
          if(fund_data.tokens[i].is_active === 0) {
              unUsedTokenIndex = i;
              break;
          }
          
      } 
      console.log("unUsedTokenIndex:",unUsedTokenIndex)
  
      if(unUsedTokenIndex === -1) {
          throw("Cannot add tokens, limit of 8 reached");
      }
  
      const dataLayout = struct([u8('instruction'), u8('index')])
  
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
          {
              instruction: 20,
              index: unUsedTokenIndex
          },
          data
      )
      const transfer_instruction = new TransactionInstruction({
          keys: [
              { pubkey: platformStateAccount, isSigner: false, isWritable: true },
              { pubkey: fundStateAcc, isSigner: false, isWritable: true },
              { pubkey: new PublicKey(tokenMintAddr), isSigner: false, isWritable: true },
              { pubkey: associatedTokenAddress, isSigner: false, isWritable: true },
          ],
          programId,
          data
      });
  
       transaction.add(transfer_instruction);
        transaction.feePayer = walletProvider?.publicKey;
        console.log("trnsaction:: ", transaction)
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

    }

    const handleUpdatePrices = async () => {
       
        if (!poolName)
        {
            alert("no token pool found")
            return
        }
        const poolInfo = raydiumPools.find(p => p.name === poolName);
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
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
        
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
          <h4>Get Token Prices / ADD or REMOVE token from FUND </h4>

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