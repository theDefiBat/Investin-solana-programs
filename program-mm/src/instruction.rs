use arrayref::{array_ref, array_refs};
use mango::matching::{OrderType, Side};
use num_enum::TryFromPrimitive;

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::processor::Fund;

#[repr(C)]
#[derive(Clone)]
pub enum FundInstruction {
    
    Initialize {
        min_amount: u64,
        performance_fee_bps: u64,
    },

    InvestorDeposit {
        amount: u64,
    },

    
    InvestorWithdraw,

    InvestorRequestWithdraw,

    ClaimPerformanceFee,
    
    ProcessDeposits,

    ProcessWithdraws,

    // MangoPlacePerpOrder {
    //     price: i64,
    //     quantity: i64,
    //     client_order_id: u64,
    //     side: Side,
    //     /// Can be 0 -> LIMIT, 1 -> IOC, 2 -> PostOnly
    //     order_type: OrderType,
    // },

    SetMangoDelegate,

    
}

impl FundInstruction {
    pub fn unpack(input: &[u8]) -> Option<Self> {
        let (&op, data) = array_refs![input, 4; ..;];
        let op = u32::from_le_bytes(op);
        Some(match op {
            0 => {
                let data = array_ref![data, 0, 8 + 8];
                let (min_amount, performance_fee_bps) = array_refs![data, 8, 8];

                FundInstruction::Initialize {
                    min_amount: u64::from_le_bytes(*min_amount),
                    performance_fee_bps: u64::from_le_bytes(*performance_fee_bps),
                }
            }
            1 => {
                let amount = array_ref![data, 0, 8];
                FundInstruction::InvestorDeposit {
                    amount: u64::from_le_bytes(*amount),
                }
            }
            2 => FundInstruction::InvestorWithdraw,
            3 => FundInstruction::InvestorRequestWithdraw,
            4 => FundInstruction::ProcessDeposits,
            5 => FundInstruction::ProcessWithdraws,
            6 => FundInstruction::ClaimPerformanceFee,
            7 => FundInstruction::SetMangoDelegate,
            _ => {
                return None;
            }
        })
    }
}

pub fn init_mm_fund(
    program_id: &Pubkey,
    admin: &Pubkey,
    fund_pda_ai: &Pubkey,
    fund_usdc_vault_ai: &Pubkey,
    mango_program_ai: &Pubkey,
    mango_group_ai: &Pubkey,
    mango_account_ai: &Pubkey,
    delegate_ai: &Pubkey,
    system_program_id: &Pubkey,
    min_amount: u64,
    performance_fee_bps: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*admin, true),
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*fund_usdc_vault_ai, false),
        AccountMeta::new(*mango_program_ai, false),
        AccountMeta::new(*mango_group_ai, false),
        AccountMeta::new(*mango_account_ai, false),
        AccountMeta::new(*delegate_ai, false),
        AccountMeta::new_readonly(*system_program_id, false),
    ];

    let _instr = FundInstruction::Initialize{
        min_amount,
        performance_fee_bps
    };
    let mut data = vec![0];
    data.extend(min_amount.to_le_bytes().to_vec());
    data.extend(performance_fee_bps.to_le_bytes().to_vec());

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}

pub fn investor_deposit(
    program_id: &Pubkey,
    fund_pda_ai: &Pubkey,
    investor_state_ai: &Pubkey,
    investor_ai: &Pubkey,
    investor_usdc_vault_ai: &Pubkey,
    fund_vault_ai: &Pubkey,
    token_prog_ai: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*investor_state_ai, false),
        AccountMeta::new(*investor_ai, true),
        AccountMeta::new(*investor_usdc_vault_ai, false),
        AccountMeta::new(*fund_vault_ai, false),
        AccountMeta::new_readonly(*token_prog_ai, false),
    ];

    let _instr = FundInstruction::InvestorDeposit{
        amount
    };
    let mut data = vec![1];
    data.extend(amount.to_le_bytes().to_vec());

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}

