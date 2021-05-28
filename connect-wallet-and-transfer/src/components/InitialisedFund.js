import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8 } from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, PLATFORM_DATA } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import { TEST_TOKENS } from "../utils/tokens";

export const InitialisedFund = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  // useEffect(() => {
  //   //   await connection.getTokenAccountsByOwner(new PublicKey("8Yx3Fo5Q6vbR9zourTBnpmBCydP71KmqfSg3dUFtUZ9D"), { mint: new PublicKey("7ujhhEvmSr33MeZybgpQXanDNnHzHJ7b27v3JMPzjBBw") })

  //   if (walletProvider) {
  //     (async () => {
  //       // const PDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

  //       console.log(`object ::: `,
  //         await connection.getTokenAccountsByOwner(PDA[0], { programId: TOKEN_PROGRAM_ID }))
  //     })()
  //   }
  // }, [walletProvider])

  const handleInitialFund = async () => {
    console.log("handle initalise fund clicked")

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    const routerPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);

    // ***what should be in the place of wallet provider in platformAccount
    const platformAccount = platformStateAccount;
    //const platformAccount = await createKeyIfNotExists(walletProvider, "", programId, PLATFORM_ACCOUNT_KEY, PLATFORM_DATA.span)
    const fundAccount = await createKeyIfNotExists(walletProvider, "", programId, FUND_ACCOUNT_KEY, FUND_DATA.span)


    console.log(`fundPDA::: `, fundPDA[0].toBase58())
    console.log('routerPDA:: ', routerPDA[0].toBase58())
    console.log(`platformData ::: `, platformAccount.toBase58())
    console.log(`fundAccount ::: `, fundAccount.toBase58())

    const fundData = await connection.getAccountInfo(fundAccount, "max");
    const platformData = await connection.getAccountInfo(platformAccount, "max");

    const x = FUND_DATA.decode(fundData.data)
    console.log('funddata span', FUND_DATA.span)
    console.log('platformData span', PLATFORM_DATA.span)
    console.log(`fundData parsed`, x)
    console.log(`platformData parsed`, PLATFORM_DATA.decode(platformData.data))
    console.log("routerPDA: ", PLATFORM_DATA.decode(platformData.data).router.toBase58())


    if (!x.is_initialized) {
      const dataLayout = struct([u8('instruction'), nu64('min_amount'), nu64('min_return'), nu64('performance_fee_percentage')])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 0,
          min_amount: min_amount * 1000000,
          min_return: min_return * 100,
          performance_fee_percentage: platform_fee_percentage * 100,
        },
        data
      )

      const PDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const associatedTokenAccounts = await connection.getTokenAccountsByOwner(PDA[0], { programId: TOKEN_PROGRAM_ID });

      const associatedTokenAddresses = associatedTokenAccounts.value.map(p => p.pubkey);

      console.log(`associatedTokenAccounts.value ::: `, associatedTokenAccounts.value)
    
      const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider?.publicKey, new PublicKey(...TEST_TOKENS.USDP.mintAddress), PDA[0]);
      const associatedTokenAddress2 = await createAssociatedTokenAccountIfNotExist(walletProvider?.publicKey, new PublicKey(...TEST_TOKENS.ALPHA.mintAddress), PDA[0]);
      const associatedTokenAddress3 = await createAssociatedTokenAccountIfNotExist(walletProvider?.publicKey, new PublicKey(...TEST_TOKENS.BETA.mintAddress), PDA[0]);

     
      console.log(`associatedTokenAddress1 ::: `, associatedTokenAccounts)


      console.log(`PDA ::: `, PDA[0].toBase58())

      console.log(`platformAccount.toBase58() ::: `, platformAccount.toBase58())
      console.log(`fundAccount.toBase58() ::: `, fundAccount.toBase58())
      console.log(`walletProvider?.publicKey.toBase58() ::: `, walletProvider?.publicKey.toBase58())
      console.log(`associatedTokenAddress1.toBase58() ::: `, associatedTokenAddresses[0].toBase58())

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformAccount, isSigner: false, isWritable: true },
          { pubkey: fundAccount, isSigner: false, isWritable: true },
          { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
          { pubkey: associatedTokenAddresses[0], isSigner: false, isWritable: true },
          { pubkey: new PublicKey(...TEST_TOKENS.USDP.mintAddress), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(...TEST_TOKENS.ALPHA.mintAddress), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(...TEST_TOKENS.BETA.mintAddress), isSigner: false, isWritable: true },
        ],
        programId,
        data
      });

      const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
      const signature = await signAndSendTransaction(walletProvider, transaction2);
      console.log(`signature :::`, signature)


      const accData = await connection.getAccountInfo(platformAccount);
      console.log(`platformData :::: `, PLATFORM_DATA.decode(accData.data));
      const accData2 = await connection.getAccountInfo(fundAccount);
      console.log(`fundData :::: `, FUND_DATA.decode(accData2.data));
      console.log("")

    }

    GlobalState.update(s => {
      s.createFundPublicKey = fundAccount;
    })
  }

  const [min_amount, setMin_amount] = useState(0);
  const [min_return, setMin_return] = useState(0);
  const [platform_fee_percentage, setPlatform_fee_percentage] = useState(0);

  return (
    <div className="form-div">
      <h4>Initialise Fund</h4>
      min_amount ::: {' '}
      <input type="number" value={min_amount} onChange={(event) => setMin_amount(event.target.value)} />
      <br />
      min_return ::: {' '}
      <input type="number" value={min_return} onChange={(event) => setMin_return(event.target.value)} />
      <br />
      platform_fee_percentage ::: {' '}
      <input type="number" value={platform_fee_percentage} onChange={(event) => setPlatform_fee_percentage(event.target.value)} />
      <br />
      <button onClick={handleInitialFund}>initialise fund</button>
    </div>
  )
}

