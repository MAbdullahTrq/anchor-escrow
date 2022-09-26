use  anchor_lang::prelude::*;
use  anchor_spl::token::{self,  CloseAccount,  Mint,  SetAuthority,  TokenAccount,  Transfer};
use  spl_token::instruction::AuthorityType;

mod  anchor_escrow;
mod  anchor_exchange;
declare_id!("4MFq2RjxeKARmiAWYZ3zCyuvqjCwg1HatNyqDmPgS83g");
// Cancel logic to cancle the escrow account
pub mod cancel 
{
    use  super::*;
    const  ESCROW_PA_SEED:  &[u8]  =  b"escrow";
    pub  fn  cancel(ctx:  Context<Cancel>)  ->  Result<()>  {
        let  (_vault_authority,  vault_authority_bump)  =
                Pubkey::find_program_address(&[ESCROW_PA_SEED],  ctx.program_id);
        let  a_seeds  =  &[&ESCROW_PA_SEED[..],  &[vault_authority_bump]];

        token::transfer(
                ctx.accounts
                        .into_transfer_to_initializer_context()
                        .with_signer(&[&a_seeds[..]]),
                ctx.accounts.escrow_account.i_amount,
        )?;

        token::close_account(
                ctx.accounts
                        .into_close_context()
                        .with_signer(&[&a_seeds[..]]),
        )?;

        Ok(())
    }

    
}
