use  anchor_lang::prelude::*;
use  anchor_spl::token::{self,  CloseAccount,  Mint,  SetAuthority,  TokenAccount,  Transfer};
use  spl_token::instruction::AuthorityType;


declare_id!("4MFq2RjxeKARmiAWYZ3zCyuvqjCwg1HatNyqDmPgS83g");
// Exchange from Taker to Initializer and Escrow account to Taker
pub  mod  anchor_exchange  {
        use  super::*;

    pub  fn  exchange(ctx:  Context<Exchange>)  ->  Result<()>  {
        const  ESCROW_PA_SEED:  &[u8]  =  b"escrow";
        let  (_vault_authority,  vault_authority_bump)  =
                Pubkey::find_program_address(&[ESCROW_PA_SEED],  ctx.program_id);
        let  a_seeds  =  &[&ESCROW_PA_SEED[..],  &[vault_authority_bump]];
        // Transfer from Taker to Initializer
        token::transfer(
                ctx.accounts.into_transfer_to_initializer_context(),
                ctx.accounts.escrow_account.t_amount,
        )?;
        // Transfer from Escrow to Taker
        token::transfer(
                ctx.accounts
                        .into_transfer_to_taker_context()
                        .with_signer(&[&a_seeds[..]]),
                ctx.accounts.escrow_account.i_amount,
        )?;
        // Close the escrow account
        token::close_account(
                ctx.accounts
                        .into_close_context()
                        .with_signer(&[&a_seeds[..]]),
        )?;

        Ok(())
    }
}
