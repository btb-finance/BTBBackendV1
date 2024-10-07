use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_lang::solana_program::clock::Clock;

declare_id!("F4JnCD9KASp74g2zCg8GkoSj1boKKzCcZvq9Fjs4LzBz");

#[program]
pub mod btb_token_sale {
    use super::*;

    pub fn initialize_sale(
        ctx: Context<InitializeSale>,
        start_time: i64,
        end_time: i64,
        total_tokens_for_sale: u64,
    ) -> Result<()> {
        let sale = &mut ctx.accounts.sale;
        sale.owner = ctx.accounts.owner.key();
        sale.btb_mint = ctx.accounts.btb_mint.key();
        sale.usdt_mint = ctx.accounts.usdt_mint.key();
        sale.sale_vault = ctx.accounts.sale_vault.key();
        sale.start_time = start_time;
        sale.end_time = end_time;
        sale.total_tokens_for_sale = total_tokens_for_sale;
        sale.tokens_sold = 0;

        // Transfer 800M BTB to the sale vault
        let transfer_amount = 800_000_000 * 10u64.pow(6); // Assuming 6 decimals
        let cpi_accounts = Transfer {
            from: ctx.accounts.owner_btb_account.to_account_info(),
            to: ctx.accounts.sale_vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, transfer_amount)?;

        Ok(())
    }

    pub fn process_purchase(
        ctx: Context<ProcessPurchase>,
        amount: u64,
        is_vested: bool,
    ) -> Result<()> {
        let sale = &mut ctx.accounts.sale;
        let clock = Clock::get()?;

        require!(
            clock.unix_timestamp >= sale.start_time && clock.unix_timestamp <= sale.end_time,
            BTBError::SaleNotActive
        );

        let price = if is_vested {
            5 // 0.0005 USDT per BTB (50% discount)
        } else {
            10 // 0.001 USDT per BTB
        };

        let usdt_amount = (amount * price) / 10;
        let btb_amount = amount * 10u64.pow(6); // Convert to 6 decimal places

        require!(
            sale.tokens_sold + btb_amount <= sale.total_tokens_for_sale,
            BTBError::InsufficientTokens
        );

        // Transfer USDT from buyer to sale account
        let cpi_accounts = Transfer {
            from: ctx.accounts.buyer_usdt_account.to_account_info(),
            to: ctx.accounts.sale_usdt_account.to_account_info(),
            authority: ctx.accounts.buyer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, usdt_amount)?;

        if is_vested {
            // Store vesting info
            let vesting_info = &mut ctx.accounts.vesting_info;
            vesting_info.buyer = ctx.accounts.buyer.key();
            vesting_info.total_vested = btb_amount;
            vesting_info.total_claimed = 0;
            vesting_info.start_time = clock.unix_timestamp;
        } else {
            // Transfer BTB tokens to buyer immediately
            let seeds = &[sale.to_account_info().key.as_ref(), &[sale.bump]];
            let signer = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.sale_vault.to_account_info(),
                to: ctx.accounts.buyer_btb_account.to_account_info(),
                authority: sale.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, btb_amount)?;
        }

        sale.tokens_sold += btb_amount;

        // Emit purchase event
        emit!(PurchaseEvent {
            buyer: ctx.accounts.buyer.key(),
            amount: btb_amount,
            is_vested,
        });

        Ok(())
    }

    pub fn claim_vested_tokens(ctx: Context<ClaimVestedTokens>) -> Result<()> {
        let vesting_info = &mut ctx.accounts.vesting_info;
        let sale = &ctx.accounts.sale;
        let clock = Clock::get()?;

        let days_since_purchase = (clock.unix_timestamp - vesting_info.start_time) / 86400; // 86400 seconds in a day
        let unlocked_tokens = (vesting_info.total_vested * days_since_purchase as u64) / 365;
        let claimable_tokens = unlocked_tokens.saturating_sub(vesting_info.total_claimed);

        require!(claimable_tokens > 0, BTBError::NoTokensToClaim);

        // Transfer claimable BTB tokens to buyer
        let seeds = &[sale.to_account_info().key.as_ref(), &[sale.bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.sale_vault.to_account_info(),
            to: ctx.accounts.buyer_btb_account.to_account_info(),
            authority: sale.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, claimable_tokens)?;

        vesting_info.total_claimed += claimable_tokens;

        // Emit claim event
        emit!(ClaimEvent {
            buyer: ctx.accounts.buyer.key(),
            amount: claimable_tokens,
        });

        Ok(())
    }

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        let sale = &ctx.accounts.sale;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp > sale.end_time, BTBError::SaleNotEnded);

        let remaining_tokens = sale.total_tokens_for_sale.saturating_sub(sale.tokens_sold);

        // Transfer remaining BTB tokens to owner
        let seeds = &[sale.to_account_info().key.as_ref(), &[sale.bump]];
        let signer = &[&seeds[..]];
        let cpi_accounts = Transfer {
            from: ctx.accounts.sale_vault.to_account_info(),
            to: ctx.accounts.owner_btb_account.to_account_info(),
            authority: sale.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, remaining_tokens)?;

        // Emit emergency withdraw event
        emit!(EmergencyWithdrawEvent {
            amount: remaining_tokens,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeSale<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8, seeds = [b"sale"], bump)]
    pub sale: Account<'info, Sale>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub btb_mint: Account<'info, token::Mint>,
    pub usdt_mint: Account<'info, token::Mint>,
    #[account(mut)]
    pub owner_btb_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sale_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ProcessPurchase<'info> {
    #[account(mut)]
    pub sale: Account<'info, Sale>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_usdt_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_btb_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sale_usdt_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sale_vault: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = buyer, space = 8 + 32 + 8 + 8 + 8)]
    pub vesting_info: Account<'info, VestingInfo>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimVestedTokens<'info> {
    #[account(mut)]
    pub sale: Account<'info, Sale>,
    #[account(mut)]
    pub vesting_info: Account<'info, VestingInfo>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_btb_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sale_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(mut, has_one = owner)]
    pub sale: Account<'info, Sale>,
    pub owner: Signer<'info>,
    #[account(mut)]
    pub owner_btb_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub sale_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Sale {
    pub owner: Pubkey,
    pub btb_mint: Pubkey,
    pub usdt_mint: Pubkey,
    pub sale_vault: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub total_tokens_for_sale: u64,
    pub tokens_sold: u64,
    pub bump: u8,
}

#[account]
pub struct VestingInfo {
    pub buyer: Pubkey,
    pub total_vested: u64,
    pub total_claimed: u64,
    pub start_time: i64,
}

#[error_code]
pub enum BTBError {
    #[msg("Sale is not active")]
    SaleNotActive,
    #[msg("Insufficient tokens available for sale")]
    InsufficientTokens,
    #[msg("No tokens available to claim")]
    NoTokensToClaim,
    #[msg("Sale has not ended yet")]
    SaleNotEnded,
}

#[event]
pub struct PurchaseEvent {
    pub buyer: Pubkey,
    pub amount: u64,
    pub is_vested: bool,
}

#[event]
pub struct ClaimEvent {
    pub buyer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EmergencyWithdrawEvent {
    pub amount: u64,
}