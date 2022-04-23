import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState , useEffect} from 'react'
import { GlobalState } from '../store/globalState';
import { createAccountInstruction, createAssociatedTokenAccountIfNotExist, signAndSendTransaction } from '../utils/web3'
import { connection, programId, RENT_PROGRAM_ID, platformStateAccount, idsIndex, FUND_ACCOUNT_KEY, LIQUIDITY_POOL_PROGRAM_ID_V4, TOKEN_PROGRAM_ID } from '../utils/constants';
import { struct, u8, nu64 } from 'buffer-layout';
import { TOKENS } from '../utils/tokens'
import { FRIKTION_VOLT, FUND_DATA, FUND_PDA_DATA, PLATFORM_DATA, PRICE_DATA, u64 } from '../utils/programLayouts';
import { devnet_pools, DEV_TOKENS, pools, raydiumPools } from '../utils/pools';
import { IDS } from '@blockworks-foundation/mango-client';
import { FriktionSDK } from "@friktion-labs/friktion-sdk";

const VOLT_PROGRAM_ID = 'VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'

const VOLT_SRM =  {
  "name": "SRM CALL",
  "voltVaultId": "Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY",
  "quoteMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "underlyingMint": "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt",
  "depositTokenMint": "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt",
  "shareTokenMint": "5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X",
  "vaultMint": "5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X",
  "shareTokenSymbol": "fcSRM",
  "shareTokenDecimals": 6,
  "voltType": 1,
}
// vaultAuthority: 2P427N5sYcEXvZAZwqNzjXEHsBMESQoLyjNquTSmGPMb
// FriktionDeposit :::   depositPool: JBPXzHLMDTN3BLACACcBHM1CAfNz2r1s6tUMUhCMgGCZ
// FriktionDeposit.js:52 premiumPool: A2sXsMsnAbtuk4RhxVc5ZbJq6s6SPVSLPWJzkHNrqGXm
// FriktionDeposit.js:52 optionPool: DbLdWdksiqPPiXX6o1WfWwCTe4vLRBUhaZmxtPpRcQpB
// FriktionDeposit.js:52 writerTokenPool: HKMwJ27Zh5ySthKqyLniFDke99JSLsTYMgzHEfeSDgq2
// FriktionDeposit.js:52 vaultMint: 5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X
// FriktionDeposit.js:52 underlyingAssetMint: SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt
// FriktionDeposit.js:52 quoteAssetMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
// FriktionDeposit.js:52 optionMint: Bf5QZCctXHVpqMpKJfpzKk5Vpom4esUcjNZgbWP4SYPS
// FriktionDeposit.js:52 writerTokenMint: 9JS8ycroX1RFjixVCt3kSH6jJ6kGJvLie4URrd6jp58R
// FriktionDeposit.js:52 optionMarket: FJZScRRHk9g8DHj6L1k9LNzGf8sesKvYx55CVVoNWMTk
// FriktionDeposit.js:54 
//                   adminKey: DxMJgeSVoe1cWo1NPExiAsmn83N3bADvkT86dSP1k7WE
// FriktionDeposit.js:52 seed: 2cABXMpMtjn7x34qt9VaneFLPrYPA4aJPq1amjZVp9Bn
//                   whitelistTokenMint: mmmFXxpwDfkPFVKt1Js8fU6DvjEMXU8tprvaTpmn8sM
// FriktionDeposit.js:52 permissionedMarketPremiumMint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
// FriktionDeposit.js:52 permissionedMarketPremiumPool: H4qUyG47mzqNv75fSBjSE59V9nRvxtvidGsLsbjJMZPV



