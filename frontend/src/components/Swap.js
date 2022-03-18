import { IDS } from '@blockworks-foundation/mango-client'
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js'
import { nu64, struct, u8 } from 'buffer-layout'
import React, { useState , useEffect} from 'react'
import { GlobalState } from '../store/globalState'
import { connection, programId, TOKEN_PROGRAM_ID, FUND_ACCOUNT_KEY, LIQUIDITY_POOL_PROGRAM_ID_V4, platformStateAccount, idsIndex } from '../utils/constants'
import { devnet_pools, orcaPools, raydiumPools } from '../utils/pools'
import { AMM_INFO_LAYOUT_V4, FUND_DATA, FUND_PDA_DATA, PLATFORM_DATA } from '../utils/programLayouts'
import { TokenAmount } from '../utils/safe-math'
import {  NATIVE_SOL, TEST_TOKENS, TOKENS } from '../utils/tokens'
import { createAssociatedTokenAccountIfNotExist, createTokenAccountIfNotExist, findAssociatedTokenAddress, sendNewTransaction, signAndSendTransaction } from '../utils/web3'

const ids= IDS['groups'][idsIndex];

export const Swap = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const tokensStatic = Object.entries(TOKENS).map( i => i[1])

 const [toggleGetTokens, setToggleGetTokens] = useState(false)

  const [firstTokenAmount, setFirstTokenAmount] = useState(0);
  const [selectedFirstToken, setSelectedFirstToken] = useState('USDC');
  const [selectedSecondToken, setSelectedSecondToken] = useState('BTC');
  const [isBuy, setIsBuy] = useState(1)
  // const [selectedTokenSymbol, setSelectedTokenSymbol] = useState('')
  const [selectedSwapProtocol, setSelectedSwapProtocol] = useState(0);

   const [fundStateAccount, setFundStateAccount] = useState('');
   const [fundPDA, setFundPDA] = useState('')
   const [platformData, setPlatformData] = useState(0)

  const [fundData, setFundData] = useState(0)
  const [tokenList, setTokenList] = useState([])
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
      // console.log("platformTokens::",platformTokens);

      let platformTokensList = []; 
      if(platformTokens?.length) {
        platformTokensList = platformTokens.map( (i) => {
          return {
            symbol: tokensStatic?.find( k => k.mintAddress ===i.mint.toBase58())?.symbol,
            mintAddress: i.mint.toBase58(),
            decimals: i.decimals?.toString()
          }
        })
      } 
      console.log("platformTokensList::",platformTokensList);


      const key = walletProvider?.publicKey;  
        if (!key ) {
          // alert("connect wallet")
          return;
        }
        const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

        // const fundStateAcc = await PublicKey.createWithSeed(
        //   key,
        //   FUND_ACCOUNT_KEY,
        //   programId,
        // );
        // console.log("FUND fundStateAcc:: ", fundStateAcc.toBase58())
        // setFundStateAccount(fundStateAcc.toBase58())

        const fundDataAcc = await connection.getAccountInfo(fundPDA[0]);
        console.log("fundDataAcc::",fundDataAcc);
        if (fundDataAcc == null)
        {
           alert("fundDataAcc info not found")
           return;
        }
         const fundData = FUND_PDA_DATA.decode(fundDataAcc.data)
         console.log("fundData::",fundData);
         setFundData(fundData)
         const fundTokens = fundData?.tokens;
         console.log("fundTokens ::",fundTokens);

         let fundTokensList = []; 
         if(fundTokens?.length) {
          fundTokensList = fundTokens.map((i) => {
            return tokensStatic.find(x => x.mintAddress === platformTokensList[i.index[i.mux]].mintAddress )
          })
         } 
         console.log("fundTokensList ::",fundTokensList);

         setTokenList(fundTokensList)
     })()
     
   },[walletProvider,toggleGetTokens])



  const handleFirstTokenSelect = (event) => {
    setSelectedFirstToken(`${event.target.value}`);
    console.log(`${event.target.value}-1 :::: `, `${event.target.value}-1`)
  }

  const handleSecondTokenSelect = (event) => {
    setSelectedSecondToken(`${event.target.value}`);
    console.log(`${event.target.value}-2 :::: `, `${event.target.value}-2`)
  }

  const swapInstructionRaydium = async (
    walletProviderP,
    poolProgramId,
    // tokenProgramId,
    // amm
    ammId,
    ammAuthority,
    ammOpenOrders,
    ammTargetOrders,
    poolCoinTokenAccount,
    poolPcTokenAccount,
    // serum
    serumProgramId,
    serumMarket,
    serumBids,
    serumAsks,
    serumEventQueue,
    serumCoinVaultAccount,
    serumPcVaultAccount,
    serumVaultSigner,
    // user
    userSourceTokenAccount,
    userDestTokenAccount,
    userOwner,
    manager,

    amountIn,
    minAmountOut
) => {
    
   

    const key = walletProvider?.publicKey;
    console.log("key::",key.toBase58())
    if (!key) {
        console.log("connect wallet")
        return;
    }

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    console.log("fundPDA-tobase:",fundPDA[0].toBase58())

    // const fundStateAcc = await PublicKey.createWithSeed(
    //     key,
    //     FUND_ACCOUNT_KEY,
    //     programId,
    // );

    // const dataLayout = struct([u8('instruction1'), u8('instruction'), nu64('amountIn'), nu64('minAmountOut')])

    const dataLayout = struct([u8('instruction1'), u8('swap_index'), u8('instruction'), nu64('amountIn'), nu64('minAmountOut')])

    const keys = [
        { pubkey: platformStateAccount, isSigner: false, isWritable: true },
        { pubkey: fundPDA[0], isSigner: false, isWritable: true },
        { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },

        { pubkey: new PublicKey(poolProgramId) , isSigner: false, isWritable: false },

        // spl token
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        // amm
        { pubkey: ammId, isSigner: false, isWritable: true },
        { pubkey: ammAuthority, isSigner: false, isWritable: true },
        { pubkey: ammOpenOrders, isSigner: false, isWritable: true },
        { pubkey: ammTargetOrders, isSigner: false, isWritable: true },
        { pubkey: poolCoinTokenAccount, isSigner: false, isWritable: true },
        { pubkey: poolPcTokenAccount, isSigner: false, isWritable: true },
        // serum
        { pubkey: serumProgramId, isSigner: false, isWritable: false },
        { pubkey: serumMarket, isSigner: false, isWritable: true },
        { pubkey: serumBids, isSigner: false, isWritable: true },
        { pubkey: serumAsks, isSigner: false, isWritable: true },
        { pubkey: serumEventQueue, isSigner: false, isWritable: true },
        { pubkey: serumCoinVaultAccount, isSigner: false, isWritable: true },
        { pubkey: serumPcVaultAccount, isSigner: false, isWritable: true },
        { pubkey: serumVaultSigner, isSigner: false, isWritable: true },

        { pubkey: userSourceTokenAccount, isSigner: false, isWritable: true },
        { pubkey: userDestTokenAccount, isSigner: false, isWritable: true },
        { pubkey: userOwner, isSigner: false, isWritable: true }
    ]
    for(let i=0; i<keys.length;i++) {
      console.log("key:",i, keys[i].pubkey.toBase58())
    }

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction1: 5,
            swap_index : 0,
            instruction: 9,
            amountIn,
            minAmountOut
        },
        data
    ) 

    return new TransactionInstruction({
        keys,
        programId: programId,
        data
    })
}

const swapInstructionOrca = async () => {
  console.log("latttter");
  alert("Noorca swap ")
}

  const swapTokens = async (
    connection,
    walletProvider,
    fundPDA,
    poolInfo,
    fromCoinMint,
    toCoinMint,
    fromTokenAccount,
    toTokenAccount,
    amount,
    slippage,
    tradeSide,
    selectedSwapProtocol
) => {

  console.log("swapTokens with params: ",
     fundPDA,
    poolInfo,
    fromCoinMint,
    toCoinMint,
    fromTokenAccount,
    toTokenAccount,
    amount,
    slippage,
    tradeSide,
    selectedSwapProtocol
   )

    const transaction = new Transaction()
    const signers = []

    const owner = walletProvider?.publicKey

    // const { amountIn, amountOut } = getSwapOutAmount(poolInfo, fromCoinMint, toCoinMint, amount, slippage)
    let amountIn = new TokenAmount(amount, poolInfo.coin.decimals, false)
    let amountOut = new TokenAmount(1, poolInfo.coin.decimals)
    if (tradeSide === "sell") {
        amountIn = new TokenAmount(amount, poolInfo.coin.decimals, false)
        amountOut = new TokenAmount(1, poolInfo.coin.decimals)
    } else {
        amountIn = new TokenAmount(amount, poolInfo.pc.decimals, false)
        amountOut = new TokenAmount(1, poolInfo.pc.decimals)
    }

    let fromMint = fromCoinMint
    let toMint = toCoinMint

    if (fromMint ===  NATIVE_SOL.mintAddress) {
        fromMint = ids.tokens[4].mintKey
    }
    if (toMint === NATIVE_SOL.mintAddress) {
        toMint = ids.tokens[4].mintKey
    }

    const newFromTokenAccount = fromTokenAccount
    const newToTokenAccount = toTokenAccount

    let instruction;

    console.log("poolInfo.programId::",poolInfo.programId)

    if(selectedSwapProtocol == 0){
        instruction = await  swapInstructionRaydium(
             walletProvider,
             poolInfo.programId,
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
             newFromTokenAccount,
             newToTokenAccount,
             fundPDA,
             owner,
             Math.floor(amountIn.toWei().toNumber()),
             Math.floor(amountOut.toWei().toNumber())
         )
    } else {
        instruction = await swapInstructionOrca(
             walletProvider,
             poolInfo.programId,

             new PublicKey(poolInfo.ammId),
             new PublicKey(poolInfo.ammAuthority),
             new PublicKey(poolInfo.poolCoinTokenAccount),
             new PublicKey(poolInfo.poolPcTokenAccount),

             newFromTokenAccount,
             newToTokenAccount,
             fundPDA,
             owner,
             
             new PublicKey(poolInfo.feeAccount),
             new PublicKey(poolInfo.lpMintAddress),

             Math.floor(amountIn.toWei().toNumber()),
             Math.floor(amountOut.toWei().toNumber()),
             tradeSide
         )
    }
    
   
    transaction.add(instruction)
    transaction.feePayer = owner;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
    // return sign;
    //return await sendNewTransaction(connection, wallet, transaction, signers)
  }

  const handleSwap = async () => {
    try {

        console.log("isBuy,firstTokenAmount,selectedFirstToken,selectedSecondToken::",isBuy,firstTokenAmount,selectedFirstToken,selectedSecondToken)
        // if( (selectedFirstToken === "USDC" && selectedSecondToken === "WSOL") 
        //      || (selectedFirstToken === "WSOL" && selectedSecondToken === "USDC")){
        //     isBuy = selectedFirstToken === "USDC"  ;
        // } else {
        //     isBuy = selectedFirstToken === "USDC" || selectedFirstToken === "WSOL" ;
        // }
        // const isBuy = ['USDC', 'WSOL'].includes(firstToken.symbol);

         //this is will break in case of selling WSOL-> USDC
        // const poolName =  isBuy ? `${selectedSecondToken}-${selectedFirstToken}` : `${selectedFirstToken}-${selectedSecondToken}`;
        const poolName1 = `${selectedSecondToken}-${selectedFirstToken}`
        const poolName2 = `${selectedFirstToken}-${selectedSecondToken}`
        console.log("poolName1, poolName2 , isBuy::: ",poolName1,poolName2, isBuy);

        let poolInfo;
        if (selectedSwapProtocol == 0) {
            poolInfo  = raydiumPools.find(p => (p.name===poolName1 ||  p.name===poolName2) );
        } else {
          poolInfo  = orcaPools.find(p => (p.name===poolName1 ||  p.name===poolName2) );
            // poolInfo  = devnet_pools.find(p => (p.name===poolName1 ||  p.name===poolName2) );
        }
        if(!poolInfo){
          alert("poolinfo undefined")
          return;
        }
        console.log("poolInfo:",poolInfo)
        const fromCoin = isBuy ? poolInfo.pc : poolInfo.coin;
        const toCoin = isBuy ? poolInfo.coin : poolInfo.pc;
        const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
        console.log("fundPDA-tobase:",fundPDA[0].toBase58())

        const fromTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(fromCoin.mintAddress));
        const toTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(toCoin.mintAddress));

        console.log("fromCoin,toCoin :",fromCoin,toCoin)
        console.log("fromTokenAccount,toTokenAccount :",fromTokenAccount.toBase58(),toTokenAccount.toBase58())

        try {
            const txId = await swapTokens(connection, walletProvider, fundPDA[0], poolInfo, fromCoin?.mintAddress,
                toCoin?.mintKey, fromTokenAccount, toTokenAccount, firstTokenAmount, 1, isBuy ? "buy" : "sell",
                selectedSwapProtocol
            );
            
        } catch (error) {
            console.error(" swap error:", error);
        }
    } catch (error) {
        console.error("error while swapping : ", error)
    }
}

  return (
    <div className="form-div">
      <h4> Swap</h4>
      fundPDA : {fundPDA}
      <br />
      Swaping  ::: {selectedFirstToken} {"=>>"} {selectedSecondToken}  
      <br />

      {/* <label htmlFor="tokens">USDC and  Token:</label> */}
      <button onClick={()=> setToggleGetTokens(!toggleGetTokens)}>LOAD FUND TOKENS</button>
      FROM ::
      <select name="tokens" onChange={handleFirstTokenSelect}>
         {
          tokenList.map((i,index) => {
            return (<option key={index} value={i.symbol}>{i.symbol}</option>)
          })
        }
      </select>
        TO :: 
      <select name="tokens" onChange={handleSecondTokenSelect}>
         {
          tokenList.map((i,index) => {
            return (<option key={index} value={i.symbol}>{i.symbol}</option>)
          })
        }
      </select>
      <br />
      <select name="protocol" onChange={ (event) => setSelectedSwapProtocol(event.target.value)}>
            <option key={0} value={0}>RAYDIUM</option>
            <option key={1} value={1}>ORCA</option>
      </select>
      {/* <br /> */}

      <select name="buy" onChange={ (event) => setIsBuy(parseInt(event.target.value))}>
            <option key={1} value={1}>BUY</option>
            <option key={0} value={0}>SELL</option>
      </select>
      <br />

      amount : {' '}<input type="number" value={firstTokenAmount} onChange={(e) => setFirstTokenAmount(e.target.value)} /><br />
    
      <button margin-right="10px" onClick={handleSwap} >TRADE</button>
      
    </div>
  )
}


