use borsh::{BorshDeserialize, BorshSerialize};
use fixed::types::U64F64;
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    entrypoint,
    msg,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar
};

use spl_token::state::{Account, Mint};

use solana_program::clock::UnixTimestamp;
pub const MAX_TOKENS:usize = 10;

#[derive(Clone, Debug, Default, Copy, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct PriceInfo {
    // mint address of the token
    pub token_mint: Pubkey,

    pub decimals: u8,

    // pub pool_account: Pubkey,

    // pub base_pool_account: Pubkey,

    // price of token
    pub token_price: u64,

    // last updated timestamp
    pub last_updated: UnixTimestamp
}

/// Define the type of state stored in accounts
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct PriceAccount {
    pub is_initialized: bool,

    /// number of tokens
    pub count: u8,

    /// decimals
    pub decimals: u8,

    /// token price info
    pub prices: [PriceInfo; MAX_TOKENS]
}

// instruction.rs

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum AggInstruction {
    
    /// Accounts Expected
    /// 0. [WRITE] Price Account
    /// 1. [READ] CLOCK SYSVAR account
    /// 2. [SIGNER] Investin Admin Account
    /// 3. [READ]   Token Mint Account
    /// 4. []   Pool Token Account
    /// 5. []   Pool Base Token Account
    /// ............
    /// N. 
    AddToken {
        count: u8 // count of tokens
    },

    /// Accounts Expected
    /// 0. [WRITE] Price Account
    /// 1. [READ] CLOCK SYSVAR account
    /// 2. [READ]   Pool Token Account
    /// 3. [READ]   Pool Base Token Account
    /// ......
    UpdateTokenPrices {
        count: u8 // count of tokens
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

    msg!("Aggregator entrypoint");
    let instruction = AggInstruction::try_from_slice(_instruction_data)?;

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let price_account = next_account_info(accounts_iter)?;
    let mut price_data = PriceAccount::try_from_slice(&price_account.data.borrow())?;

    let clock_sysvar_info = next_account_info(accounts_iter)?;
    let clock = &Clock::from_account_info(clock_sysvar_info)?;

    match instruction {
        AggInstruction::AddToken { count } => {
            msg!("add tokens callled");
            let mut cnt = count;
            let admin_account = next_account_info(accounts_iter)?;
            if !admin_account.is_signer{
                return Err(ProgramError::MissingRequiredSignature);
            }
            loop {
                let mint_account = next_account_info(accounts_iter)?;
                let pool_coin_account = next_account_info(accounts_iter)?;
                let pool_pc_account = next_account_info(accounts_iter)?;
                

                let mint_data = Mint::unpack(&mint_account.data.borrow())?;
                msg!("mint accounts parsed");
                msg!("parsing account:: {:?}", *pool_coin_account.key);

                let pool_coin_data = parse_token_account(pool_coin_account)?;
                msg!("coin token accounts parsed");

                let pool_pc_data = parse_token_account(pool_pc_account)?;

                msg!(" pool tokens accounts parsed");

                if pool_coin_data.mint != *mint_account.key {
                    msg!("error in mint: {:?}", pool_coin_data.mint);
                    return Err(ProgramError::InvalidAccountData);
                }
                
                if !price_data.is_initialized {
                    price_data.is_initialized = true;
                    price_data.decimals = 6;
                    price_data.count = 0;
                }
                price_data.prices[price_data.count as usize].token_mint = *mint_account.key;
                price_data.prices[price_data.count as usize].decimals = mint_data.decimals;
                // price_data.prices[price_data.count as usize].pool_account = *pool_coin_account.key;
                // price_data.prices[price_data.count as usize].base_pool_account = *pool_pc_account.key;

                update_price(&mut price_data.prices[price_data.count as usize], pool_pc_data.amount, pool_coin_data.amount, clock.unix_timestamp)?;

                price_data.count += 1;
                cnt -= 1;
                if cnt == 0 { break; }
            }
        }
        AggInstruction::UpdateTokenPrices { count } => {
            let mut cnt = count;
            loop {

                let pool_coin_data = Account::unpack(&(next_account_info(accounts_iter)?).data.borrow())?;
                let pool_pc_data = Account::unpack(&(next_account_info(accounts_iter)?).data.borrow())?;

                let index = find_index(&price_data, pool_coin_data.mint)?;
                update_price(&mut price_data.prices[index], pool_pc_data.amount, pool_coin_data.amount, clock.unix_timestamp)?;

                cnt -= 1;
                if cnt == 0 { break; }
            }
        }
    }
    price_data.serialize(&mut *price_account.data.borrow_mut())?;

    Ok(())
}

pub fn find_index (
    price_data: &PriceAccount,
    mint: Pubkey
) -> Result<usize, ProgramError> {
    for i in 0..price_data.count {
        if price_data.prices[i as usize].token_mint == mint {
            return Ok(i as usize)
        }
    }
    return Err(ProgramError::InvalidArgument)
}
pub fn update_price(
    price_info: &mut PriceInfo,
    pc_res: u64,
    coin_res: u64,
    timestamp: UnixTimestamp
) -> ProgramResult {
    msg!("update price called");
    let adj_decimals = 6 + (price_info.decimals - 6);

    price_info.token_price = U64F64::to_num(
        U64F64::from_num(pc_res)
        .checked_div(U64F64::from_num(coin_res)).unwrap()
        .checked_mul(U64F64::from_num(10u64.pow(adj_decimals as u32))).unwrap()
    );
    price_info.last_updated = timestamp;

    Ok(())
}

pub fn parse_token_account (account_info: &AccountInfo) -> Result<Account, ProgramError> {
    if account_info.owner != &spl_token::ID {
        msg!("Account not owned by spl-token program");
        return Err(ProgramError::IncorrectProgramId);
    }
    let parsed = Account::unpack(&account_info.try_borrow_data()?)?;
    if !parsed.is_initialized() {
        msg!("Token account not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    Ok(parsed)
}
