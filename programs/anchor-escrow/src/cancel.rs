use anchor_lang::prelude::*;
use anchor_spl::token::{ self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer };
use spl_token::instruction::AuthorityType;

declare_id!("4MFq2RjxeKARmiAWYZ3zCyuvqjCwg1HatNyqDmPgS83g");
// Cancel logic to cancle the Room transactions
pub mod cancel {
// account struct to hold the room accounts
  #[account]
  pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub i_amount: u64,
    pub t_amount: u64,
  }
  use super::*;
  const ESCROW_PA_SEED: &[u8] = b"escrow";
  pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
    let (_room_authority, room_authority_bump) = Pubkey::find_program_address(
      &[ESCROW_PA_SEED],
      ctx.program_id
    );
    let a_seeds = &[&ESCROW_PA_SEED[..], &[room_authority_bump]];
// Reversing the escrow transactions
    token::transfer(
      ctx.accounts.into_transfer_to_initializer_context().with_signer(&[&a_seeds[..]]),
      ctx.accounts.escrow_account.i_amount
    )?;
// closing the accounts
    token::close_account(ctx.accounts.into_close_context().with_signer(&[&a_seeds[..]]))?;

    Ok(())
  }

  #[derive(Accounts)]
  pub struct Cancel<'info> {
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    #[account(mut,  signer)]
    pub initializer: AccountInfo<'info>,
    #[account(mut)]
    pub room_account: Account<'info, TokenAccount>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub room_authority: AccountInfo<'info>,
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    #[account(
                mut,
                constraint  =  escrow_account.initializer_key  ==  *initializer.key,
                constraint  =  escrow_account.initializer_deposit_token_account  ==  *initializer_deposit_token_account.to_account_info().key,
                close  =  initializer
        )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub token_program: AccountInfo<'info>,
  }

  impl<'info> Cancel<'info> {
    fn into_transfer_to_initializer_context(
      &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
        from: self.room_account.to_account_info().clone(),
        to: self.initializer_deposit_token_account.to_account_info().clone(),
        authority: self.room_authority.clone(),
      };
      CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
      let cpi_accounts = CloseAccount {
        account: self.room_account.to_account_info().clone(),
        destination: self.initializer.clone(),
        authority: self.room_authority.clone(),
      };
      CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
  }
}
