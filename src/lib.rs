use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("EFTS5RZsKzMJktKEKfXVp1cqprtumVHPEvWyrNcixtP5");

#[program]
mod multi_signature_wallet {
    use super::*;

    pub fn create_multi_signature_wallet(
        ctx: Context<CreateWallet>,
        name: String,
        threshold: u64,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        if users.is_empty() {
            return Err(Errors::InvalidUsers.into());
        }

        if threshold == 0 || threshold > users.len() as u64 {
            return Err(Errors::InvalidThreshold.into());
        }

        let signer = &ctx.accounts.signer;
        if !users.contains(&signer.key()) {
            return Err(Errors::SignerNotIncluded.into());
        }

        let wallet_account = &mut ctx.accounts.wallet_account;
        wallet_account.name = name.clone();
        wallet_account.threshold = threshold;
        wallet_account.users = users;
        wallet_account.created_by = signer.key();

        emit!(WalletCreated {
            wallet: wallet_account.key(),
            users: wallet_account.users.clone(),
            threshold,
        });

        msg!(
            "Multi Signature Wallet created with name: {} and threshold: {}",
            name,
            threshold
        );
        Ok(())
    }

    pub fn create_multi_signature_transaction(
        ctx: Context<CreateTransaction>,
        name: String,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            return Err(Errors::InvalidAmount.into());
        }

        let wallet_account = &ctx.accounts.wallet_account;
        let signer = &ctx.accounts.signer;
        if !wallet_account.users.contains(&signer.key()) {
            return Err(Errors::SignerNotIncluded.into());
        }

        let balance = wallet_account.to_account_info().lamports();
        if balance < amount {
            return Err(Errors::InsufficientBalance.into());
        }

        let wallet_transaction_account = &mut ctx.accounts.wallet_transaction_account;
        wallet_transaction_account.name = name.clone();
        wallet_transaction_account.threshold = wallet_account.threshold;
        wallet_transaction_account.reciever = ctx.accounts.reciever.key();
        wallet_transaction_account.created_by = signer.key();
        wallet_transaction_account.amount = amount;
        wallet_transaction_account.wallet_account = wallet_account.key();
        wallet_transaction_account.completed_signers = 0;
        let clock = Clock::get()?;
        wallet_transaction_account.time_stamp = clock.unix_timestamp as u64;

        emit!(TransactionCreated {
            transaction: wallet_transaction_account.key(),
            wallet: wallet_account.key(),
        });

