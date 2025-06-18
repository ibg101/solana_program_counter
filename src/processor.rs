use solana_program::{
    msg,
    rent::Rent,
    sysvar::Sysvar,
    program::invoke_signed,
    instruction::Instruction,
    system_instruction,
    system_program,
    pubkey::Pubkey,
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    program_pack::Pack,
    program_error::ProgramError
};
use super::{
    state::CounterPDA,
    instruction::CounterInstruction
};


pub struct Processor;

impl Processor {
    const COUNTER_ACCOUNT_SPACE: usize = 10;  // counter.value + counter.is_initialized + counter.bump
    
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8]
    ) -> ProgramResult {
        let instruction: CounterInstruction = CounterInstruction::unpack(data)?; 
        
        match instruction {
            CounterInstruction::InitializeCounter => Self::process_initialize_counter(program_id, accounts)?,
            CounterInstruction::IncrementCounter { increment_by } => Self::process_increment_counter(program_id, accounts, increment_by)?,
            CounterInstruction::CloseCounter => Self::process_close_counter(program_id, accounts)?
        };

        Ok(())
    }

    fn process_initialize_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        
        let payer_account: &AccountInfo = next_account_info(accounts_iter)?;
        let pda_account: &AccountInfo = next_account_info(accounts_iter)?;
        let system_account: &AccountInfo = next_account_info(accounts_iter)?;
        
        let rent_exemp: u64 = Rent::get()?.minimum_balance(Self::COUNTER_ACCOUNT_SPACE);
        let (seed1, seed2) = (b"counter", payer_account.key.as_ref());
        let (_, bump) = Pubkey::find_program_address(
            &[seed1, seed2], 
            program_id
        );
        let signers_seeds: &[&[u8]] = &[seed1, seed2, &[bump]];

        let create_account_ix: Instruction = system_instruction::create_account(
            payer_account.key, 
            pda_account.key, 
            rent_exemp, 
            Self::COUNTER_ACCOUNT_SPACE as u64, 
            program_id
        );
        invoke_signed(
            &create_account_ix,
            &[
                payer_account.clone(),
                pda_account.clone(),
                system_account.clone()
            ],
            &[signers_seeds]
        )?;
        msg!("Create Counter Account - success");

        let counter_pda: CounterPDA = CounterPDA::new(0, bump);
        let pda_data: &mut [u8] = &mut **pda_account.data.borrow_mut();
        counter_pda.pack_into_slice(pda_data);
        msg!("Initialize Account's State - success");

        Ok(())
    }

    fn process_increment_counter(program_id: &Pubkey, accounts: &[AccountInfo], increment_by: u64) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let payer_account: &AccountInfo = next_account_info(accounts_iter)?;
        let pda_account: &AccountInfo = next_account_info(accounts_iter)?;

        if pda_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let pda_data: &mut [u8] = &mut **pda_account.data.borrow_mut();
        let bump: u8 = pda_data[9..10][0];
        // note that we use create_program_address instead of find_program_address, because we already have a valid bump and it will reduce
        // iterations amount needed to find the canonical bump.
        let expected_pda: Pubkey = Pubkey::create_program_address(
            &[b"counter", payer_account.key.as_ref(), &[bump]], 
            program_id
        )?;

        if pda_account.key != &expected_pda {
            return Err(ProgramError::InvalidSeeds);
        }

        // ----> This approach costs 1200 CU.
        
        // let mut counter_pda: CounterPDA = CounterPDA::unpack(&pda_data)?;
        // msg!("Counter.value before: {}", counter_pda.value);
        // counter_pda.value = counter_pda.value
        //     .checked_add(increment_by)
        //     .ok_or(ProgramError::ArithmeticOverflow)?;
        // msg!("Counter.value after: {}", counter_pda.value);
        // counter_pda.pack_into_slice(pda_data);

        // -------------------------------------

        // ----> This approach costs 600 CU, which is 2x boost!

        let (value_slice, _rest) = pda_data.split_at_mut(8); 
        let value: u64 = u64::from_le_bytes(
            value_slice.try_into().map_err(|_| ProgramError::InvalidAccountData)?
        );
        match value.checked_add(increment_by) {
            Some(_) => {
                value_slice.copy_from_slice(&increment_by.to_le_bytes());
                msg!("Incremented manually value!");
            },
            None => return Err(ProgramError::ArithmeticOverflow)
        }
        
        // -------------------------------------

        Ok(())
    }

    fn process_close_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let _payer_account: &AccountInfo = next_account_info(accounts_iter)?;
        let pda_account: &AccountInfo = next_account_info(accounts_iter)?;

        if pda_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let recipient_account: &AccountInfo = next_account_info(accounts_iter)?;

        // 1. reallocate space (use .resize() instead, but in solana-account-info 2.2.1 it's not available yet)
        pda_account.realloc(0, true)?;
        
        // 2. transfer rent exempt to recipient
        let pda_balance: u64 = pda_account.lamports();
        let recipient_balance: u64 = recipient_account.lamports();
        **recipient_account.lamports.borrow_mut() = recipient_balance
            .checked_add(pda_balance)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        **pda_account.lamports.borrow_mut() = 0;

        // 3. assign SystemProgram as a new owner
        pda_account.assign(&system_program::ID);

        Ok(())
    }
}