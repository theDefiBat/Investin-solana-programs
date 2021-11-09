pub fn mango_init_margin_account (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError>
{
    const NUM_FIXED: usize = 6;
    let accounts = array_ref![accounts, 0, NUM_FIXED + NUM_MARGIN];
    let (
        fixed_accs,
        margin_accs
    ) = array_refs![accounts, NUM_FIXED, NUM_MARGIN];

    let [
        fund_state_acc,
        manager_acc,
        fund_pda_acc,
        mango_prog_acc,
        mango_group_acc,
        rent_acc
    ] = fixed_accs;

    let mut fund_data = FundData::load_mut_checked(fund_state_acc, program_id)?;

    check!(manager_acc.is_signer, ProgramError::MissingRequiredSignature);
    check_eq!(fund_data.manager_account, *manager_acc.key);


    for i in 0..NUM_MARGIN {
        check_eq!(fund_data.mango_positions[i].margin_account, Pubkey::default());
        invoke_signed(
            &init_margin_account(mango_prog_acc.key, mango_group_acc.key, margin_accs[i].key, fund_pda_acc.key)?,
            &[
                mango_prog_acc.clone(),
                mango_group_acc.clone(),
                margin_accs[i].clone(),
                fund_pda_acc.clone(),
                rent_acc.clone()
            ],
            &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
        )?;
        fund_data.mango_positions[i].margin_account = *margin_accs[i].key;
        fund_data.mango_positions[i].state = 0; // inactive state
    }
    Ok(())
}