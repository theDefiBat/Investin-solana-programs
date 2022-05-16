import { PublicKey, SYSVAR_CLOCK_PUBKEY, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState, useEffect } from 'react'
import { GlobalState } from '../store/globalState';
import { createAccountInstruction, createAssociatedTokenAccountIfNotExist, signAndSendTransaction } from '../utils/web3'
import { connection, programId, RENT_PROGRAM_ID, platformStateAccount, idsIndex, FUND_ACCOUNT_KEY, LIQUIDITY_POOL_PROGRAM_ID_V4, TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID } from '../utils/constants';
import { struct, u8, nu64 } from 'buffer-layout';
import { TOKENS } from '../utils/tokens'
import { FRIKTION_VOLT, FUND_DATA, FUND_PDA_DATA, PLATFORM_DATA, PRICE_DATA, u64 } from '../utils/programLayouts';
import { devnet_pools, DEV_TOKENS, pools, raydiumPools } from '../utils/pools';
import { IDS } from '@blockworks-foundation/mango-client';
import { FriktionSDK, VoltSDK } from "@friktion-labs/friktion-sdk";
import { Card, Col, Row } from 'reactstrap';

const VOLT_PROGRAM_ID = 'VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'

const VOLT_SRM = {
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

const VOLT_fpSOLHigh = {
  "name": "SOL PUT",
  "voltVaultId": "BTuiZkgodmKKJtNDhVQGvAzqW1fdSNWasQADDTTyoAxN",
  "quoteMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "underlyingMint": "So11111111111111111111111111111111111111112",
  "depositTokenMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "shareTokenMint": "G8jsAWUA2KdDn7XmV1sBqUdbEXESaPdjPWDEYCsnkRX2",
  "vaultMint": "G8jsAWUA2KdDn7XmV1sBqUdbEXESaPdjPWDEYCsnkRX2",
  "shareTokenSymbol": "fpSOLHigh",
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

const client = new FriktionSDK({ provider: { connection: connection } });


export const FriktionDeposit = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const [voltDecoded, setVoltDecoded] = useState({})
  const [selectedVolt, setSelectedVolt] = useState({})
  const [pendingDeposits, setPendingDeposits] = useState({})
  const [pendingWithdraws, setPendingWithdraws] = useState({})

  const [friktionBalances, setFriktionBalances] = useState({})
  const [roundKey, setRoundKey] = useState({})

  useEffect(() => {
    (async () => {

      try {
        let selectedVolt = await client.loadVoltAndExtraDataByKey(new PublicKey(VOLT_fpSOLHigh.voltVaultId));
        console.log("selectedVolt VOLT_fpSOLHigh:", selectedVolt)
        setSelectedVolt(selectedVolt) //srm

        const roundkey = await selectedVolt.getCurrentRound();
        console.log("roundKey:", roundkey)
        setRoundKey(roundkey)

        // if (walletProvider && walletProvider?.publicKey) {
           const user = new PublicKey('JwV3M6PvMykzYeQznSeZA7WdwTvyaXCC4dFWoKvSYZS')
          const bal = await selectedVolt.getBalancesForUser(user) //(walletProvider?.publicKey)
          console.log("bal:", bal)
          setFriktionBalances(bal)

          // const allPendingDeposits = await selectedVolt.getAllPendingDeposits();
          // // allPendingDeposits[0].roundNumber
          // console.log("allPendingDeposits:",allPendingDeposits)

          const pendingDeposits = await selectedVolt.getPendingDepositForGivenUser(user) //(walletProvider?.publicKey);
          console.log("pendingDeposits:", pendingDeposits)
          setPendingDeposits(pendingDeposits)

          const key = (
            await VoltSDK.findPendingWithdrawalInfoAddress(
              new PublicKey(VOLT_SRM.voltVaultId),
              user,
              //(walletProvider?.publicKey),
              new PublicKey(VOLT_PROGRAM_ID)
            )
          )[0];

          const pendingWithdraws = await selectedVolt.getPendingWithdrawalByKey(key)
          console.log("pendingWithdraws:", pendingWithdraws)
          setPendingWithdraws(pendingWithdraws)
        // }

        await handleGetFriktionDataWeb3();
      } catch (error) {
        console.log("error:", error)
      }
    })()
  }, [walletProvider])

  const handleGetFriktionDataWeb3 = async () => {
    console.log("handleGetFriktionDataWeb3 of :", VOLT_fpSOLHigh.voltVaultId)
    const vault_key = new PublicKey(VOLT_fpSOLHigh.voltVaultId);
    const friktionDataAcc = await connection.getAccountInfo(vault_key);
    console.log("FriktionDataAccount:: ", friktionDataAcc);
    if (friktionDataAcc) {
      const friktionData = FRIKTION_VOLT.decode(friktionDataAcc.data);
      console.error("FriktionData ::", friktionData);
      setVoltDecoded(friktionData);

      if (walletProvider && walletProvider?.publicKey) {
        const bal = await selectedVolt.getBalancesForUser(walletProvider?.publicKey)
        console.log("bal:", bal);
        setFriktionBalances(bal);
      }

      // for (const [key, value] of Object.entries(friktionData)) {
      //   try {
      //     console.log(`${key}: ${value.toBase58()}`);
      //   } catch (error) {
      //     console.log("e:", `${key}: ${value.toString()}`)
      //   }
      // }

    }
  }

  const handleFriktionGetDataOnContract = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
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
      { pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: false },
      { pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: false },
      { pubkey: pendingWithdrawalsPDA[0], isSigner: false, isWritable: false }
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
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleFriktionDeposit = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);

    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.depositTokenMint), fundPDA[0], transaction);
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

    // const volt = new VoltSDK(FriktionSDK, selectedVolt, );

    // const ExtraVoltDataAddress = await selectedVolt.findExtraVoltDataAddress(selectedVolt.voltKey)
    const [extraVoltKey] = await VoltSDK.findExtraVoltDataAddress(selectedVolt.voltKey);
    const [roundInfoKey, roundInfoKeyBump] = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const roundVoltTokensAddress = (
      await VoltSDK.findRoundVoltTokensAddress(
        selectedVolt.voltKey,
        selectedVolt.voltVault.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    )[0];

    const roundUnderlyingTokensAddress = (
      await VoltSDK.findRoundUnderlyingTokensAddress(
        selectedVolt.voltKey,
        selectedVolt.voltVault.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    )[0];

    const [epochInfoKey, epochInfoBump] = await VoltSDK.findEpochInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    console.log('pendingDepositPDA:: ', pendingDepositsPDA[0].toBase58());


    const dataLayout = struct([u8('instruction'), nu64('deposit_amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 34,
        deposit_amount: 100000
      },
      data
    )

    const keys = [
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },


      { pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true },
      // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },

      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: extraVoltKey, isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: selectedVolt.voltVault.depositPool, isSigner: false, isWritable: true },
      { pubkey: selectedVolt.voltVault.writerTokenPool, isSigner: false, isWritable: false },
      { pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true },//Fund_Vault_token_acc
      { pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true },//Fund_Underlying_token_acc
      { pubkey: roundInfoKey, isSigner: false, isWritable: true },
      { pubkey: roundVoltTokensAddress, isSigner: false, isWritable: true },
      { pubkey: roundUnderlyingTokensAddress, isSigner: false, isWritable: true },

      // {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
      { pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true }, //PendingDepositsPDA --fund

      { pubkey: epochInfoKey, isSigner: false, isWritable: true },
      // {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},

      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false },

    ];
    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleFriktionInvestorWithdrawUL = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);

    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.depositTokenMint), fundPDA[0], transaction);
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

    // const volt = new VoltSDK(FriktionSDK, selectedVolt, );

    // const ExtraVoltDataAddress = await selectedVolt.findExtraVoltDataAddress(selectedVolt.voltKey)
    const [extraVoltKey] = await VoltSDK.findExtraVoltDataAddress(selectedVolt.voltKey);
    const [roundInfoKey, roundInfoKeyBump] = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const roundVoltTokensAddress = (
      await VoltSDK.findRoundVoltTokensAddress(
        selectedVolt.voltKey,
        selectedVolt.voltVault.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    )[0];

    const roundUnderlyingTokensAddress = (
      await VoltSDK.findRoundUnderlyingTokensAddress(
        selectedVolt.voltKey,
        selectedVolt.voltVault.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    )[0];

    const [epochInfoKey, epochInfoBump] = await VoltSDK.findEpochInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    console.log('pendingDepositPDA:: ', pendingDepositsPDA[0].toBase58());


    const dataLayout = struct([u8('instruction'), nu64('deposit_amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 43,
        deposit_amount: 100000
      },
      data
    )

    const keys = [
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: true, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },


      { pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true },
      // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true},
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },

      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: extraVoltKey, isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: selectedVolt.voltVault.depositPool, isSigner: false, isWritable: true },
      { pubkey: selectedVolt.voltVault.writerTokenPool, isSigner: false, isWritable: false },
      { pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true },//Fund_Vault_token_acc
      { pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true },//Fund_Underlying_token_acc
      { pubkey: roundInfoKey, isSigner: false, isWritable: true },
      { pubkey: roundVoltTokensAddress, isSigner: false, isWritable: true },
      { pubkey: roundUnderlyingTokensAddress, isSigner: false, isWritable: true },

      // {pubkey: new PublicKey('6yphtPNxWnESktG8zmk1GpD7GgjR37WerxtoWdqedXbX'), isSigner: false, isWritable: true},
      // {pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true},
      { pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true }, //PendingDepositsPDA --fund

      { pubkey: epochInfoKey, isSigner: false, isWritable: true },
      // {pubkey: new PublicKey('Gnz3cwbgh6vHH9EMVp3f36Pvs6rkeD14ayoJFyptGEA4'), isSigner: false, isWritable: true},

      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ];
    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleUpdateFriktionValue = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);
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
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const [epochInfoKey, epochInfoBump] = await VoltSDK.findEpochInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    console.log('pendingDepositPDA:: ', pendingDepositsPDA[0].toBase58());


    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 40
      },
      data
    )

    const keys = [
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },
      { pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true }, //PendingDepositsPDA --fund
      { pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true },//Fund_Vault_token_acc
      { pubkey: pendingWithdrawalsPDA[0], isSigner: false, isWritable: true },
      { pubkey: epochInfoKey, isSigner: false, isWritable: true },

    ];
    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleFriktionWithdraw = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);
    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.depositTokenMint), fundPDA[0], transaction);
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

    const [extraVoltKey] = await VoltSDK.findExtraVoltDataAddress(selectedVolt.voltKey);
    const [roundInfoKey, roundInfoKeyBump] = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const roundUnderlyingTokensAddress = (
      await VoltSDK.findRoundUnderlyingTokensAddress(
        selectedVolt.voltKey,
        selectedVolt.voltVault.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    )[0];

    const [epochInfoKey, epochInfoBump] = await VoltSDK.findEpochInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );
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
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp'), isSigner: false, isWritable: false },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true }, //vault mint

      // {pubkey: new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY'), isSigner: false, isWritable: true}, //volt_vault
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },

      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: extraVoltKey, isSigner: false, isWritable: false }, //extra_volt_data
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false }, //whitelist
      { pubkey: selectedVolt.voltVault.depositPool, isSigner: false, isWritable: true }, //deposit_pool
      { pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true },//Fund_Underlying_token_acc
      { pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true },//Fund_Vault_token_acc
      { pubkey: roundInfoKey, isSigner: false, isWritable: true },
      { pubkey: roundUnderlyingTokensAddress, isSigner: false, isWritable: true },
      { pubkey: pendingWithdrawalsPDA[0], isSigner: false, isWritable: true }, //PendingDepositsPDA --fund
      { pubkey: epochInfoKey, isSigner: false, isWritable: true },
      { pubkey: new PublicKey('FhrcvL91UwgVpbMmpmyx3GTPUsuofWpjRGBdpV34ern2'), isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable: false },

    ];

    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleFriktionCancelPendingDeposit = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('DNa849drqW19uBV5X9ohpJ5brRGzq856gk3HDRqveFrA'), fundPDA[0], transaction);
    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'), fundPDA[0], transaction);
    const textEncoder = new TextEncoder();

    const pendingDepositsPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_fpSOLHigh.voltVaultId).toBuffer(),
        fundPDA[0].toBuffer(),
        textEncoder.encode("pendingDeposit"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );


    const extraVoltsDataPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_fpSOLHigh.voltVaultId).toBuffer(),
        textEncoder.encode("extraVoltData"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );

    const [roundInfoKey, roundInfoKeyBump] = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const roundUnderlyingTokensAddress = (
      await VoltSDK.findRoundUnderlyingTokensAddress(
        selectedVolt.voltKey,
        selectedVolt.voltVault.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    )[0];

    const [epochInfoKey, epochInfoBump] = await VoltSDK.findEpochInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
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
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VOLT_fpSOLHigh.shareTokenMint), isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VOLT_fpSOLHigh.voltVaultId), isSigner: false, isWritable: true },
      { pubkey: extraVoltsDataPDA[0], isSigner: false, isWritable: false },

      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true },//Fund_Underlying_token_acc
      // {pubkey: roundInfoPDA[0], isSigner: false, isWritable: false},CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ
      { pubkey: roundInfoKey, isSigner: false, isWritable: true },
      {pubkey: roundUnderlyingTokensAddress, isSigner: false, isWritable: true},
      // { pubkey: new PublicKey('8mSbc6sVm7xPCW23mk2UMXxtoYuMTWgXmo9itnxQsLXH'), isSigner: false, isWritable: true },
      { pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true }, //PendingDepositsPDA --fund
      { pubkey: epochInfoKey, isSigner: false, isWritable: true },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false },
      { pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable: false }

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

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleFriktionCancelPendingWithdrawal = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);

    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.depositTokenMint), fundPDA[0], transaction);
    const textEncoder = new TextEncoder();

    const pendingWithdrawalPDA = await PublicKey.findProgramAddress(
      [
        new PublicKey(VOLT_SRM.voltVaultId).toBuffer(),
        fundPDA[0].toBuffer(),
        textEncoder.encode("pendingWithdrawal"),
      ],
      new PublicKey(VOLT_PROGRAM_ID)
    );


    const [extraVoltKey] = await VoltSDK.findExtraVoltDataAddress(selectedVolt.voltKey);


    const roundInfoPDA = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    )

    const roundUnderlyingTokensPDA = await VoltSDK.findRoundUnderlyingTokensAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    )

    const [epochInfoKey, epochInfoBump] = await VoltSDK.findEpochInfoAddress(
      selectedVolt.voltKey,
      selectedVolt.voltVault.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
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
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VOLT_SRM.shareTokenMint), isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VOLT_SRM.voltVaultId), isSigner: false, isWritable: true },
      { pubkey: extraVoltKey, isSigner: false, isWritable: false },

      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true },//Fund_Underlying_token_acc
      // {pubkey: roundInfoPDA[0], isSigner: false, isWritable: false},CZBmqRPuPhTkd56kH3JQbsnPK4jCUnCBeF7EfLPPZaoJ
      { pubkey: roundInfoPDA[0], isSigner: false, isWritable: true },
      { pubkey: pendingWithdrawalPDA[0], isSigner: false, isWritable: true }, //pendingWithdrawalPDA --fund
      { pubkey: epochInfoKey, isSigner: false, isWritable: true },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
      { pubkey: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'), isSigner: false, isWritable: false },
      { pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable: false }

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

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleFriktionClaimPendingDeposit = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);

    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.depositTokenMint), fundPDA[0], transaction);
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


    const pendingDepositsPDAData = await selectedVolt.getPendingDepositByKey(pendingDepositsPDA[0]);
    console.log('x ::: ', pendingDepositsPDAData)

    const [extraVoltKey] = await VoltSDK.findExtraVoltDataAddress(selectedVolt.voltKey);

    const [pendingDepositRoundInfoKey, roundInfoKeyBump] = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      pendingDepositsPDAData.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const [PendingDepositRoundVoltTokens, pdrvtabump] = (
      await VoltSDK.findRoundVoltTokensAddress(
        selectedVolt.voltKey,
        pendingDepositsPDAData.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    );

    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 38,
        deposit_amount: 1000000
      },
      data
    )

    const keys = [
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },
      { pubkey: extraVoltKey, isSigner: false, isWritable: false },
      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: associatedTokenAddressfcSRM, isSigner: false, isWritable: true },
      { pubkey: pendingDepositRoundInfoKey, isSigner: false, isWritable: true },
      { pubkey: PendingDepositRoundVoltTokens, isSigner: false, isWritable: true },
      { pubkey: pendingDepositsPDA[0], isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    ];

    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    })

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)

  }

  const handleFriktionClaimPendingWithdrawal = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);

    const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.depositTokenMint), fundPDA[0], transaction);
    const textEncoder = new TextEncoder();

    const pendingWithdrawalPDA = await PublicKey.findProgramAddress(
      [
        // new PublicKey('Ef2CD9yhQE7BvReQXct68uuYFW8GLKj62u2YPfmua3JY').toBuffer(),
        (selectedVolt.voltKey).toBuffer(),
        fundPDA[0].toBuffer(),
        textEncoder.encode("pendingWithdrawal"),
      ],
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );


    const pendingWithdrawalPDAdata = await selectedVolt.getPendingWithdrawalByKey(pendingWithdrawalPDA[0]);
    console.log('x ::: ', pendingWithdrawalPDAdata)

    const [extraVoltKey] = await VoltSDK.findExtraVoltDataAddress(selectedVolt.voltKey);

    const [pendingWithdrawalRoundInfoKey, roundInfoKeyBump] = await VoltSDK.findRoundInfoAddress(
      selectedVolt.voltKey,
      pendingWithdrawalPDAdata.roundNumber,
      new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
    );

    const [RoundUnderlyingPendingWithdrawalsAddress, pdrvtabump] = (
      await VoltSDK.findRoundUnderlyingPendingWithdrawalsAddress(
        selectedVolt.voltKey,
        pendingWithdrawalPDAdata.roundNumber,
        new PublicKey('VoLT1mJz1sbnxwq5Fv2SXjdVDgPXrb9tJyC8WpMDkSp')
      )
    );

    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 39
      },
      data
    )

    const keys = [
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },
      { pubkey: extraVoltKey, isSigner: false, isWritable: false },
      { pubkey: selectedVolt.voltVault.vaultAuthority, isSigner: false, isWritable: false },
      { pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true },
      { pubkey: associatedTokenAddressSRM, isSigner: false, isWritable: true },
      { pubkey: pendingWithdrawalRoundInfoKey, isSigner: false, isWritable: true },
      { pubkey: pendingWithdrawalPDA[0], isSigner: false, isWritable: true },
      { pubkey: RoundUnderlyingPendingWithdrawalsAddress, isSigner: false, isWritable: true },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    ];

    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    })

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)

  }

  const handleAddFriktionToFund = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    // const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);
    
    // const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt'), fundPDA[0], transaction);    
    // const textEncoder = new TextEncoder();


    const dataLayout = struct([u8('instruction'), u8('ul_token_slot')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 41,
        ul_token_slot: 0
      },
      data
    )

    const keys = [
      { pubkey: platformStateAccount, isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      // {pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true},
    ];
    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }

  const handleRemoveFriktionToFund = async () => {

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    //fcSRM
    // const associatedTokenAddressfcSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(VOLT_fpSOLHigh.shareTokenMint), fundPDA[0], transaction);
    
    // const associatedTokenAddressSRM = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt'), fundPDA[0], transaction);    
    // const textEncoder = new TextEncoder();


    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 42,
      },
      data
    )

    const keys = [
      // { pubkey: platformStateAccount, isSigner: true, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      // { pubkey: selectedVolt.voltKey, isSigner: false, isWritable: true },
      // { pubkey: new PublicKey(VOLT_PROGRAM_ID), isSigner: false, isWritable: false },
      // {pubkey: selectedVolt.voltVault.vaultMint, isSigner: false, isWritable: true},
    ];
    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

    for (let i = 0; i < keys.length; i++) {
      console.log("key:", i, keys[i].pubkey.toBase58())
    }

    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://explorer.solana.com/tx/${sign}`)
  }


  return (
    <div className="form-div">
      <Card className="justify-content-center">
        <Row className="justify-content-between">
          <Col lg="6" xs="6">
            <h4>Friktion Deposit </h4>
            <button onClick={handleFriktionDeposit}>Deposit</button>

            {/* <h4>Friktion CancelPendingDeposit</h4> */}
            <button onClick={handleFriktionCancelPendingDeposit}>CancelPendingDeposit</button>

            {/* <h4>Friktion ClaimPendingDeposit</h4> */}
            <button onClick={handleFriktionClaimPendingDeposit}>ClaimPendingDeposit-STEP-1</button>

            <br /><br />

            <h4>Friktion Withdraw</h4>
            <button onClick={handleFriktionWithdraw}>Withdraw</button>

            {/* <h4>Friktion CancelPendingWithdrawal</h4> */}
            <button onClick={handleFriktionCancelPendingWithdrawal}>CancelPendingWithdrawal</button>

            {/* <h4>Friktion CLAIMPendingWithdrawal</h4> */}
            <button onClick={handleFriktionClaimPendingWithdrawal}>CLAIMPendingWithdrawal-STEP-1</button>
            <br /><br />

            <h4>Friktion Get Data</h4>
            <button onClick={handleUpdateFriktionValue}>UPDATE FRIKTION VALUE FOR FUND-STEP-2</button> <br />
            <button onClick={handleFriktionGetDataOnContract}> Read Data on contract</button>
            <button onClick={handleGetFriktionDataWeb3}>GET FRIKTION VAULT DATA WEB3</button>

            <br /><br />
            <button onClick={handleAddFriktionToFund}>ADD to Fund</button>
            <button onClick={handleRemoveFriktionToFund}>REMOVE from Fund</button>
          </Col>

          <Col lg="6" xs="6">
            <h4>Friktion Data</h4>
            <p> current epoch - volt_vault.round_info {roundKey?.number?.toString()}</p>
            <p> deposit epoch - pending_deposit.round_info {pendingDeposits?.roundNumber?.toString()}</p>
            <p> withdraw epoch - pending_withdraw.round_info {pendingWithdraws?.roundNumber?.toString()}</p>

            <p> claimableUnderlying {friktionBalances?.claimableUnderlying?.toString()}</p>
            <p> mintableShares {friktionBalances?.mintableShares?.toString()}</p>

            <p> pendingDeposit {friktionBalances?.pendingDeposits?.toString()}</p>
            <p> pendingWithdraw {friktionBalances?.pendingWithdrawals?.toString()}</p>

          </Col>
        </Row>
      </Card>

    </div>
  )

}