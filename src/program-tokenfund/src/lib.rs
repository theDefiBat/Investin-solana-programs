use borsh::{BorshDeserialize, BorshSerialize, BorshSchema};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    instruction:: {AccountMeta, Instruction},
    program_error::ProgramError,
    log::{sol_log_compute_units, sol_log_params, sol_log_slice},
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
};

use spl_token::state::Account as TokenAccount;

/// Struct wrapping data and providing metadata
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct FundData {
    /// The account allowed to update the data
    pub owner: Pubkey,

    /// The data contained by the account, could be anything serializable
    pub amount: u64,

    /// The performance at the time of investment
    pub startPerformance: u64,

    pub manager: Pubkey,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct AccountData {
    /// Account of the manager of the fund
    pub manager: Pubkey,

    /// Base Token Program
    pub baseToken: Pubkey,

    /// Minimum Amount
    pub minAmount: u8,

    /// Minimum Return
    pub minReturn: u8,

    /// Total Amount in fund
    pub totalAmount: u64,

    /// Preformance in fund
    pub performance: u64,

    /// Number of Active Investments in fund
    pub numberOfActiveInvestments: u64,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Data {
    pub instr: u8,
    pub amountIn: u64,
    pub minAmountOut: u64
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum FundInstruction {
    /// Accounts expected
    /// 0. [WRITE]  Main Account
    /// 1. Manager Account
    /// 2. BaseToken Program
    Initialize {
        minAmount: u64,
        minReturn: u64
    },
    
    /// 0. [WRITE]  Main Account
    /// 1. [SIGNER] Investor Account
    /// 2. []       Router Token Account
    /// 3. []       Investor Token Account
    /// 4. []       Manager Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    InvestorDeposit {
        amount: u64
    },

    /// 0. [WRITE]  Main Account
    /// 1. []       Investor Account
    /// 2. []       Router Token Account
    /// 3. []       Investor Token Account
    /// 4. []       Manager Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    InvestorWithdraw {
        amount: u64
    },

    /// 0. [WRITE]  Main Account
    /// 1. []       Investor Account
    /// 2. []       Router Token Account
    /// 3. []       Investor Token Account
    /// 4. []       Manager Token Account
    /// 5. []       PDA of Router
    /// 6. []       PDA of Manager
    /// 7. []       Token Program
    ManagerTransfer {
        amount: u64
    },

    
    Swap {
        data: Data
    },

    GetPerformance {
        amount: u64
    }
}

use thiserror::Error;
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum FundError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    /// Amount less than minimum Amount
    #[error("Amount less than minimum amount")]
    InvalidAmount,

    /// Investor Mismatch
    #[error("Investor Mismatch")]
    InvestorMismatch,

    /// Manager Mismatch
    #[error("Manager Mismatch")]
    ManagerMismatch,
}

