use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    declare_id,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Counter {
    pub count: u64,
}

mod state;
use state::*;

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (instruction_discriminant, instruction_data_inner) = instruction_data.split_at(1);
    match instruction_discriminant[0] {
        0 => {
            msg!("Instruction: Deposit");
            process_deposit(accounts, instruction_data_inner)?;
        }
        1 => {
            msg!("Instruction: Withdraw");
            process_withdraw(accounts, instruction_data_inner)?;
        }
        _ => {
            msg!("Error: unknown instruction")
        }
    }
    Ok(())
}

pub fn process_deposit(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let transfer_amount: u64 = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

    // let pda_key = Pubkey::create_program_address(&[], program_id)?;
    // if pda_account.key != &pda_key {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    assert!(
        pda_account.is_writable,
        "Deposit account must be writable"
    );

    // Creating transfer instruction
    let transfer_instruction = solana_program::system_instruction::transfer(
        user_account.key,
        pda_account.key,
        transfer_amount,
    );

    // Transfering sol to pda account
    invoke(
        &transfer_instruction,
        &[user_account.clone(), pda_account.clone(), system_program.clone()],
    )?;

    msg!("Successful deposit of {:?}", transfer_amount);

    Ok(())
}

pub fn process_withdraw(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let withdraw_amount: u64 = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

    assert!(
        pda_account.is_writable,
        "Deposit account must be writable"
    );

    // Check if pda account has enough lamport.
    // TODO: Check if PDA account seeds correspond to withdrawer account

    let pda_data = pda_account.data.borrow();
    if pda_data.len() < 8 {
        return Err(ProgramError::InvalidAccountData);
    }
    let pda_balance = u64::from_le_bytes(pda_data[0..8].try_into().unwrap());
    if (pda_balance < withdraw_amount) {
        return Err(ProgramError::InsufficientFunds);
    }

    let transfer_instruction = solana_program::system_instruction::transfer(
        pda_account.key,
        user_account.key,
        withdraw_amount,
    );
    invoke(
        &transfer_instruction,
        &[pda_account.clone(), user_account.clone(), system_program.clone()],
    )?;

    msg!("Successful withdraz of {:?}", withdraw_amount);
    Ok(())
}