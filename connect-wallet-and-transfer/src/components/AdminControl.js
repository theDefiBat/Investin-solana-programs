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

    const [Platform_is_initialized, setPlatform_is_initialized] = useState(false);
    const [Fund_is_initialized, setFund_is_initialized] = useState(false);
    const [Fund_min_amount, setFund_min_amount] = useState(0);
    const [Fund_min_return, setFund_min_return] = useState(0);
    const [Fund_performance_fee_perc, setFund_performance_fee_perc] = useState(0);

    const [fundPDA, setFundPDA] = useState('');
    const [fundStateAccount, setFundStateAccount] = useState('');
    const [funds, setFunds] = useState([]);

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleDeposit = async () => {

        const key = walletProvider?.publicKey;
    
        if (!key) {
          alert("connect wallet")
          return;
        };

        const transaction = new Transaction();
        const platformAccount = platformStateAccount;

        const dataLayout = struct([u8('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 6,
        platform_is_initialized: Platform_is_initialized,
        fund_is_initialized: Fund_is_initialized,
        fund_min_amount: Fund_min_amount * ( 10 ** TEST_TOKENS['USDR'].decimals),
        fund_min_return: Fund_min_return * 10000,
        fund_performance_fee_percentage: Fund_performance_fee_percentage * 10000
      },
      data
    )

    // DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w
    // HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o
    // HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: platformAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true }, //fund State Account
        //call if first time settig platform data
        // { pubkey: new PublicKey(TEST_TOKENS['USDR'].mintAddress), isSigner: false, isWritable: true },
      ],
      programId,
      data
    });

    const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
    const signature = await signAndSendTransaction(walletProvider, transaction2);
    let result = await connection.confirmTransaction(signature, "confirmed");
    console.log("tx:: ", signature)
    }

    const handleFunds = async () => {
        let managers = []
        const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
        const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
        console.log("platformData :: ", platformData)
      
        for(let i=0; i<platformData.no_of_active_funds; i++) {
          let manager = platformData.fund_managers[i];
          let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
          let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);
          
          managers.push({
            fundPDA: PDA[0].toBase58(),
            fundManager: manager.toBase58(),
            fundStateAccount: fundState.toBase58()
          });
        }
        console.log(managers)
        setFunds(managers);
      }
    
      const handleFundSelect = async(event) => {
      
        setFundPDA(event.target.value);
        funds.forEach(fund => {
          if (fund.fundPDA == event.target.value) 
          {setFundStateAccount(fund.fundStateAccount)
           console.log("set fundStateAcoount")}
        });
        console.log(`setting fundPDA :::: `, fundPDA)
        console.log(`setting fundStateAccount :::: `, fundStateAccount)
      }
    
      return (
        <div className="form-div">
          <h4>Admin Control</h4>
          Platform_is_initialized ::: {' '}
          <input type="number" value={Platform_is_initialized} onChange={(event) => setPlatform_is_initialized(event.target.value)} />
          <br />
          Fund_is_initialized ::: {' '}
          <input type="number" value={Fund_is_initialized} onChange={(event) => setFund_is_initialized(event.target.value)} />
          <br />
          Fund_min_amount ::: {' '}
          <input type="number" value={Fund_min_amount} onChange={(event) => setFund_min_amount(event.target.value)} />
          <br />
          Fund_min_return ::: {' '}
          <input type="number" value={Fund_min_return} onChange={(event) => setFund_min_return(event.target.value)} />
          <br />
          fFnd_performance_fee_perc ::: {' '}
          <input type="number" value={Fund_performance_fee_perc} onChange={(event) => setFund_performance_fee_perc(event.target.value)} />
          <br />
          <label htmlFor="funds">Select Fund Address:</label>
    
          <select name="funds" width = "100px" onClick={handleFundSelect}>
            {
              funds.map((fund) => {
                return (<option key={fund.fundPDA} value={fund.fundPDA}>{fund.fundPDA}</option>)
              })
            }
          </select>
          <button onClick={handleFunds}>Load Funds</button>
          <button onClick={handleAdminControl}>Set</button>
        </div>
      )
}