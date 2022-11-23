# Solana deposits
# Built with Seahorse v0.2.3
#
# This smart contract receives SOL deposits and mints a token to the depositor with exchange rate 1:1.
# Depositors can then redeem their tokens and obtain their SOL back.

from seahorse.prelude import *

declare_id('2gZztwduAXpyFTdHaibrZsCimyJHqEtgJvL3U22EFiS9')


class DepositPool(Account):
  bump: u8


@instruction
def init(owner: Signer, pool: Empty[DepositPool], mint: Empty[TokenMint]):
  bump = pool.bump()

  pool = pool.init(
    payer=owner,
    seeds=['dep-pool']
  )

  mint.init(
    payer = owner,
    seeds = ['pool-token-mint'],
    decimals = 9,
    authority = pool
  )

  pool.bump = bump


@instruction
def init_associated_token_account(
  new_token_account: Empty[TokenAccount],
  mint: TokenMint,
  signer: Signer
):
  new_token_account.init(
    payer = signer,
    seeds = ['pool-token-mint', signer],
    mint = mint,
    authority = signer
  )

@instruction
def deposit(user: Signer, user_tkn_acc: TokenAccount, pool: DepositPool, mintaccount: TokenMint, n: u64):
  bump = pool.bump

  mintaccount.mint(
    authority = pool,
    to = user_tkn_acc,
    amount = n,
    signer = ['dep-pool', bump]
  )

  user.transfer_lamports(
  to = pool,
  amount = n,  # *1000000000,
  )

  print(f'User {user.key()} deposited {n/f64(1000000000)} SOL.')

@instruction
def withdraw(user: Signer, user_tkn_acc: TokenAccount, pool: DepositPool, pooltokenaccount: TokenAccount, n: u64):
  bump = pool.bump

  user_tkn_acc.transfer(
    authority = user,
    to = pooltokenaccount,
    amount = n,
  )

  pool.transfer_lamports(
  to = user,
  amount = n, # *1000000000,
  )

  print(f'User {user.key()} withdrew {n/f64(1000000000)} SOL.')
