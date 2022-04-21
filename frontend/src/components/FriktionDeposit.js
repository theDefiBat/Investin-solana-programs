import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState , useEffect} from 'react'
import { GlobalState } from '../store/globalState';
import { createAccountInstruction, createAssociatedTokenAccountIfNotExist, signAndSendTransaction } from '../utils/web3'
import { connection, programId, priceStateAccount, platformStateAccount, idsIndex, FUND_ACCOUNT_KEY, LIQUIDITY_POOL_PROGRAM_ID_V4 } from '../utils/constants';
import { struct, u8, nu64 } from 'buffer-layout';
import { TOKENS } from '../utils/tokens'
import { FUND_DATA, FUND_PDA_DATA, PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, DEV_TOKENS, pools, raydiumPools } from '../utils/pools';
import { IDS } from '@blockworks-foundation/mango-client';

export const FriktionDeposit = () => {
  
    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleFriktionDeposit = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()

      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

      //fcSRM
      const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), fundPDA[0], transaction);    
      const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt'), fundPDA[0], transaction);    
      const textEncoder = new TextEncoder();

      const pendingDepositsPDA = await PublicKey.findProgramAddress(
        [
          new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          fundPDA[0].toBuffer(),
          textEncoder.encode("pendingDeposit"),
        ],
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      );

      // const pendingDepositsPDAAccount = await connection.getAccountInfo(pendingDepositsPDA[0]);
  
  // if (pendingDepositsPDAAccount == null) {

  //     const pendingDepositsPDALamports =
  //   await connection.getMinimumBalanceForRentExemption(
  //     25,
  //     'singleGossip'
  //   )
  //   let signers = []
  //   await createAccountInstruction(connection, key, 25, new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'), pendingDepositsPDALamports, transaction, signers);
  // }

    
      console.log('pendingDepositPDA:: ', pendingDepositsPDA[0].toBase58());
     

      const dataLayout = struct([u8('instruction'), nu64('deposit_amount')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 24,
          deposit_amount: 1000000
        },
        data
      )

      const keys = [
        {pubkey: fundPDA[0], isSigner: false, isWritable: true},
        {pubkey: key, isSigner: true, isWritable: true },
        {pubkey: new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'), isSigner: false, isWritable: false},
        {pubkey: key, isSigner: true, isWritable: true},
        {pubkey: fundPDA[0], isSigner: false, isWritable: true},
        {pubkey: fundPDA[0], isSigner: false, isWritable: true},
        {pubkey: new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('2P427N5sYcEXvZAZwqNzjXEHsBMESQoLyjNquTSmGPMb'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('9e7XC1K2sPbDALCA7ZD8oxockHLe6KtXjEKhjEzqEGuj'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('JBPXzHLMDTN3BLACACcBHM1CAfNz2r1s6tUMUhCMgGCZ'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('HKMwJ27Zh5ySthKqyLniFDke99JSLsTYMgzHEfeSDgq2'), isSigner: false, isWritable: false},
        {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Vault_token_acc
        {pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true},//Fund_Underlying_token_acc
        {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
        {pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true}, //PendingDepositsPDA --fund
        {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false},
        
    ];
        const instruction = new TransactionInstruction({
        keys,
        programId,
        data
        });

        for(let i=0; i<keys.length;i++) {
            console.log("key:",i, keys[i].pubkey.toBase58())
        }
  
        transaction.add(instruction);
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
        console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
    }

    const handleFriktionDeposit0 = async () => {
    
      const key = walletProvider?.publicKey;

    if (!key ) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), key, transaction);    
    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt'), key, transaction);    
    const textEncoder = new TextEncoder();

    const pendingDepositsPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
        key.toBuffer(),
        textEncoder.encode("pendingDeposit"),
      ],
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    // const pendingDepositsPDAAccount = await connection.getAccountInfo(pendingDepositsPDA[0]);

// if (pendingDepositsPDAAccount == null) {

//     const pendingDepositsPDALamports =
//   await connection.getMinimumBalanceForRentExemption(
//     25,
//     'singleGossip'
//   )
//   let signers = []
//   await createAccountInstruction(connection, key, 25, new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'), pendingDepositsPDALamports, transaction, signers);
// }

  
    console.log('pendingDepositPDA:: ', pendingDepositsPDA[0].toBase58());
   

    const dataLayout = struct([u8('instruction'), nu64('deposit_amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 34,
        deposit_amount: 1000000
      },
      data
    )

    const keys = [
      {pubkey: fundPDA[0], isSigner: false, isWritable: true},
      {pubkey: key, isSigner: true, isWritable: true },
      {pubkey: new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'), isSigner: false, isWritable: false},
      {pubkey: key, isSigner: true, isWritable: true},
      {pubkey: key, isSigner: false, isWritable: true},
      {pubkey: key, isSigner: false, isWritable: true},
      {pubkey: new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('2P427N5sYcEXvZAZwqNzjXEHsBMESQoLyjNquTSmGPMb'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('9e7XC1K2sPbDALCA7ZD8oxockHLe6KtXjEKhjEzqEGuj'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('JBPXzHLMDTN3BLACACcBHM1CAfNz2r1s6tUMUhCMgGCZ'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('HKMwJ27Zh5ySthKqyLniFDke99JSLsTYMgzHEfeSDgq2'), isSigner: false, isWritable: false},
      {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Vault_token_acc
      {pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true},//Fund_Underlying_token_acc
      {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
      {pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true}, //PendingDepositsPDA --fund
      {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false},
      
  ];
      const instruction = new TransactionInstruction({
      keys,
      programId,
      data
      });

      for(let i=0; i<keys.length;i++) {
          console.log("key:",i, keys[i].pubkey.toBase58())
      }

      transaction.add(instruction);
      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
  }

    const handleFriktionGetData = async () => {

      const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()

      const dataLayout = struct([u8('instruction')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 33,
        },
        data
      )

      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const textEncoder = new TextEncoder();
      const pendingDepositsPDA = await PublicKey.findProgramAddress(
        [
          new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          fundPDA[0].toBuffer(),
          textEncoder.encode("pendingDeposit"),
        ],
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      );
      const textEncoder2 = new TextEncoder();
      const pendingWithdrawalsPDA = await PublicKey.findProgramAddress(
        [
          new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          fundPDA[0].toBuffer(),
          textEncoder2.encode("pendingWithdrawal"),
        ],
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      );

      const keys = [
        {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: false},
        {pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: false},
        {pubkey: pendingWithdrawalsPDA[0], isSigner: false, isWritable: false}
      ];

      const instruction = new TransactionInstruction({
        keys,
        programId,
        data
      });

      transaction.add(instruction);
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
        console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`)
    }

    return (
        <div className="form-div">
          <h4>Friktion Deposit</h4>

          <button onClick={handleFriktionDeposit}>Deposit</button>

          <h4>Friktion Deposit 0000</h4>

          <button onClick={handleFriktionDeposit0}>Deposit</button>


          <h4>Friktion Get Data</h4>

          <button onClick={handleFriktionGetData}>Get Data</button>

        </div>
      )
      
}