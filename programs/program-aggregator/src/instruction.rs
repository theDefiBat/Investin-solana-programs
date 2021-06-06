use arrayref::{array_ref, array_refs};

#[repr(C)]
#[derive(Clone)]
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