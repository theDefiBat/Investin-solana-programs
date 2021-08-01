import { Blob, seq, struct, u32, u8, u16, ns64 } from 'buffer-layout';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

export const NUM_TOKENS = 8
export const MAX_TOKENS = 50
export const NUM_MARGIN = 2
export const MAX_INVESTORS = 10

class PublicKeyLayout extends Blob {
  constructor(property) {
    super(32, property);
  }

  decode(b, offset) {
    return new PublicKey(super.decode(b, offset));
  }

  encode(src, b, offset) {
    return super.encode(src.toBuffer(), b, offset);
  }
}

export function publicKeyLayout(property = "") {
  return new PublicKeyLayout(property);
}

class BNLayout extends Blob {
  constructor(number, property) {
    super(number, property);
    // restore prototype chain
    Object.setPrototypeOf(this, new.target.prototype)
  }

  decode(b, offset) {
    return new BN(super.decode(b, offset), 10, 'le');
  }

  encode(src, b, offset) {
    return super.encode(src.toArrayLike(Buffer, 'le', this['span']), b, offset);
  }
}

class U64F64Layout extends Blob {
  constructor(property) {
    super(16, property);
  }

  decode(b, offset) {
    const raw = new BN(super.decode(b, offset), 10, 'le');

    return raw / Math.pow(2, 64);
  }

  encode(src, b, offset) {
    console.log("src ::: ", src)
    return super.encode(src.toArrayLike(Buffer, 'le', this['span']), b, offset);
  }
}

export function U64F64(property = "") {
  return new U64F64Layout(property)
}

export function u64(property = "") {
  return new BNLayout(8, property);
}

export function u128(property = "") {
  return new BNLayout(16, property);
}
 
export const PLATFORM_DATA = struct([
  u8('is_initialized'),
  u8('version'),
  u8('router_nonce'),
  u8('no_of_active_funds'),
  u8('token_count'),
  seq(u8(), 3, 'padding'),

  publicKeyLayout('router'),
  publicKeyLayout('investin_admin'),
  publicKeyLayout('investin_vault'),

  seq(
    struct([
      publicKeyLayout('mint'),
      u64('decimals'),
      publicKeyLayout('pool_coin_account'),
      publicKeyLayout('pool_pc_account'),
      U64F64('pool_price'),
      ns64('last_updated'),
      u64('padding')
    ]),
    MAX_TOKENS, 'token_list'
  ),

])

export const FUND_DATA = struct([
  u8('is_initialized'),
  u8('number_of_active_investments'),
  u8('no_of_investments'),
  u8('signer_nonce'),
  u8('no_of_margin_positions'),
  u8('no_of_assets'),
  u16('position_count'),

  u8('version'),
  seq(u8(), 7, 'padding'),

  u64('min_amount'),
  U64F64('min_return'),
  U64F64('performance_fee_percentage'),
  U64F64('total_amount'),
  U64F64('prev_performance'),

  u64('amount_in_router'),
  U64F64('performance_fee'),
  publicKeyLayout('manager_account'),
  publicKeyLayout('fund_pda'),

  seq(
    struct([
      u8('is_initialized'),
      u8('index'),
      seq(u8(), 6, 'padding'),

      u64('balance'),
      u64('debt'),

      publicKeyLayout('vault')
    ]),
    NUM_TOKENS, 'tokens'
  ),
  seq(publicKeyLayout(), MAX_INVESTORS, 'investors'),
  seq(
    struct([
      publicKeyLayout('margin_account'),
      u8('state'),
      u8('margin_index'),
      u8('position_side'),
      seq(u8('padding'), 3),
      u16('position_id'),

      u64('trade_amount'),
      U64F64('fund_share'),
      U64F64('share_ratio')
    ]),
    NUM_MARGIN, 'mango_positions'
  ),
  seq(u8(), 32, 'padding'),

])

export const INVESTOR_DATA = struct([
  u8('is_initialized'),
  u8('has_withdrawn'),
  u8('withdrawn_from_margin'),
  seq(u8('padding'), 5),


  publicKeyLayout('owner'),
  u64('amount'),
  U64F64('start_performance'),
  u64('amount_in_router'),
  publicKeyLayout('manager'),
  seq(U64F64(), NUM_MARGIN, 'margin_debt'),
  seq(u64(), NUM_MARGIN, 'margin_position_id'),

  seq(u8(), NUM_TOKENS, 'token_indexes'),
  seq(u64(), NUM_TOKENS, 'token_debts'),

  seq(u8(), 32, 'xpadding')

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

  publicKeyLayout('poolCoinTokenAccount'),
  publicKeyLayout('poolPcTokenAccount'),
  publicKeyLayout('coinMintAddress'),
  publicKeyLayout('pcMintAddress'),
  publicKeyLayout('lpMintAddress'),
  publicKeyLayout('ammOpenOrders'),
  publicKeyLayout('serumMarket'),
  publicKeyLayout('serumProgramId'),
  publicKeyLayout('ammTargetOrders'),
  publicKeyLayout('poolWithdrawQueue'),
  publicKeyLayout('poolTempLpTokenAccount'),
  publicKeyLayout('ammOwner'),
  publicKeyLayout('pnlOwner')
])


// Aggregator Accounts
export const PRICE_DATA = struct([
  u32('count'),
  u32('decimals'),
  seq(
    struct([
      publicKeyLayout('token_mint'),
      publicKeyLayout('pool_account'),
      publicKeyLayout('base_pool_account'),
      u64('decimals'),
      u64('token_price'),
      ns64('last_updated'),
    ]),
    MAX_TOKENS, 'prices'
  ),
])