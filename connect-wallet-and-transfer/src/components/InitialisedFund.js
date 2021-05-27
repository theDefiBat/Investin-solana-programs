import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8 } from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA } from '../utils/programLayouts';

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
    // ***what should be in the place of wallet provider in platformAccount
    const platformAccount = await createKeyIfNotExists(walletProvider, undefined, programId, 355)
    const fundAccount = await createKeyIfNotExists(walletProvider, undefined, programId, (219 + 320 + 85))

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);


    console.log(`fundAccount ::: `, fundAccount.toBase58())
    console.log(`fundPDA ::: `, fundPDA[0].toBase58())
    const fundData = await connection.getAccountInfo(fundAccount);
    console.log(`fundData ::: `, fundData)
    const x = FUND_DATA.decode(fundData.data)

    console.log(`object :::: `, x)
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

      const transaction = new Transaction()

      const clientAccount = await connection.getAccountInfo(walletProvider?.publicKey);


      const PDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const associatedTokenAccounts = await connection.getTokenAccountsByOwner(PDA[0], { programId: TOKEN_PROGRAM_ID });

      const associatedTokenAddresses = associatedTokenAccounts.value.map(p => p.pubkey);

      console.log(`associatedTokenAccounts.value ::: `, associatedTokenAccounts.value)


      if (associatedTokenAccounts.value.length !== 3) {
        const associatedTokenAddress1 = await createAssociatedTokenAccount(walletProvider?.publicKey, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), PDA[0], transaction);
        const associatedTokenAddress2 = await createAssociatedTokenAccount(walletProvider?.publicKey, new PublicKey('HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o'), PDA[0], transaction);
        const associatedTokenAddress3 = await createAssociatedTokenAccount(walletProvider?.publicKey, new PublicKey('HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty'), PDA[0], transaction);

        associatedTokenAddresses.push(associatedTokenAddress1, associatedTokenAddress2, associatedTokenAddress3)
      }

      // const associatedTokenAddress1 = await createAssociatedTokenAccount(walletProvider?.publicKey, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), PDA[0], transaction);
      // const associatedTokenAddress2 = await createAssociatedTokenAccount(walletProvider?.publicKey, new PublicKey('HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o'), PDA[0], transaction);
      // const associatedTokenAddress3 = await createAssociatedTokenAccount(walletProvider?.publicKey, new PublicKey('HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty'), PDA[0], transaction);

      console.log(`associatedTokenAddress1 ::: `, associatedTokenAccounts)
      // console.log(`associatedTokenAddress2 ::: `, associatedTokenAddress2)
      // console.log(`associatedTokenAddress3 ::: `, associatedTokenAddress3)

      console.log(`transaction ::: `, transaction)
      transaction.feePayer = walletProvider?.publicKey;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);

      console.log(`sign ::: `, sign)
      // console.log(`x ::: `, x)
      console.log(walletProvider.TokenAccountsFilter)

      console.log(`PDA ::: `, PDA[0].toBase58())

      // console.log(
      //   `walletAccount ::: `,
      //   await connection.getTokenAccountsByOwner(new PublicKey("8Yx3Fo5Q6vbR9zourTBnpmBCydP71KmqfSg3dUFtUZ9D"), { mint: new PublicKey("HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o") })
      // )
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
          { pubkey: new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), isSigner: false, isWritable: true },
          { pubkey: new PublicKey("HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o"), isSigner: false, isWritable: true },
          { pubkey: new PublicKey("HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty"), isSigner: false, isWritable: true },
        ],
        programId,
        data
      });

      const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
      const signature = await signAndSendTransaction(walletProvider, transaction2);
      console.log(`signature :::`, signature)


      const accData = await connection.getAccountInfo(fundAccount);

      console.log(`accData :::: `, accData)
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

