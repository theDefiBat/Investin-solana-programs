use arrayref::{array_ref, array_refs};
use mango::matching::{OrderType, Side};
use num_enum::TryFromPrimitive;

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
    
    ProcessDeposits {
    },

    ProcessWithdraws {
    },

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
