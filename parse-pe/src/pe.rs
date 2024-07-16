use crate::read::{Reader, ReaderError};

pub const MZ_MAGIC: &[u8; 2] = b"MZ";

#[derive(Debug)]
pub struct Pe;

#[derive(Debug)]
pub enum PeError {
    ReaderError(ReaderError),
    MZMagic,
}

impl From<ReaderError> for PeError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}

impl Pe {
    pub fn parse(reader: &mut Reader) -> Result<Pe, PeError> {
        // Check MZ
        let mz = reader.read_bytes(MZ_MAGIC.len())?;

        if mz != MZ_MAGIC {
            return Err(PeError::MZMagic);
        }

        // Go to the PE offset.

        Ok(Self)
    }
}