impl From<FundError> for ProgramError {
    fn from(e: FundError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!("Token Program Entrypoint");
    
    let instruction = FundInstruction::try_from_slice(_instruction_data)?;

    let account_info_iter = &mut accounts.iter();
  
    // let temp_token_account = next_account_info(account_info_iter)?;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"fund"], program_id);

    let (man_pda, bump_seed) = Pubkey::find_program_address(&[b"manager"], program_id);
    msg!("Manager PDA: {:?}", man_pda);

    msg!("PDA: {:?}", pda);
    
    

    match instruction {
        FundInstruction::Initialize { minAmount, minReturn } => {
            let fund_account = next_account_info(account_info_iter)?;
            let mut account_data = AccountData::try_from_slice(*fund_account.data.borrow())?;

            account_data.managerAccount = next_account_info(account_info_iter)?;
            account_data.baseToken = next_account_info(account_info_iter)?;

            account_data.minAmount = minAmount;
            account_data.minReturn = minReturn;

            account_data.totalAmount = 0; 
            account_data.performance = 1;
            account_data.numberOfActiveInvestments = 0;
        }
        
        FundInstruction::InvestorDeposit { amount } => {
            msg!("Depositing Lamports");

            let mut fund_data = FundData::try_from_slice(*fund_account.data.borrow())?;
            let mut account_data = AccountData::try_from_slice(*fund_account.data.borrow())?;

            if amount <= account_data.minAmount  {
                return Err(ProgramError::InvalidAmount)
                // msg!("Amount less than minimum Amount");
            }
            
            let fund_account = next_account_info(account_info_iter)?;
            let investor_account = next_account_info(account_info_iter)?;
            if !investor_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            if *investor_account.key == fund_data.owner {
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            let temp_token_account = next_account_info(account_info_iter)?;    
            let investor_token_account = next_account_info(account_info_iter)?;
            if *investor_token_account.owner != spl_token::id() {
                return Err(ProgramError::IncorrectProgramId);
            }
            let manager_token_account = next_account_info(account_info_iter)?;
            let pda_account = next_account_info(account_info_iter)?;
            let mpda_account = next_account_info(account_info_iter)?;
            let token_program = next_account_info(account_info_iter)?;
            
            
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                investor_token_account.key,
                temp_token_account.key,
                investor_account.key,
                &[&investor_account.key],
                amount,
            )?;
            msg!("Calling the token program to transfer tokens from investor_token_account to fund_token_account...");
            invoke(
                &transfer_to_fund_ix,
                &[
                    investor_token_account.clone(),
                    temp_token_account.clone(),
                    investor_account.clone(),
                    token_program.clone(),
                ],
            )?;
            msg!("Transfer done");
            msg!("Writing to state account: {:?}", *fund_account.key);

            fund_data.owner = *investor_account.key;
            fund_data.manager = *manager_token_account.key;
            fund_data.amount += amount;
            fund_data.serialize(&mut *fund_account.data.borrow_mut());
        }
        // update numberOfActiveInvestments, totalAmount, previousPerformance
        FundInstruction::InvestorWithdraw { amount: u64 } => {
            msg!("Withdraw Lamports");
            let mut fund_data = FundData::try_from_slice(*fund_account.data.borrow())?;
            let mut account_data = AccountData::try_from_slice(*fund_account.data.borrow())?;


            // if fund_data.amount <= amount {
            //     /// return Err(ProgramError::InvalidAmount)
            // }

            let fund_account = next_account_info(account_info_iter)?;
            let investor_account = next_account_info(account_info_iter)?;
            let temp_token_account = next_account_info(account_info_iter)?;    
            let investor_token_account = next_account_info(account_info_iter)?;
            let manager_token_account = next_account_info(account_info_iter)?;
            let pda_account = next_account_info(account_info_iter)?;
            let mpda_account = next_account_info(account_info_iter)?;
            let token_program = next_account_info(account_info_iter)?;
            
            if fund_data.owner != *investor_account.key {
                return Err(FundError::InvestorMismatch)
                // msg!("Owner mismatch");
            }

            // ***get poolProgID and token_program
            self::getAmountAndPerformance(
                accounts: &[
                // poolProgID.clone(),
                token_program.clone(),
                ], 
                amount: u64,
                program_id: &Pubkey
            ); //totalAmount and performance gets updated

            let percentageReturn = account_data.performance / fund_data.startPerformance;
            msg!("percentage Return: ", percentageReturn);
            let investmentReturns;

            let res = fund_data.amount * percentageReturn;
            if percentageReturn >= account_data.minReturn {
                // calc manager's performance fee
                investmentReturns = res;
            } else {
                investmentReturns =res;
            }

            msg!("authority changed");
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                manager_token_account.key,
                investor_token_account.key,
                &man_pda,
                &[&man_pda],
                investmentReturns,
                // fund_data.amount,
            )?;
            msg!("Calling the token program to transfer tokens from fund_token_account to investor_token_account...");
            invoke_signed(
                &transfer_to_fund_ix,
                &[
                    manager_token_account.clone(),
                    investor_token_account.clone(),
                    mpda_account.clone(),
                    token_program.clone(),
                ],
                &[&[&b"manager"[..], &[bump_seed]]],
            )?;
            fund_data.amount = 0;
            fund_data.startPerformance = 0;
            fund_data.serialize(&mut *fund_account.data.borrow_mut());

            account_data.numberOfActiveInvestments -= 1;
            
        }
        
        FundInstruction::ManagerTransfer {amount: u64} => {
            
            let fund_account = next_account_info(account_info_iter)?;
            let investor_account = next_account_info(account_info_iter)?;
            let temp_token_account = next_account_info(account_info_iter)?;    
            let investor_token_account = next_account_info(account_info_iter)?;
            let manager_token_account = next_account_info(account_info_iter)?;
            let pda_account = next_account_info(account_info_iter)?;
            let mpda_account = next_account_info(account_info_iter)?;
            let token_program = next_account_info(account_info_iter)?;

            msg!("Manager transfering funds");
            let mut fund_data = FundData::try_from_slice(*fund_account.data.borrow())?;
            
            if fund_data.manager != *manager_token_account.key {
                return Err(FundError::ManagerMismatch)
                // msg!("Manager mismatch");
            }
            let transfer_to_fund_ix = spl_token::instruction::transfer(
                token_program.key,
                temp_token_account.key,
                manager_token_account.key,
                &pda,
                &[&pda],
                fund_data.amount
            )?;
            msg!("Calling the token program to transfer tokens from temp_token_account to manager_token_account...");
            invoke_signed(
                &transfer_to_fund_ix,
                &[
                    temp_token_account.clone(),
                    manager_token_account.clone(),
                    pda_account.clone(),
                    token_program.clone(),
                ],
                &[&[&b"fund"[..], &[_nonce]]],
            )?;

            let mut account_data = AccountData::try_from_slice(*fund_account.data.borrow())?;
            account_data.numberOfActiveInvestments += 1;
            // ***get poolProgID and token_program
            // self::getAmountAndPerformance(
            //     accounts: &[
            //     poolProgID.clone(),
            //     token_program.clone(),
            //     ], 
            //     amount: u64,
            //     program_id: &Pubkey
            // ); //totalAmount and performance gets updated
            let mut fund_data = FundData::try_from_slice(*fund_account.data.borrow())?;
            fund_data.startPerformance = account_data.performance;
        }
        FundInstruction::Swap { data } => {
            
            msg!("Data passed: {:?}", data);
            let poolProgId = next_account_info(account_info_iter)?;
            let tokenProgramId = next_account_info(account_info_iter)?;
            let ammId = next_account_info(account_info_iter)?;
            let ammAuthority = next_account_info(account_info_iter)?;
            let ammOpenOrders = next_account_info(account_info_iter)?;    
            let ammTargetOrders = next_account_info(account_info_iter)?;
            let poolCoinTokenAccount = next_account_info(account_info_iter)?;
            let poolPcTokenAccount = next_account_info(account_info_iter)?;
            let serumProgramId = next_account_info(account_info_iter)?;
            let serumMarket = next_account_info(account_info_iter)?;
            let serumBids = next_account_info(account_info_iter)?;
            let serumAsks = next_account_info(account_info_iter)?;
            let serumEventQueue = next_account_info(account_info_iter)?;
            let serumCoinVaultAccount = next_account_info(account_info_iter)?;
            let serumPcVaultAccount = next_account_info(account_info_iter)?;
            let serumVaultSigner = next_account_info(account_info_iter)?;
            let userSourceTokenAccount = next_account_info(account_info_iter)?;
            let userDestTokenAccount = next_account_info(account_info_iter)?;
            let userOwner = next_account_info(account_info_iter)?;

            let mut accounts = Vec::with_capacity(18);

            accounts.push(AccountMeta::new(*tokenProgramId.key, false));
            accounts.push(AccountMeta::new(*ammId.key, false));
            accounts.push(AccountMeta::new(*ammAuthority.key, false));
            accounts.push(AccountMeta::new(*ammOpenOrders.key, false));
            accounts.push(AccountMeta::new(*ammTargetOrders.key, false));
            accounts.push(AccountMeta::new(*poolCoinTokenAccount.key, false));
            accounts.push(AccountMeta::new(*poolPcTokenAccount.key, false));
            accounts.push(AccountMeta::new(*serumProgramId.key, false));
            accounts.push(AccountMeta::new(*serumMarket.key, false));
            accounts.push(AccountMeta::new(*serumBids.key, false));
            accounts.push(AccountMeta::new(*serumAsks.key, false));
            accounts.push(AccountMeta::new(*serumEventQueue.key, false));
            accounts.push(AccountMeta::new(*serumCoinVaultAccount.key, false));
            accounts.push(AccountMeta::new(*serumPcVaultAccount.key, false));
            accounts.push(AccountMeta::new(*serumVaultSigner.key, false));
            accounts.push(AccountMeta::new(*userSourceTokenAccount.key, false));
            accounts.push(AccountMeta::new(*userDestTokenAccount.key, false));
            accounts.push(AccountMeta::new(*userOwner.key, true));

            let swap_ix = Instruction::new_with_borsh(*poolProgId.key, &data, accounts);
            msg!("invoking swap of pool program");
            invoke_signed(
                &swap_ix,
                &[
                    tokenProgramId.clone(),
                    ammId.clone(),
                    ammAuthority.clone(),
                    ammOpenOrders.clone(),
                    ammTargetOrders.clone(),
                    poolCoinTokenAccount.clone(),
                    poolPcTokenAccount.clone(),
                    serumProgramId.clone(),
                    serumMarket.clone(),
                    serumBids.clone(),
                    serumAsks.clone(),
                    serumEventQueue.clone(),
                    serumCoinVaultAccount.clone(),
                    serumPcVaultAccount.clone(),
                    serumVaultSigner.clone(),
                    userSourceTokenAccount.clone(),
                    userDestTokenAccount.clone(),
                    userOwner.clone()
                ],
                &[&[&b"manager"[..], &[bump_seed]]],
            )?;
        }

        FundInstruction::GetPerformance{amount: u64} => {
            msg!("Instruction: getPerformance");
                Self::getAmountAndPerformance(accounts, amount, program_id)
        }
    }

    fn getAmountAndPerformance(
        tokenProgramId: pubkey,
    ) -> ProgramResult {
        
        let baseTokenValue = 0;
        baseTokenValue += FundData::baseToken.balance(); //get baseToken Balance
        baseTokenValue += tokenProgramId.balance() * self::getTokenPrice(token0: tokenProgramId.key, token1: FundData::baseToken);; //get Balance
        
        let mut fund_data = FundData::try_from_slice(*fund_account.data.borrow())?;
        
        if FundData::numberOfActiveInvestments != 0 {
            fund_data.previousPerformance = (baseTokenValue / fund_data.totalAmount) * fund_data.previousPerformance;
        }
    }

    fn getTokenPrice(
        token0: &pubkey,
        token1: &pubkey,
    ) -> u8 {
        let tokenPrice = 0;
        if token0 == tokenProgramId && token1 == FundData::baseToken {
            const coinAmount = new TokenAmount(amount, token0.decimals, false); //get total reserve of coin in poolProgID
            const pcAmount = new TokenAmount(amount, token1.decimals, false); //get total reserve of pc in poolProgID
            tokenPrice = coinAmount / pcAmount;
        } else {
            const coinAmount = new TokenAmount(amount, token0.decimals, false); //get total reserve of coin in poolProgID
            const pcAmount = new TokenAmount(amount, token1.decimals, false); //get total reserve of pc in poolProgID
            tokenPrice = pcAmount / coinAmount;   
        }
        return tokenPrice;
    }
    Ok(())
    
}   
