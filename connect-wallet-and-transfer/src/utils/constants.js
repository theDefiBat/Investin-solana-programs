import { Connection, PublicKey } from "@solana/web3.js";

export const PLATFORM_ACCOUNT_KEY = "platformAccount";
export const FUND_ACCOUNT_KEY = "fundAccount";
export const INVESTOR_ACCOUNT_KEY = "investorAccount";

export const adminAccount = new PublicKey('Fepyuf4vy7mKZVgpzS52UoUeSLmVvGnoMDyraCsjYUqn')

export const cluster = "https://api.devnet.solana.com";
export const connection = new Connection(cluster, "confirmed");

export const programId = new PublicKey('BX8FpMj7MgTYr2YBan2kG9KX2UGimtBrECk2DabTYEYP');

// change later
export const platformStateAccount = new PublicKey('5Uhg65hoU7BmXwJAwfuS9enGUzK9YDbVMh73GA1hN6xR')

export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
export const MEMO_PROGRAM_ID = new PublicKey('Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo')
export const RENT_PROGRAM_ID = new PublicKey('SysvarRent111111111111111111111111111111111')
export const CLOCK_PROGRAM_ID = new PublicKey('SysvarC1ock11111111111111111111111111111111')
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL')

export const LIQUIDITY_POOL_PROGRAM_ID_V4 = '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8'
export const SERUM_PROGRAM_ID_V3 = '9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin'