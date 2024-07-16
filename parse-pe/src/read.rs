use core::array::TryFromSliceError;

pub struct Reader<'a> {
    bytes: &'a [u8],
    idx: usize,
}

impl<'a> Reader<'a> {
    pub fn peek<P: Primitive>(&self) -> Result<P, ReaderError> {
        let end = core::mem::size_of::<P>();
        P::read(self.bytes.get(self.idx..end)
            .ok_or(ReaderError::OutOfBounds(self.idx, self.bytes.len()))?)
    }

    pub fn read<P: Primitive>(&mut self) -> Result<P, ReaderError> {
        // Read the value
        let value = self.peek::<P>()?;
        // If the read was successful, move the cursor
        self.idx += core::mem::size_of::<P>();
        // Return the value
        Ok(value)
    }

    /// Peek `len` bytes from the underlying data support. Returns a slice containing the desired
    /// amount and `None` otherwise.
    pub fn peek_bytes(&self, len: usize) -> Option<&[u8]> {
        let end = self.idx.saturating_add(len);
        self.bytes.get(self.idx..end)
    }

    /// Read `len` bytes from the underlying data support, moving the cursor forward by `len` bytes
    /// in case of success. Returns a slice containing the desired amount or `OutOfBounds` error
    /// otherwise.
    pub fn read_bytes(&mut self, len: usize) -> Result<&[u8], ReaderError> {
        let end = self.idx.saturating_add(len);
        let bytes = self.bytes.get(self.idx..end)
            .ok_or(ReaderError::OutOfBounds(self.idx, self.bytes.len()))?;
        self.idx += bytes.len();
        Ok(bytes)
    }

    /// Skip `count` bytes from the current index position and set the index value as that new
    /// position. If the `len` is bigger than the amount of bytes available from current cursor,
    /// the cursor will just be set as moving to the end of the buffer
    pub fn skip(&mut self, len: usize) {
        let end = self.idx.saturating_add(len);
        self.idx = core::cmp::min(end, self.bytes.len());
    }

    /// Seek the index to the desired `offset`. If the `offset` is not within the bounds, return
    /// an error.
    pub fn seek(&mut self, offset: usize) -> Result<(), ReaderError> {
        if offset > self.bytes.len() {
            return Err(ReaderError::OutOfBounds(offset, self.bytes.len()));
        }

        self.idx = offset;
        Ok(())
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
    OutOfBounds(usize, usize),
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

#[macro_export]
macro_rules! read_impl {
    ($typ:ty) => {
        impl Primitive for $typ {
            fn read(data: &[u8]) -> Result<Self, ReaderError> {
                let len = core::mem::size_of::<Self>();
                let bytes = data.get(..len).ok_or(ReaderError::UnsufficientBytes(len, data.len()))?;
                let value = <$typ>::from_le_bytes(bytes.try_into()?);
                Ok(value)
            }
        }
    };
}

read_impl!(u8);
read_impl!(u16);
read_impl!(u32);
read_impl!(u64);
read_impl!(i8);
read_impl!(i16);
read_impl!(i32);
read_impl!(i64);
