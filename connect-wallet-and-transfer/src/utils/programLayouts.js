import { bool, publicKey, struct, u32, u64, u8 } from '@project-serum/borsh'

export const INVESTOR_DATA = struct([
  bool('is_initialized'),
  u64('amount'),
  u64('start_performance'),
  publicKey('manager'),
  u8('signer_nonce')
])

export const TOKEN_INFO = struct([
  publicKey('mint1'),
  u8('decimals1'),
  publicKey('vault1'),
  u64('balance1'),
])
export const TOKEN_INFO1 = struct([
  publicKey('mint2'),
  u8('decimals2'),
  publicKey('vault2'),
  u64('balance2'),
])
export const TOKEN_INFO2 = struct([
  publicKey('mint3'),
  u8('decimals3'),
  publicKey('vault3'),
  u64('balance3'),
])

export const FUND_DATA = struct([
  bool('is_initialized'),
  publicKey('manager_account'),
  u8('signer_nonce'),
  u64('min_amount'),

  /// Minimum Return
  u64('min_return'),

  /// Total Amount in fund
  u64('total_amount'),

  // decimals
  u8('decimals'),

  /// Preformance in fund
  u64('prev_performance'),
  u64('number_of_active_investments'),

  /// Tokens owned
  // pub tokens: [TokenInfo; NUM_TOKENS]
  publicKey('mint1'),
  u8('decimals1'),
  publicKey('vault1'),
  u64('balance1'),
  
  publicKey('mint2'),
  u8('decimals2'),
  publicKey('vault2'),
  u64('balance2'),

  publicKey('mint3'),
  u8('decimals3'),
  publicKey('vault3'),
  u64('balance3'),
  
])
