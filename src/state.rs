use solana_program::{
    program_error::ProgramError,
    program_pack::{Pack, Sealed, IsInitialized}
};


#[repr(C)]
pub struct CounterPDA {
    pub value: u64,
    is_initialized: bool,
    pub bump: u8,
}

#[allow(dead_code)]
impl CounterPDA {
    pub fn new(value: u64, bump: u8) -> Self {
        Self { value, is_initialized: true, bump }
    }
}

impl IsInitialized for CounterPDA {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for CounterPDA {}

impl Pack for CounterPDA {
    const LEN: usize = 10;

    fn pack_into_slice(&self, dst: &mut [u8]) -> () {
        dst[..8].copy_from_slice(&self.value.to_le_bytes());
        dst[8..10].copy_from_slice(&[self.is_initialized as u8, self.bump]);
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let (value_slice, is_initialized_and_bump_slice) = src.split_at(8);
        let value: u64 = u64::from_le_bytes(value_slice.try_into().map_err(|_| ProgramError::InvalidAccountData)?);
        let is_initialized: bool = if is_initialized_and_bump_slice[0] == 0 { false } else { true };
        let bump: u8 = is_initialized_and_bump_slice[1];
        Ok(Self { value, is_initialized, bump })
    }
}