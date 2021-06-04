import { array, bool, publicKey, str, struct, u32, u64, u8, f64, u128, i64, u16 } from '@project-serum/borsh'

export const NUM_TOKENS = 10
export const MAX_INVESTORS = 10
export const MAX_FUNDS = 200

export const PLATFORM_DATA = struct([
  bool('is_initialized'),
  u8('router_nonce'),
  u8('no_of_active_funds'),
  array(u8('padding'), 5),
  
  publicKey('router'),
  publicKey('investin_admin'),
  array(publicKey('manager'), MAX_FUNDS, 'fund_managers')
])

export const FUND_DATA = struct([
  bool('is_initialized'),
  u8('decimals'),
  u8('number_of_active_investments'),
  u8('no_of_investments'),
  u32('signer_nonce'),

  u64('min_amount'),
  u64('min_return'),
  u64('performance_fee_percentage'),
  u64('total_amount'),
  u64('prev_performance'),
  u64('amount_in_router'),
  u64('performance_fee'),
  publicKey('manager_account'),
  array(
    struct([
      publicKey('mint'),
      u64('decimals'),
      publicKey('vault'),
      u64('balance')
    ]),
    NUM_TOKENS, 'tokens'
  ),
  array(publicKey('investor'), MAX_INVESTORS, 'investors')
])

export const INVESTOR_DATA = struct([
  bool('is_initialized'),
  array(u8('padding'), 7),

  publicKey('owner'),
  u64('amount'),
  u64('start_performance'),
  u64('amount_in_router'),
  publicKey('manager'),
])



export const AMM_INFO_LAYOUT_V4 = struct([
  u64('status'),
  u64('nonce'),
  u64('orderNum'),
  u64('depth'),
  u64('coinDecimals'),
  u64('pcDecimals'),
  u64('state'),
  u64('resetFlag'),
  u64('minSize'),
  u64('volMaxCutRatio'),
  u64('amountWaveRatio'),
  u64('coinLotSize'),
  u64('pcLotSize'),
  u64('minPriceMultiplier'),
  u64('maxPriceMultiplier'),
  u64('systemDecimalsValue'),
  // Fees
  u64('minSeparateNumerator'),
  u64('minSeparateDenominator'),
  u64('tradeFeeNumerator'),
  u64('tradeFeeDenominator'),
  u64('pnlNumerator'),
  u64('pnlDenominator'),
  u64('swapFeeNumerator'),
  u64('swapFeeDenominator'),
  // OutPutData
  u64('needTakePnlCoin'),
  u64('needTakePnlPc'),
  u64('totalPnlPc'),
  u64('totalPnlCoin'),
  u128('poolTotalDepositPc'),
  u128('poolTotalDepositCoin'),
  u128('swapCoinInAmount'),
  u128('swapPcOutAmount'),
  u64('swapCoin2PcFee'),
  u128('swapPcInAmount'),
  u128('swapCoinOutAmount'),
  u64('swapPc2CoinFee'),

  publicKey('poolCoinTokenAccount'),
  publicKey('poolPcTokenAccount'),
  publicKey('coinMintAddress'),
  publicKey('pcMintAddress'),
  publicKey('lpMintAddress'),
  publicKey('ammOpenOrders'),
  publicKey('serumMarket'),
  publicKey('serumProgramId'),
  publicKey('ammTargetOrders'),
  publicKey('poolWithdrawQueue'),
  publicKey('poolTempLpTokenAccount'),
  publicKey('ammOwner'),
  publicKey('pnlOwner')
])


export const MAX_TOKENS = 50
// Aggregator Accounts
export const PRICE_DATA = struct([
  u32('count'),
  u32('decimals'),
  array(
    struct([
      publicKey('token_mint'),
      publicKey('pool_account'),
      publicKey('base_pool_account'),
      u64('decimals'),
      u64('token_price'),
      i64('last_updated'),
    ]),
    MAX_TOKENS, 'prices'
  ),
])