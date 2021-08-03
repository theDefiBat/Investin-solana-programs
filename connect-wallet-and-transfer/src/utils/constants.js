import { Connection, PublicKey } from "@solana/web3.js";

export const PLATFORM_ACCOUNT_KEY = "pacck";
export const FUND_ACCOUNT_KEY = "facck";
export const MARGIN_ACCOUNT_KEY_1 = "macck1"
export const MARGIN_ACCOUNT_KEY_2 = "macck2"

export const adminAccount = new PublicKey('Fepyuf4vy7mKZVgpzS52UoUeSLmVvGnoMDyraCsjYUqn')

export const cluster = "https://solana-api.projectserum.com";

export const connection = new Connection(cluster, "confirmed");
export const programId = new PublicKey('8dbbmZXbLsUirEsgaBVcPBEdciESza6L2zkEuer4crR');

// change later
export const platformStateAccount = new PublicKey('Cpf6kq7w4iR2hWdWTkWeoxvyRrrduke5XhA7QM5SkGNo')

export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
export const MEMO_PROGRAM_ID = new PublicKey('Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo')
export const RENT_PROGRAM_ID = new PublicKey('SysvarRent111111111111111111111111111111111')
export const CLOCK_PROGRAM_ID = new PublicKey('SysvarC1ock11111111111111111111111111111111')
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL')

export const LIQUIDITY_POOL_PROGRAM_ID_V4 = new PublicKey('675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8')
export const SERUM_PROGRAM_ID_V3 = new PublicKey('9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin')

export const MANGO_PROGRAM_ID_V2 = new PublicKey('5fNfvyp5czQVX77yoACa3JJVEhdRaWjPuazuWgjhTqEH')
export const MANGO_GROUP_ACCOUNT = new PublicKey('2oogpTYm1sp6LPZAWD3bp2wsFpnV2kXL1s52yyFhW5vp')
