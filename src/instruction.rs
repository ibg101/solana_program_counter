use solana_program::program_error::ProgramError;


/// API for all available instructions.
pub enum CounterInstruction {
    InitializeCounter,
    IncrementCounter { increment_by: u64 }
}

impl CounterInstruction {
    pub fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let (instr_type, rest) = data.split_at(1);

        Ok(match instr_type[0] {
            0 => Self::InitializeCounter,
            1 => {
                let increment_by: u64 = u64::from_le_bytes(
                    rest.try_into().map_err(|_| ProgramError::InvalidInstructionData)?
                );
                Self::IncrementCounter { increment_by }
            },
            _ => return Err(ProgramError::InvalidInstructionData) 
        })
    }
}