#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{assign, index_assign, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct DepositPool {
    pub bump: u8,
}

impl<'info, 'entrypoint> DepositPool {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedDepositPool<'info, 'entrypoint>> {
        let bump = account.bump;

        Mutable::new(LoadedDepositPool {
            __account__: account,
            __programs__: programs_map,
            bump,
        })
    }

    pub fn store(loaded: Mutable<LoadedDepositPool>) {
        let mut loaded = loaded.borrow_mut();
        let bump = loaded.bump;

        loaded.__account__.bump = bump;
    }
}

#[derive(Debug)]
pub struct LoadedDepositPool<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, DepositPool>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub bump: u8,
}

pub fn init_associated_token_account_handler<'info>(
    mut new_token_account: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut mint: SeahorseAccount<'info, '_, Mint>,
    mut signer: SeahorseSigner<'info, '_>,
) -> () {
    new_token_account.account.clone();
}

pub fn withdraw_handler<'info>(
    mut user: SeahorseSigner<'info, '_>,
    mut user_tkn_acc: SeahorseAccount<'info, '_, TokenAccount>,
    mut pool: Mutable<LoadedDepositPool<'info, '_>>,
    mut pooltokenaccount: SeahorseAccount<'info, '_, TokenAccount>,
    mut n: u64,
) -> () {
    let mut bump = pool.borrow().bump;

    token::transfer(
        CpiContext::new(
            user_tkn_acc.programs.get("token_program"),
            token::Transfer {
                from: user_tkn_acc.to_account_info(),
                authority: user.to_account_info(),
                to: pooltokenaccount.to_account_info(),
            },
        ),
        n,
    )
    .unwrap();

    {
        let amount = n;

        **pool
            .borrow()
            .__account__
            .to_account_info()
            .try_borrow_mut_lamports()
            .unwrap() -= amount;

        **user.to_account_info().try_borrow_mut_lamports().unwrap() += amount;
    };

    solana_program::msg!(
        "{}",
        format!(
            "User {:?} withdrew {} SOL.",
            user.key(),
            ((n as f64) / <f64 as TryFrom<_>>::try_from(1000000000).unwrap())
        )
    );
}

pub fn init_handler<'info>(
    mut owner: SeahorseSigner<'info, '_>,
    mut pool: Empty<Mutable<LoadedDepositPool<'info, '_>>>,
    mut mint: Empty<SeahorseAccount<'info, '_, Mint>>,
) -> () {
    let mut bump = pool.bump.unwrap();
    let mut pool = pool.account.clone();

    mint.account.clone();

    assign!(pool.borrow_mut().bump, bump);
}

pub fn deposit_handler<'info>(
    mut user: SeahorseSigner<'info, '_>,
    mut user_tkn_acc: SeahorseAccount<'info, '_, TokenAccount>,
    mut pool: Mutable<LoadedDepositPool<'info, '_>>,
    mut mintaccount: SeahorseAccount<'info, '_, Mint>,
    mut n: u64,
) -> () {
    let mut bump = pool.borrow().bump;

    token::mint_to(
        CpiContext::new_with_signer(
            mintaccount.programs.get("token_program"),
            token::MintTo {
                mint: mintaccount.to_account_info(),
                authority: pool.borrow().__account__.to_account_info(),
                to: user_tkn_acc.to_account_info(),
            },
            &[Mutable::new(vec![
                "dep-pool".as_bytes().as_ref(),
                bump.to_le_bytes().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        n,
    )
    .unwrap();

    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            &user.key(),
            &pool.borrow().__account__.key(),
            n,
        ),
        &[
            user.to_account_info(),
            pool.borrow().__account__.to_account_info(),
            user.programs.get("system_program").clone(),
        ],
    )
    .unwrap();

    solana_program::msg!(
        "{}",
        format!(
            "User {:?} deposited {} SOL.",
            user.key(),
            ((n as f64) / <f64 as TryFrom<_>>::try_from(1000000000).unwrap())
        )
    );
}
