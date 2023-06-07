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

// #[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
// pub struct Counter {
//     pub count: u64,
// }

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
            process_deposit(_program_id, accounts, instruction_data_inner)?;
        }
        1 => {
            msg!("Instruction: Withdraw");
            process_withdraw(_program_id, accounts, instruction_data_inner)?;
        }
        _ => {
            msg!("Error: unknown instruction")
        }
    }
    Ok(())
}

pub fn process_deposit(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let transfer_amount: u64 = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

    assert!(pda_account.is_writable, "Deposit account must be writable");

    // Creating transfer instruction
    let transfer_instruction = solana_program::system_instruction::transfer(
        user_account.key,
        pda_account.key,
        transfer_amount,
    );

    // Transfering sol to pda account
    invoke(
        &transfer_instruction,
        &[
            user_account.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
    )?;

    msg!("Successful deposit of {:?}", transfer_amount);

    Ok(())
}

pub fn process_withdraw(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let user_account = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let withdraw_amount: u64 = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

    assert!(pda_account.is_writable, "Deposit account must be writable");

    // Wrong account
    if pda_account.owner != _program_id {
        msg!("Greeted account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check if user is associated with PDA
    let user_key = user_account.key;
    let pda_key = pda_account.key;
    if !pda_key.to_bytes().starts_with(&user_key.to_bytes()[..32]) {
        return Err(ProgramError::InvalidAccountData);
    }

    // 8 first octal is for pda balance
    let pda_data = pda_account.data.borrow();
    if pda_data.len() < 8 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if pda account has enough lamport.
    let pda_balance = u64::from_le_bytes(pda_data[0..8].try_into().unwrap());
    if pda_balance < withdraw_amount {
        return Err(ProgramError::InsufficientFunds);
    }

    // Withdraw sol to user
    let transfer_instruction = solana_program::system_instruction::transfer(
        pda_account.key,
        user_account.key,
        withdraw_amount,
    );
    invoke(
        &transfer_instruction,
        &[
            pda_account.clone(),
            user_account.clone(),
            system_program.clone(),
        ],
    )?;

    msg!("Successful withdraw of {:?}", withdraw_amount);
    Ok(())
}