export const FriktionDeposit = () => {
  
    const walletProvider = GlobalState.useState(s => s.walletProvider);
    const [voltDecoded, setVoltDecoded] = useState({})
    const [selectedVolt, setSelectedVolt] = useState({})

    useEffect(() => {
      (async () => {
         await handleGetFriktionData();
         const client = new FriktionSDK({ provider: { connection: connection } });

         const vaults = await client.getAllVoltVaults()
         console.log("vaults :",vaults)

         try {
          for (let i=0 ;i<=vaults.length; i++) {
            console.log("volt-"+i,vaults[i].voltKey.toBase58(), vaults[i].voltVault.underlyingAssetMint.toBase58())
          }
         } catch (error) {
           console.log("error:",error)
         }
         

         setSelectedVolt(vaults[5]) //srm
          
         const data = await client.loadVoltAndExtraDataByKey(vaults[0].voltKey);
         console.log('FriktionDeposit data :>> ', await data.getAllRounds());
     
      })()
     }, [])

    const handleGetFriktionData = async () => {
      console.log("handleGetFriktionData of :",VOLT_SRM.voltVaultId)
      const vault_key = new PublicKey(VOLT_SRM.voltVaultId);
      const friktionDataAcc = await connection.getAccountInfo(vault_key);
        console.log("FriktionDataAccount:: ",friktionDataAcc);
        if (friktionDataAcc) {
          const friktionData = FRIKTION_VOLT.decode(friktionDataAcc.data);
          console.error("FriktionData ::",friktionData);
          setVoltDecoded(friktionData);

          // for (const [key, value] of Object.entries(friktionData)) {
          //   try {
          //     console.log(`${key}: ${value.toBase58()}`);
          //   } catch (error) {
          //     console.log("e:", `${key}: ${value.toString()}`)
          //   }
          // }

        }
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
          // new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          (selectedVolt.voltKey).toBuffer(),
          fundPDA[0].toBuffer(),
          textEncoder.encode("pendingDeposit"),
        ],
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      );
      const pendingWithdrawalsPDA = await PublicKey.findProgramAddress(
        [
          // new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          (selectedVolt.voltKey).toBuffer(),
          fundPDA[0].toBuffer(),
          textEncoder.encode("pendingWithdrawal"),
        ],
        new PublicKey(VOLT_PROGRAM_ID)
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
          // new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          (selectedVolt.voltKey).toBuffer(),
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
          instruction: 34,
          deposit_amount: 1000000
        },
        data
      )

      const keys = [
        {pubkey: fundPDA[0], isSigner: false, isWritable: true},
        {pubkey: key, isSigner: true, isWritable: true },
        {pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false},
        {pubkey: key, isSigner: true, isWritable: true},
        {pubkey: fundPDA[0], isSigner: false, isWritable: true},
        {pubkey: fundPDA[0], isSigner: false, isWritable: true},


        {pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true},
        // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
        {pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true},

        {pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false},
        {pubkey: selectedVolt.findExtraVoltDataAddress(selectedVolt.voltKey)[0], isSigner: false, isWritable: false},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: selectedVolt.voltVault.depositPool , isSigner: false, isWritable: true},
        {pubkey: selectedVolt.voltVault.writerTokenPool, isSigner: false, isWritable: false},
        {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Vault_token_acc
        {pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true},//Fund_Underlying_token_acc
        {pubkey: selectedVolt.findRoundInfoAddress(selectedVolt.voltKey, selectedVolt.voltVault.roundNumber ,new PublicKey(VOLT_PROGRAM_ID)), isSigner: false, isWritable: true},
        {pubkey: selectedVolt.findRoundVoltTokensAddress(selectedVolt.voltKey, selectedVolt.voltVault.roundNumber ,new PublicKey(VOLT_PROGRAM_ID)), isSigner: false, isWritable: true},
        {pubkey: selectedVolt.findRoundUnderlyingTokensAddress(selectedVolt.voltKey, selectedVolt.voltVault.roundNumber ,new PublicKey(VOLT_PROGRAM_ID)), isSigner: false, isWritable: true},
       
        // {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
        // {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
        {pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true}, //PendingDepositsPDA --fund

        {pubkey: selectedVolt.findEpochInfoAddress(selectedVolt.voltKey, selectedVolt.voltVault.roundNumber ,new PublicKey(VOLT_PROGRAM_ID)), isSigner: false, isWritable: true},
        // {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
        
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

    const handleFriktionWithdraw= async () => {
      
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

      const pendingWithdrawalsPDA = await PublicKey.findProgramAddress(
        [
          // new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
          (selectedVolt.voltKey).toBuffer(),
          fundPDA[0].toBuffer(),
          textEncoder.encode("pendingWithdrawal"),
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

    
      console.log('pendingDepositPDA:: ', pendingWithdrawalsPDA[0].toBase58());
    

      const dataLayout = struct([u8('instruction'), nu64('deposit_amount')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 36,
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
        {pubkey: new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), isSigner: false, isWritable: true}, //vault mint

        // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true}, //volt_vault
        {pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true},

        {pubkey: new PublicKey('2P427N5sYcEXvZAZwqNzjXEHsBMESQoLyjNquTSmGPMb'), isSigner: false, isWritable: false}, //vault_authority
        {pubkey: new PublicKey('9e7XC1K2sPbDALCA7ZD8oxockHLe6KtXjEKhjEzqEGuj'), isSigner: false, isWritable: false}, //extra_volt_data
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false}, //whitelist
        {pubkey: new PublicKey('JBPXzHLMDTN3BLACACcBHM1CAfNz2r1s6tUMUhCMgGCZ'), isSigner: false, isWritable: true}, //deposit_pool
        {pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true},//Fund_Underlying_token_acc
        {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Vault_token_acc
        {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
        {pubkey: pendingWithdrawalsPDA[0], isSigner: false, isWritable: true}, //PendingDepositsPDA --fund
        {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('FhrcvL91UwgVpbMmpmyx3GTPUsuofWpjRGBdpV34ern2'), isSigner: false, isWritable: true},
        {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
        {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
        {pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable: false},
        
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

  const handleFriktionCancelPendingDeposit = async () => {
    
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
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        fundPDA[0].toBuffer(),
        textEncoder.encode("pendingDeposit"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );


    const extraVoltsDataPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        textEncoder.encode("extraVoltData"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

    const roundInfoPDA =  await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        // voltDecoded.roundNumber,
        new u64(voltDecoded.roundNumber.toString()).toBuffer(),
        textEncoder.encode("roundInfo"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

    const roundUnderlyingTokensPDA =  await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        voltDecoded.roundNumber,
        textEncoder.encode("roundUnderlyingTokens"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

  
    console.log('pendingDepositPDA:: ', pendingDepositsPDA[0].toBase58());
    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 35,
      },
      data
    )

    const keys = [
      {pubkey: fundPDA[0], isSigner: false, isWritable: true},
      {pubkey: key, isSigner: true, isWritable: true },
      {pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false},
      {pubkey: fundPDA[0], isSigner: false, isWritable: true},
      {pubkey: new PublicKey(VOLT_SRM.shareTokenMint), isSigner: false, isWritable: true},
      {pubkey: new PublicKey(VOLT_SRM.voltVaultId), isSigner: false, isWritable: true},
      {pubkey: extraVoltsDataPDA[0], isSigner: false, isWritable: false},

      {pubkey: voltDecoded.vaultAuthority, isSigner: false, isWritable: false},
      {pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true},//Fund_Underlying_token_acc
      // {pubkey: roundInfoPDA[0], isSigner: false, isWritable: false},CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ
      {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
      // {pubkey: roundUnderlyingTokensPDA[0], isSigner: false, isWritable: false},
      {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
      {pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true}, //PendingDepositsPDA --fund
      {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false},
      {pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable: false}
      
      // {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Vault_token_acc
      // {pubkey: new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('2P427N5sYcEXvZAZwqNzjXEHsBMESQoLyjNquTSmGPMb'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('9e7XC1K2sPbDALCA7ZD8oxockHLe6KtXjEKhjEzqEGuj'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('JBPXzHLMDTN3BLACACcBHM1CAfNz2r1s6tUMUhCMgGCZ'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('HKMwJ27Zh5ySthKqyLniFDke99JSLsTYMgzHEfeSDgq2'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
    ];

    for(let i=0; i<keys.length;i++) {
      console.log("key:",i, keys[i].pubkey.toBase58())
    }

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

  const handleFriktionCancelPendingWithdrawal = async () => {
    
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

    const pendingWithdrawalPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        fundPDA[0].toBuffer(),
        textEncoder.encode("pendingWithdrawal"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );


    const extraVoltsDataPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        textEncoder.encode("extraVoltData"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

    const roundInfoPDA =  await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        voltDecoded.roundNumber,
        textEncoder.encode("roundInfo"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

    const roundUnderlyingTokensPDA =  await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        voltDecoded.roundNumber,
        textEncoder.encode("roundUnderlyingTokens"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

  
    console.log('pendingDepositPDA:: ', pendingWithdrawalPDA[0].toBase58());
    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 37,
      },
      data
    )

    const keys = [
      {pubkey: fundPDA[0], isSigner: false, isWritable: true},
      {pubkey: key, isSigner: true, isWritable: true },
      {pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false},
      {pubkey: fundPDA[0], isSigner: false, isWritable: true},
      {pubkey: new PublicKey(VOLT_SRM.shareTokenMint), isSigner: false, isWritable: true},
      {pubkey: new PublicKey(VOLT_SRM.voltVaultId), isSigner: false, isWritable: true},
      {pubkey: extraVoltsDataPDA[0], isSigner: false, isWritable: false},

      {pubkey: voltDecoded.vaultAuthority, isSigner: false, isWritable: false},
      {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Underlying_token_acc
      // {pubkey: roundInfoPDA[0], isSigner: false, isWritable: false},CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ
      {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
      {pubkey: pendingWithdrawalPDA[0], isSigner: false, isWritable: true}, //pendingWithdrawalPDA --fund
      {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
      {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      {pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false},
      {pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable: false}
      
      // {pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true},//Fund_Vault_token_acc
      // {pubkey: new PublicKey('5SLqZSywodLS8ih6U2AAioZrxpgR149hR8SApmCB7r5X'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('2P427N5sYcEXvZAZwqNzjXEHsBMESQoLyjNquTSmGPMb'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('9e7XC1K2sPbDALCA7ZD8oxockHLe6KtXjEKhjEzqEGuj'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('JBPXzHLMDTN3BLACACcBHM1CAfNz2r1s6tUMUhCMgGCZ'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('HKMwJ27Zh5ySthKqyLniFDke99JSLsTYMgzHEfeSDgq2'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
      // {pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false},
    ];

    for(let i=0; i<keys.length;i++) {
      console.log("key:",i, keys[i].pubkey.toBase58())
    }

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



    return (
        <div className="form-div">

          <h4>Friktion Deposit 0000</h4>
          <button onClick={handleFriktionDeposit}>Deposit</button>

          <h4>Friktion CancelPendingDeposit</h4>
          <button onClick={handleFriktionCancelPendingDeposit}>CancelPendingDeposit</button>

          <h4>Friktion Withdraw</h4>
          <button onClick={handleFriktionWithdraw}>Withdraw</button>

          <h4>Friktion CancelPendingWithdrawal</h4>
          <button onClick={handleFriktionCancelPendingWithdrawal}>CancelPendingWithdrawal</button>

          <h4>Friktion Get Data</h4>

          <button onClick={handleFriktionGetData}>Get Data</button>

          <button onClick={handleGetFriktionData}>GET FRIKTION VAULT DATA</button>


        </div>
      )
      
}