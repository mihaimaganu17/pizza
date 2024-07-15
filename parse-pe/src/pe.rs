use crate::read::{Reader, ReaderError};

#[derive(Debug)]
pub struct Pe;

#[derive(Debug)]
pub enum PeError {
    ReaderError(ReaderError),
}

impl From<ReaderError> for PeError {
    fn from(err: ReaderError) -> Self {
        Self::ReaderError(err)
    }
}

impl Pe {
    pub fn parse(reader: &mut Reader) -> Result<Pe, PeError> {
        reader.read::<u32>()?;
        Ok(Self)
    }
}
