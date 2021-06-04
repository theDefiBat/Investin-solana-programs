use std::cell::{Ref, RefMut};
use std::mem::size_of;
use fixed::types::U64F64;
use arrayref::{array_ref, array_refs};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    entrypoint,
    msg,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    pubkey::Pubkey,
    clock::{Clock, UnixTimestamp},
    sysvar::Sysvar
};
use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use spl_token::state::{Account, Mint};

pub trait Loadable: Pod {
    fn load_mut<'a>(account: &'a AccountInfo) -> Result<RefMut<'a, Self>, ProgramError> {
        // TODO verify if this checks for size
        Ok(RefMut::map(account.try_borrow_mut_data()?, |data| from_bytes_mut(data)))
    }
    fn load<'a>(account: &'a AccountInfo) -> Result<Ref<'a, Self>, ProgramError> {
        Ok(Ref::map(account.try_borrow_data()?, |data| from_bytes(data)))
    }

    fn load_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        Ok(from_bytes(data))
    }
}

pub const MAX_TOKENS:usize = 50;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PriceInfo {
    // mint address of the token
    pub token_mint: Pubkey,

    pub pool_account: Pubkey,

    pub base_pool_account: Pubkey,

    // decimals for token
    pub decimals: u64,

    // price of token
    pub token_price: u64,

    // last updated timestamp
    pub last_updated: UnixTimestamp,

}
unsafe impl Zeroable for PriceInfo{}
unsafe impl Pod for PriceInfo {}

/// Define the type of state stored in accounts
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PriceAccount {
    /// number of tokens
    pub count: u32,

    /// decimals
    pub decimals: u32,

    /// token price info
    pub prices: [PriceInfo; MAX_TOKENS]
}
unsafe impl Zeroable for PriceAccount {}
unsafe impl Pod for PriceAccount {}
impl Loadable for PriceAccount {}


// instruction.rs
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
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

impl AggInstruction {
    pub fn unpack(input: &[u8]) -> Option<Self> {
        let (&op, data) = array_refs![input, 1; ..;];
        let op = u8::from_le_bytes(op);
        Some(match op {
            0 => {
                let data = array_ref![data, 0, 1];
                AggInstruction::AddToken {
                    count: u8::from_le_bytes(*data)
                }
            }
            1 => {
                let data = array_ref![data, 0, 1];
                AggInstruction::UpdateTokenPrices {
                    count: u8::from_le_bytes(*data)
                }
            }
            _ => { return None; }
        })
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

    msg!("entrypoint");
    let instruction = AggInstruction::unpack(_instruction_data).ok_or(ProgramError::InvalidInstructionData)?;

    msg!("instruction unpacked");
    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let price_account = next_account_info(accounts_iter)?;

    msg!("size of account:: {:?}, size of data:: {:?}", size_of::<PriceAccount>(), price_account.data_len());
    let mut price_data = PriceAccount::load_mut(price_account)?;

    let clock_sysvar_info = next_account_info(accounts_iter)?;
    let clock = &Clock::from_account_info(clock_sysvar_info)?;

    match instruction {
        AggInstruction::AddToken { count } => {
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
                let pool_coin_data = parse_token_account(pool_coin_account)?;
                let pool_pc_data = parse_token_account(pool_pc_account)?;

                if pool_coin_data.mint != *mint_account.key {
                    return Err(ProgramError::InvalidAccountData);
                }
                
                if price_data.count == 0 {
                    price_data.decimals = 6;
                }

                // panic if token already present
                find_index(&price_data, pool_coin_data.mint).unwrap_err();
                
                let index = price_data.count as usize;
                price_data.prices[index].token_mint = *mint_account.key;
                price_data.prices[index].decimals = mint_data.decimals.into();
                price_data.prices[index].pool_account = *pool_coin_account.key;
                price_data.prices[index].base_pool_account = *pool_pc_account.key;

                update_price(&mut price_data.prices[index], pool_pc_data.amount, pool_coin_data.amount, clock.unix_timestamp)?;

                price_data.count += 1;
                cnt -= 1;
                if cnt == 0 { break; }
            }
        }
        AggInstruction::UpdateTokenPrices { count } => {
            let mut cnt = count;
            loop {

                let pool_coin_account = next_account_info(accounts_iter)?;
                let pool_pc_account = next_account_info(accounts_iter)?;
                
                let pool_coin_data = parse_token_account(pool_coin_account)?;
                let pool_pc_data = parse_token_account(pool_pc_account)?;

                let index = find_index(&price_data, pool_coin_data.mint)?;
                if price_data.prices[index].pool_account != *pool_coin_account.key {
                    return Err(ProgramError::InvalidAccountData);
                }
                if price_data.prices[index].base_pool_account != *pool_pc_account.key {
                    return Err(ProgramError::InvalidAccountData);
                }

                update_price(&mut price_data.prices[index], pool_pc_data.amount, pool_coin_data.amount, clock.unix_timestamp)?;

                cnt -= 1;
                if cnt == 0 { break; }
            }
        }
    }
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
