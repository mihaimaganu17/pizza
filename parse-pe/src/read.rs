use core::array::TryFromSliceError;

pub struct Reader<'a> {
    bytes: &'a [u8],
    idx: usize,
}

impl<'a> Reader<'a> {
    fn peek<P: Primitive>(&mut self) -> Result<P, ReaderError> {
        P::read(self.bytes.get(self.idx..)
            .ok_or(ReaderError::IdxOverflow(self.idx, self.bytes.len()))?)
    }

    pub fn read<P: Primitive>(&mut self) -> Result<P, ReaderError> {
        // Read the value
        let value = self.peek::<P>()?;
        // If the read was successful, move the cursor
        self.idx += core::mem::size_of::<P>();
        // Return the value
        Ok(value)
    }
}

impl<'a> From<&'a [u8]> for Reader<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            idx: 0,
        }
    }
}

#[derive(Debug)]
pub enum ReaderError {
    UnsufficientBytes(usize, usize),
    IdxOverflow(usize, usize),
    TryFromSliceError(TryFromSliceError),
}

impl From<TryFromSliceError> for ReaderError {
    fn from(err: TryFromSliceError) -> Self {
        Self::TryFromSliceError(err)
    }
}

pub trait Primitive: Sized {
    fn read(data: &[u8]) -> Result<Self, ReaderError>;
}

impl Primitive for u32 {
    fn read(data: &[u8]) -> Result<Self, ReaderError> {
        let len = core::mem::size_of::<Self>();
        let bytes = data.get(..len).ok_or(ReaderError::UnsufficientBytes(len, data.len()))?;
        let value = u32::from_le_bytes(bytes.try_into()?);
        Ok(value)
    }
}
