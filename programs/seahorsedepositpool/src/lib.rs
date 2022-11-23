#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("2gZztwduAXpyFTdHaibrZsCimyJHqEtgJvL3U22EFiS9");

pub mod seahorse_util {
    use super::*;
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index > 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index > 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }
}

#[program]
mod seahorsedepositpool {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    pub struct InitAssociatedTokenAccount<'info> {
        # [account (init , payer = signer , seeds = ["pool-token-mint" . as_bytes () . as_ref () , signer . key () . as_ref ()] , bump , token :: mint = mint , token :: authority = signer)]
        pub new_token_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub mint: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub signer: Signer<'info>,
        pub token_program: Program<'info, Token>,
        pub system_program: Program<'info, System>,
        pub rent: Sysvar<'info, Rent>,
    }

    pub fn init_associated_token_account(ctx: Context<InitAssociatedTokenAccount>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let new_token_account = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.new_token_account,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("new_token_account").map(|bump| *bump),
        };

        let mint = SeahorseAccount {
            account: &ctx.accounts.mint,
            programs: &programs_map,
        };

        let signer = SeahorseSigner {
            account: &ctx.accounts.signer,
            programs: &programs_map,
        };

        init_associated_token_account_handler(
            new_token_account.clone(),
            mint.clone(),
            signer.clone(),
        );

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (n : u64)]
    pub struct Withdraw<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub user_tkn_acc: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub pool: Box<Account<'info, dot::program::DepositPool>>,
        #[account(mut)]
        pub pooltokenaccount: Box<Account<'info, TokenAccount>>,
        pub token_program: Program<'info, Token>,
    }

    pub fn withdraw(ctx: Context<Withdraw>, n: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let user = SeahorseSigner {
            account: &ctx.accounts.user,
            programs: &programs_map,
        };

        let user_tkn_acc = SeahorseAccount {
            account: &ctx.accounts.user_tkn_acc,
            programs: &programs_map,
        };

        let pool = dot::program::DepositPool::load(&mut ctx.accounts.pool, &programs_map);
        let pooltokenaccount = SeahorseAccount {
            account: &ctx.accounts.pooltokenaccount,
            programs: &programs_map,
        };

        withdraw_handler(
            user.clone(),
            user_tkn_acc.clone(),
            pool.clone(),
            pooltokenaccount.clone(),
            n,
        );

        dot::program::DepositPool::store(pool);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct Init<'info> {
        #[account(mut)]
        pub owner: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: DepositPool > () + 8 , payer = owner , seeds = ["dep-pool" . as_bytes () . as_ref ()] , bump)]
        pub pool: Box<Account<'info, dot::program::DepositPool>>,
        # [account (init , payer = owner , seeds = ["pool-token-mint" . as_bytes () . as_ref ()] , bump , mint :: decimals = 9 , mint :: authority = pool)]
        pub mint: Box<Account<'info, Mint>>,
        pub token_program: Program<'info, Token>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn init(ctx: Context<Init>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let owner = SeahorseSigner {
            account: &ctx.accounts.owner,
            programs: &programs_map,
        };

        let pool = Empty {
            account: dot::program::DepositPool::load(&mut ctx.accounts.pool, &programs_map),
            bump: ctx.bumps.get("pool").map(|bump| *bump),
        };

        let mint = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.mint,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("mint").map(|bump| *bump),
        };

        init_handler(owner.clone(), pool.clone(), mint.clone());

        dot::program::DepositPool::store(pool.account);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (n : u64)]
    pub struct Deposit<'info> {
        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub user_tkn_acc: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub pool: Box<Account<'info, dot::program::DepositPool>>,
        #[account(mut)]
        pub mintaccount: Box<Account<'info, Mint>>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    pub fn deposit(ctx: Context<Deposit>, n: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let user = SeahorseSigner {
            account: &ctx.accounts.user,
            programs: &programs_map,
        };

        let user_tkn_acc = SeahorseAccount {
            account: &ctx.accounts.user_tkn_acc,
            programs: &programs_map,
        };

        let pool = dot::program::DepositPool::load(&mut ctx.accounts.pool, &programs_map);
        let mintaccount = SeahorseAccount {
            account: &ctx.accounts.mintaccount,
            programs: &programs_map,
        };

        deposit_handler(
            user.clone(),
            user_tkn_acc.clone(),
            pool.clone(),
            mintaccount.clone(),
            n,
        );

        dot::program::DepositPool::store(pool);

        return Ok(());
    }
}
