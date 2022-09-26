use anchor_lang::prelude::*;
use anchor_spl::token::{ self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer };
use spl_token::instruction::AuthorityType;

declare_id!("4MFq2RjxeKARmiAWYZ3zCyuvqjCwg1HatNyqDmPgS83g");
// Exchange from Taker to Initializer and Escrow account to Taker
pub mod exchange_tokens {
  use super::*;
// Exchange function makes all the transactions in a room. Every transaction which is made in a room
// goes through this function. This function is called by the buyer.
// It transfers the tokens from the buyer to the room escrow account.
// Waits for the seller to transfer the tokens to the buyer
// Releases the tokens from the room escrow account to the seller
  pub fn exchange(ctx: Context<Exchange>) -> Result<()> {
    const ESCROW_PA_SEED: &[u8] = b"escrow";
    let (_room_authority, room_authority_bump) = Pubkey::find_program_address(
      &[ESCROW_PA_SEED],
      ctx.program_id
    );
    let a_seeds = &[&ESCROW_PA_SEED[..], &[room_authority_bump]];
    // Transfer from Taker to Initializer
    token::transfer(
      ctx.accounts.into_transfer_to_initializer_context(),
      ctx.accounts.escrow_account.t_amount
    )?;
    // Transfer from Escrow to Taker
    token::transfer(
      ctx.accounts.into_transfer_to_taker_context().with_signer(&[&a_seeds[..]]),
      ctx.accounts.escrow_account.i_amount
    )?;
    // Close the escrow account
    token::close_account(ctx.accounts.into_close_context().with_signer(&[&a_seeds[..]]))?;

    Ok(())
  }

  // Account struct to hold the room accounts
  #[account]
  pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub i_amount: u64,
    pub t_amount: u64,
  }
  //
  impl<'info> Exchange<'info> {
    fn into_transfer_to_initializer_context(
      &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
        from: self.taker_deposit_token_account.to_account_info().clone(),
        to: self.initializer_receive_token_account.to_account_info().clone(),
        authority: self.taker.clone(),
      };
      CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_taker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
        from: self.room_account.to_account_info().clone(),
        to: self.taker_receive_token_account.to_account_info().clone(),
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

  #[derive(Accounts)]
  pub struct Exchange<'info> {
    #[account(signer)]
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub taker: AccountInfo<'info>,
    #[account(mut)]
    pub taker_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub taker_receive_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub initializer_receive_token_account: Box<Account<'info, TokenAccount>>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    #[account(
                mut,
                constraint  =  escrow_account.t_amount  <=  taker_deposit_token_account.amount,
                constraint  =  escrow_account.initializer_deposit_token_account  ==  *initializer_deposit_token_account.to_account_info().key,
                constraint  =  escrow_account.initializer_receive_token_account  ==  *initializer_receive_token_account.to_account_info().key,
                constraint  =  escrow_account.initializer_key  ==  *initializer.key,
                close  =  initializer
        )]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    #[account(mut)]
    pub room_account: Box<Account<'info, TokenAccount>>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub room_authority: AccountInfo<'info>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub token_program: AccountInfo<'info>,
  }
}