        msg!("Transaction proposed successfully");
        Ok(())
    }

    pub fn sign_the_transaction(ctx: Context<SignTheTransaction>) -> Result<()> {
        let wallet_account = &ctx.accounts.wallet_account;
        let wallet_transaction_account = &mut ctx.accounts.wallet_transaction_account;
        let transaction_signature_account = &mut ctx.accounts.transaction_signature_account;

        if !wallet_account.users.contains(&ctx.accounts.signer.key()) {
            return Err(Errors::UnauthorizedUser.into());
        }

        if wallet_account.key() != wallet_transaction_account.wallet_account {
            return Err(Errors::MismatchedWalletAndTxn.into());
        }

        transaction_signature_account.signer = ctx.accounts.signer.key();
        transaction_signature_account.wallet_account = wallet_account.key();
        transaction_signature_account.wallet_transaction_account = wallet_transaction_account.key();
        let clock = Clock::get()?;
        transaction_signature_account.timestamp = clock.unix_timestamp as u64;

        wallet_transaction_account.completed_signers += 1;

        emit!(TransactionSigned {
            transaction: wallet_transaction_account.key(),
            wallet: wallet_account.key(),
            signer: ctx.accounts.signer.key(),
            transaction_signature: transaction_signature_account.key()
        });

        if wallet_transaction_account.completed_signers >= wallet_transaction_account.threshold {
            let reciever = wallet_transaction_account.reciever;
            let amount = wallet_transaction_account.amount;

            if reciever != ctx.accounts.reciever.key() {
                return Err(Errors::InvalidReciever.into());
            }

            **ctx
                .accounts
                .wallet_account
                .to_account_info()
                .try_borrow_mut_lamports()? -= amount;
            **ctx
                .accounts
                .reciever
                .to_account_info()
                .try_borrow_mut_lamports()? += amount;

            emit!(TransactionExecuted {
                transaction: wallet_transaction_account.key(),
                wallet: wallet_account.key(),
            });
        }

        Ok(())
    }

    pub fn transfer_sol_to_wallet(ctx: Context<TransferSolToWallet>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(Errors::InvalidAmount.into());
        }

        let ixn = system_instruction::transfer(
            &ctx.accounts.signer.key(),
            &ctx.accounts.wallet_account.key(),
            amount,
        );

        invoke(
            &ixn,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.wallet_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        msg!("Transaction completed successfully");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateWallet<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [b"multi_signature_wallet", signer.key().as_ref(), name.as_ref()],
        bump,
        space = 8 + WalletAccount::INIT_SPACE
    )]
    pub wallet_account: Account<'info, WalletAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateTransaction<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [b"multi_signature_transaction", signer.key().as_ref(), wallet_account.key().as_ref(), reciever.key().as_ref(), name.as_ref()],
        bump,
        space = 8 + WalletTransaction::INIT_SPACE + 8
    )]
    pub wallet_transaction_account: Account<'info, WalletTransaction>,
    #[account(mut)]
    pub wallet_account: Account<'info, WalletAccount>,
    pub reciever: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SignTheTransaction<'info> {
    #[account(
        init,
        payer = signer,
        seeds = [b"transaction_signature", signer.key().as_ref(), wallet_account.key().as_ref(), wallet_transaction_account.key().as_ref()],
        bump,
        space = 8 + TransactionSignature::INIT_SPACE + 8
    )]
    pub transaction_signature_account: Account<'info, TransactionSignature>,
    #[account(mut)]
    pub wallet_account: Account<'info, WalletAccount>,
    #[account(mut)]
    pub wallet_transaction_account: Account<'info, WalletTransaction>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub reciever: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferSolToWallet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub wallet_account: Account<'info, WalletAccount>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct WalletAccount {
    #[max_len(32)]
    pub name: String,
    pub created_by: Pubkey,
    #[max_len(5)]
    pub users: Vec<Pubkey>,
    pub threshold: u64,
}

#[account]
#[derive(InitSpace)]
pub struct WalletTransaction {
    #[max_len(32)]
    pub name: String,
    pub wallet_account: Pubkey,
    pub reciever: Pubkey,
    pub amount: u64,
    pub threshold: u64,
    pub completed_signers: u64,
    pub time_stamp: u64,
    pub created_by: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct TransactionSignature {
    pub signer: Pubkey,
    pub wallet_account: Pubkey,
    pub wallet_transaction_account: Pubkey,
    pub timestamp: u64,
}

#[event]
pub struct WalletCreated {
    pub wallet: Pubkey,
    pub users: Vec<Pubkey>,
    pub threshold: u64,
}

#[event]
pub struct TransactionCreated {
    pub transaction: Pubkey,
    pub wallet: Pubkey,
}

#[event]
pub struct TransactionSigned {
    pub transaction: Pubkey,
    pub wallet: Pubkey,
    pub signer: Pubkey,
    pub transaction_signature: Pubkey,
}

#[event]
pub struct TransactionExecuted {
    pub transaction: Pubkey,
    pub wallet: Pubkey,
}

#[error_code]
pub enum Errors {
    #[msg("Please enter a valid number of users")]
    InvalidUsers,
    #[msg("Please provide a valid threshold")]
    InvalidThreshold,
    #[msg("Signer must be included in the users")]
    SignerNotIncluded,
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("You have insufficient balance")]
    InsufficientBalance,
    #[msg("Unauthorized user")]
    UnauthorizedUser,
    #[msg("This transaction is not related to this wallet")]
    MismatchedWalletAndTxn,
    #[msg("Reciever must be the same as in the transaction that was proposed")]
    InvalidReciever,
}
