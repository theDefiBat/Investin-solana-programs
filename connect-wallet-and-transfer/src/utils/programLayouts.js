import { array, bool, publicKey, str, struct, u32, u64, u8, f64 } from '@project-serum/borsh'

export const NUM_TOKENS = 3
export const MAX_INVESTORS = 10
export const MAX_FUNDS = 20

export const PLATFORM_DATA = struct([
  bool('is_initialized'),
  publicKey('router'),
  u8('router_nonce'),
  u64('no_of_active_funds'),
  array(publicKey('manager'), MAX_FUNDS, 'fund_managers')
])

export const FUND_DATA = struct([
  bool('is_initialized'),
  publicKey('manager_account'),
  u8('signer_nonce'),
  u64('min_amount'),
  u64('min_return'),
  u64('performance_fee_percentage'),
  u64('total_amount'),
  u8('decimals'),
  u64('prev_performance'),
  u8('number_of_active_investments'),
  u8('no_of_investments'),
  u64('amount_in_router'),
  u64('performance_fee'),
  array(
    struct([
      publicKey('mint'),
      u8('decimals'),
      publicKey('vault'),
      u64('balance')
    ]),
    NUM_TOKENS, 'tokens'
  ),
  array(publicKey('investor'), MAX_INVESTORS, 'investors')
])

export const INVESTOR_DATA = struct([
  bool('is_initialized'),
  publicKey('owner'),
  u64('amount'),
  u64('start_performance'),
  u64('amount_in_router'),
  publicKey('manager'),
])



