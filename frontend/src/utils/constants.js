import { Connection, PublicKey } from "@solana/web3.js";

export const PLATFORM_ACCOUNT_KEY = "platAccKey_11";
export const FUND_ACCOUNT_KEY = "fundAccKey_11";
export const MARGIN_ACCOUNT_KEY_1 = "margin_account_key_11"
export const MARGIN_ACCOUNT_KEY_2 = "margin_account_key_22"
export const PRICE_ACCOUNT_KEY = "margin_account_key_22"


export const adminAccount = new PublicKey('B1J3ttZ2PLCG4KUQHLtkgYSZUmD6f8KNLjGSWzz8eE1Y')

export const cluster = "https://api.devnet.solana.com";
//export const cluster = "https://solana-api.projectserum.com";

export const connection = new Connection(cluster, "confirmed");

export const programId = new PublicKey('6HwgHw4QdgKR6kjHpdFdwGoP8MBy1gHGtde4JkS1GQmE');
//main = 8dbbmZXbLsUirEsgaBVcPBEdciESza6L2zkEuer4crR

// change later
export const platformStateAccount = new PublicKey('GYmNA5Tw15e9MkuDHKB2ynqrgNFweTNAVT6yDQCPDWrv')
export const priceStateAccount = new PublicKey('ARXWJc6FmrAB7zkL2LAeRWGgaAe3MUqD1MJUTZGyiRQn')

export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
export const MEMO_PROGRAM_ID = new PublicKey('Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo')
export const RENT_PROGRAM_ID = new PublicKey('SysvarRent111111111111111111111111111111111')
export const CLOCK_PROGRAM_ID = new PublicKey('SysvarC1ock11111111111111111111111111111111')
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL')

//export const LIQUIDITY_POOL_PROGRAM_ID_V4 = '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8'
//export const SERUM_PROGRAM_ID_V3 = '9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin'
export const LIQUIDITY_POOL_PROGRAM_ID_V4 = new PublicKey('9rpQHSyFVM1dkkHFQ2TtTzPEW7DVmEyPmN8wVniqJtuC')
export const SERUM_PROGRAM_ID_V3 = new PublicKey('DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY')

export const MANGO_PROGRAM_ID_V2 = new PublicKey('9XzhtAtDXxW2rjbeVFhTq4fnhD8dqzr154r5b2z6pxEp')
export const MANGO_GROUP_ACCOUNT = new PublicKey('B9Uddrao7b7sCjNZp1BJSQqFzqhMEmBxD2SvYTs2TSBn')
export const MANGO_VAULT_ACCOUNT_USDC = new PublicKey('CRZemtdc8FjHYWqLDVPwYK5PGbVSxPCKoyf6fyEFkfjq')

export const ORACLE_BTC_DEVNET = new PublicKey("FuEnReoxhqW8Li6EMLoaaUWbWAEjTfSRuBARo5GrGCqN")
export const ORACLE_ETH_DEVNET = new PublicKey("GzfYWGM1oeVrha9zvM1awnTJEUAuinpnVRUyYQYELzqg")
export const ORACLE_SOL_DEVNET = new PublicKey("AshULbjkGvse8YW2ojjeqHdMbFGigLy2xxiGVhsLqX5T")
export const ORACLE_SRM_DEVNET = new PublicKey("B3nWGxqNQzJeRfpYSXU8qJaTQxspZmqAt91FRAhfoFQL")

//export const SOL_USDC_MARKET = new PublicKey("4Rf4qZYwBVo6RsxisBnm8RJCRMehiZ2TsDwfyoR9X4dF")
export const SOL_USDC_MARKET = new PublicKey("E1mfsnnCcL24JcDQxr7F2BpWjkyy5x2WHys8EL2pnCj9")
