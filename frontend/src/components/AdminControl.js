import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8 } from 'buffer-layout';
import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, PLATFORM_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import {  TEST_TOKENS, TOKENS } from "../utils/tokens";
import BN from 'bn.js';
import { Card, Col, Row } from 'reactstrap';

export const AdminControl = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleAdminControl = async () => {
    console.log("handle initalise fund clicked")

    const transaction = new Transaction()

    // ***what should be in the place of wallet provider in platformAccount
    const platformAccount = await createKeyIfNotExists(walletProvider, "", programId, PLATFORM_ACCOUNT_KEY, PLATFORM_DATA.span, transaction)

    console.log(`PLATFORM_DATA.span :::: `, PLATFORM_DATA.span)


    if (1) {
      const dataLayout = struct([
        u8('instruction'),
        u8('intialize_platform'),
        u8('freeze_platform'),
        u8('unfreeze_platform'),
        u8('change_vault'),
        u8('freeze_fund'),
        u8('unfreeze_fund'),
        nu64('change_min_amount'),
        nu64('change_min_return'),
        nu64('change_perf_fee')
      ])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 7,
          intialize_platform: v0,
          freeze_platform: v1,
          unfreeze_platform: v2,
          change_vault: v3,
          freeze_fund: v4,
          unfreeze_fund: v5,
          min_amount: min_amount * (10 ** TOKENS['USDC'].decimals),
          min_return: new BN(min_return),
          performance_fee_percentage: new BN(platform_fee_percentage),
        },
        data
      )
      const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['USDR'].mintAddress), walletProvider?.publicKey, transaction);

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformAccount, isSigner: false, isWritable: true },
          { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
          { pubkey: associatedTokenAddress1, isSigner: false, isWritable: true },
          { pubkey: new PublicKey(TEST_TOKENS['USDR'].mintAddress), isSigner: false, isWritable: true },

          // { pubkey: fundAccount, isSigner: false, isWritable: true },
        ],
        programId,
        data
      });
      transaction.add(instruction)
      transaction.feePayer = walletProvider?.publicKey;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 
    }

  }

  const handleAddTokenToWhitelist = async () => {
    let transaction = new Transaction()
    
    console.log("mintAddress,poolCoinTokenAccount, poolPcTokenAccount::",mintAddress,poolCoinTokenAccount, poolPcTokenAccount)
    const associatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(mintAddress), walletProvider?.publicKey, transaction);
    
    const associatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(mintAddress), walletProvider?.publicKey, transaction);
    
    
    const dataLayout = struct([u8('instruction'),u8('token_id'),u8('pc_index')])

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 17,
            token_id : 0,
            pc_index : 0,
        },
        data
    )
    const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformStateAccount, isSigner: false, isWritable: true },
          { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: true },
          { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
          { pubkey: new PublicKey(mintAddress), isSigner: false, isWritable: true },
          { pubkey: associatedTokenAddress, isSigner: false, isWritable: false },
          { pubkey: associatedTokenAddress, isSigner: false, isWritable: false },
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
  console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
  }

 

  const [v0, setv0] = useState(0);
  const [v1, setv1] = useState(0);
  const [v2, setv2] = useState(0);
  const [v3, setv3] = useState(0);
  const [v4, setv4] = useState(0);
  const [v5, setv5] = useState(0);

  const [min_amount, setMin_amount] = useState(0);
  const [min_return, setMin_return] = useState(0);
  const [platform_fee_percentage, setPlatform_fee_percentage] = useState(0);
  const [platformData, setPlatformData] = useState(0)

  const [mintAddress, setMintAddress] = useState('11111111111111111111111111111111')
  const [poolCoinTokenAccount, setPoolCoinTokenAccount] = useState('11111111111111111111111111111111')
  const [poolPcTokenAccount, setPoolPcTokenAccount] = useState('11111111111111111111111111111111')

  useEffect(  ()=> {
    (async () => {
      const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
        const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
        console.log("platformData::",platformData);
        setPlatformData(platformData)
    })()
    
  },[walletProvider])

  return (
    <div className="form-div">
       <Card className="justify-content-center">
            <Row className="justify-content-between">
       <Col lg="6" xs="6">
      <h4>Admin Controls</h4>
      init_platform ::: {' '}
      <input type="number" value={v0} onChange={(event) => setv0(event.target.value)} />
      <br />
      freeze_platform ::: {' '}
      <input type="number" value={v1} onChange={(event) => setv1(event.target.value)} />
      <br />
      unfreeze_platform ::: {' '}
      <input type="number" value={v2} onChange={(event) => setv2(event.target.value)} />
      <br />
      change_vault ::: {' '}
      <input type="number" value={v3} onChange={(event) => setv3(event.target.value)} />
      <br />
      freeze_fund ::: {' '}
      <input type="number" value={v4} onChange={(event) => setv4(event.target.value)} />
      <br />
      unfreeze_fund ::: {' '}
      <input type="number" value={v5} onChange={(event) => setv5(event.target.value)} />
      <br />

      min_amount ::: {' '}
      <input type="number" value={min_amount} onChange={(event) => setMin_amount(event.target.value)} />
      <br />
      min_return ::: {' '}
      <input type="number" value={min_return} onChange={(event) => setMin_return(event.target.value)} />
      <br />
      platform_fee_percentage ::: {' '}
      <input type="number" value={platform_fee_percentage} onChange={(event) => setPlatform_fee_percentage(event.target.value)} />
      <br />
      <button onClick={handleAdminControl}>Admin Control</button>

      <br />
      <hr/>
      <br />
       Whitelist token to platform {' '}<br />
       mintAddress : <input type="text" style={{width:'400px'}} value={mintAddress} onChange={(event) => setMintAddress(event.target.value)} /><br />
       poolCoinTokenAccount:  <input type="text" style={{width:'400px'}} value={poolCoinTokenAccount} onChange={(event) => setPoolCoinTokenAccount(event.target.value)} /><br />
       poolPcTokenAccount : <input type="text" style={{width:'400px'}} value={poolPcTokenAccount} onChange={(event) => setPoolPcTokenAccount(event.target.value)} /><br />
      <button onClick={handleAddTokenToWhitelist}>ADD</button>
      <hr/>
      </Col>

      <Col lg="6" xs="6">
      <h4>Platform State </h4>
      <p>version : {platformData?.version}</p>
        <p>investin_admin : {platformData?.investin_admin?.toBase58()}</p>
        <p>investin_vault : {platformData?.investin_vault?.toBase58()}</p>
        <p>router : {platformData?.router?.toBase58()}</p>
        <p>router_nonce : {platformData?.router_nonce}</p>
        <p>is_initialized : {platformData?.is_initialized}</p>
        <p>no_of_active_funds : {platformData?.no_of_active_funds}</p>
        <p>token_count : {platformData?.token_count}</p>
       
      </Col>
    </Row>
    <table>
          
          {
            platformData?.token_list && 

            platformData?.token_list.map((i,index)=>{
               return <tr key={i?.mint?.toBase58()}>
                 <td >{index}</td>
                 <td >{i?.mint?.toBase58()}</td>
                 <td>{i?.pool_coin_account?.toBase58()}</td>
                 <td>{i?.pool_pc_account?.toBase58()}</td>
               </tr>
            })
          }
          </table>
      </Card>
    </div>
  )
}

