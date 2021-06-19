import { Connection, PublicKey } from "@solana/web3.js";

export const PLATFORM_ACCOUNT_KEY = "platAccKey_5";
export const FUND_ACCOUNT_KEY = "fundAccKey_5";
export const PRICE_ACCOUNT_KEY = "price_account_key_5";

export const adminAccount = new PublicKey('Fepyuf4vy7mKZVgpzS52UoUeSLmVvGnoMDyraCsjYUqn')

export const cluster = "https://api.devnet.solana.com";
export const connection = new Connection(cluster, "confirmed");

export const programId = new PublicKey('BX8FpMj7MgTYr2YBan2kG9KX2UGimtBrECk2DabTYEYP');

// change later
export const platformStateAccount = new PublicKey('HfKDiUUY8G28ouaYiDwa9zJCKGdS4KgLV4sWpx1Fdjfe')
export const priceStateAccount = new PublicKey('ARXWJc6FmrAB7zkL2LAeRWGgaAe3MUqD1MJUTZGyiRQn')

export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
export const MEMO_PROGRAM_ID = new PublicKey('Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo')
export const RENT_PROGRAM_ID = new PublicKey('SysvarRent111111111111111111111111111111111')
export const CLOCK_PROGRAM_ID = new PublicKey('SysvarC1ock11111111111111111111111111111111')
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL')

//export const LIQUIDITY_POOL_PROGRAM_ID_V4 = '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8'
//export const SERUM_PROGRAM_ID_V3 = '9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin'
export const LIQUIDITY_POOL_PROGRAM_ID_V4 = '9rpQHSyFVM1dkkHFQ2TtTzPEW7DVmEyPmN8wVniqJtuC'
export const SERUM_PROGRAM_ID_V3 = 'DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY'

export const MANGO_PROGRAM_ID_V2 = '9XzhtAtDXxW2rjbeVFhTq4fnhD8dqzr154r5b2z6pxEp'
export const MANGO_GROUP_ACCOUNT = 'B9Uddrao7b7sCjNZp1BJSQqFzqhMEmBxD2SvYTs2TSBn'
export const MANGO_VAULT_ACCOUNT_USDC = 'CRZemtdc8FjHYWqLDVPwYK5PGbVSxPCKoyf6fyEFkfjq'
 