pub fn process_deposit(
    program_id: &Pubkey,
    fund_pda_ai: &Pubkey,
    manager_ai:&Pubkey,
    mango_program_ai: &Pubkey,
    mango_group_ai: &Pubkey,
    mango_account_ai: &Pubkey,
    mango_cache_ai: &Pubkey,
    root_bank_ai: &Pubkey,
    node_bank_ai: &Pubkey,
    vault_ai: &Pubkey,
    token_prog_ai: &Pubkey,
    fund_usdc_vault_ai: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*manager_ai, true),
        AccountMeta::new(*mango_program_ai, false),
        AccountMeta::new(*mango_group_ai, false),
        AccountMeta::new(*mango_account_ai, false),
        AccountMeta::new(*mango_cache_ai, false),
        AccountMeta::new(*root_bank_ai, false),
        AccountMeta::new(*node_bank_ai, false),
        AccountMeta::new(*vault_ai, false),
        AccountMeta::new_readonly(*token_prog_ai, false),
        AccountMeta::new(*fund_usdc_vault_ai, false),
    ];

    let _instr = FundInstruction::ProcessDeposits{};
    let mut data = vec![4];

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}

pub fn investor_request_withdraw(
    program_id: &Pubkey,
    fund_pda_ai: &Pubkey,
    investor_state_ai: &Pubkey,
    investor_ai: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*investor_state_ai, false),
        AccountMeta::new(*investor_ai, true),
    ];

    let _instr = FundInstruction::InvestorRequestWithdraw{};
    let mut data = vec![3];

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}

pub fn process_withdraw(
    program_id: &Pubkey,
    fund_pda_ai: &Pubkey,
    manager_ai:&Pubkey,
    mango_program_ai: &Pubkey,
    mango_group_ai: &Pubkey,
    mango_account_ai: &Pubkey,
    mango_cache_ai: &Pubkey,
    root_bank_ai: &Pubkey,
    node_bank_ai: &Pubkey,
    vault_ai: &Pubkey,
    signer_ai: &Pubkey,
    token_prog_ai: &Pubkey,
    fund_usdc_vault_ai: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*manager_ai, true),
        AccountMeta::new(*mango_program_ai, false),
        AccountMeta::new(*mango_group_ai, false),
        AccountMeta::new(*mango_account_ai, false),
        AccountMeta::new(*mango_cache_ai, false),
        AccountMeta::new(*root_bank_ai, false),
        AccountMeta::new(*node_bank_ai, false),
        AccountMeta::new(*vault_ai, false),
        AccountMeta::new(*signer_ai, false),
        AccountMeta::new_readonly(*token_prog_ai, false),
        AccountMeta::new(*fund_usdc_vault_ai, false),
    ];

    let _instr = FundInstruction::ProcessWithdraws{};
    let mut data = vec![5];

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}

pub fn investor_withdraw(
    program_id: &Pubkey,
    fund_pda_ai: &Pubkey,
    investor_state_ai: &Pubkey,
    investor_ai: &Pubkey,
    investor_usdc_vault_ai: &Pubkey,
    fund_vault_ai: &Pubkey,
    token_prog_ai: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*investor_state_ai, false),
        AccountMeta::new(*investor_ai, true),
        AccountMeta::new(*investor_usdc_vault_ai, false),
        AccountMeta::new(*fund_vault_ai, false),
        AccountMeta::new(*token_prog_ai, false),
    ];

    let _instr = FundInstruction::InvestorWithdraw{};
    let mut data = vec![2];

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}

pub fn claim_performnace_fee(
    program_id: &Pubkey,
    fund_pda_ai: &Pubkey,
    manager_ai:&Pubkey,
    mango_program_ai: &Pubkey,
    mango_group_ai: &Pubkey,
    mango_account_ai: &Pubkey,
    mango_cache_ai: &Pubkey,
    root_bank_ai: &Pubkey,
    node_bank_ai: &Pubkey,
    vault_ai: &Pubkey,
    token_prog_ai: &Pubkey,
    manager_usdc_vault_ai: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*fund_pda_ai, false),
        AccountMeta::new(*manager_ai, true),
        AccountMeta::new(*mango_program_ai, false),
        AccountMeta::new(*mango_group_ai, false),
        AccountMeta::new(*mango_account_ai, false),
        AccountMeta::new(*mango_cache_ai, false),
        AccountMeta::new(*root_bank_ai, false),
        AccountMeta::new(*node_bank_ai, false),
        AccountMeta::new(*vault_ai, false),
        AccountMeta::new_readonly(*token_prog_ai, false),
        AccountMeta::new(*manager_usdc_vault_ai, false),
    ];

    let _instr = FundInstruction::ClaimPerformanceFee{};
    let mut data = vec![6];

    Ok(Instruction {
        program_id: *program_id,
        accounts: accounts,
        data: data,
    })
}