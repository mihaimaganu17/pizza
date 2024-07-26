#![no_std]

use core::array::TryFromSliceError;

pub struct Reader<'a> {
    bytes: &'a [u8],
    idx: usize,
}

impl<'a> Reader<'a> {
    pub fn peek<P: Primitive>(&self) -> Result<P, ReaderError> {
        let end = self.idx.saturating_add(core::mem::size_of::<P>());
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

    /// Returns the current position of the cursor
    pub fn offset(&self) -> usize {
        self.idx
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
    InsufficientBytes(usize, usize),
    OutOfBounds(usize, usize),
    TryFromSliceError(TryFromSliceError),
    Infallible(core::convert::Infallible),
}

impl From<TryFromSliceError> for ReaderError {
    fn from(err: TryFromSliceError) -> Self {
        Self::TryFromSliceError(err)
    }
}

impl From<core::convert::Infallible> for ReaderError {
    fn from(err: core::convert::Infallible) -> Self {
        Self::Infallible(err)
    }
}

pub trait Primitive: Sized {
    fn read(data: &[u8]) -> Result<Self, ReaderError>;
    // Returns the size on disk of the type implementing this trait. This is equivalent to the size
    // of the structure in memory with align(1) -> alignment by 1 byte
    fn size_on_disk(&self) -> usize;
}

#[macro_export]
macro_rules! read_impl {
    ($typ:ty) => {
        impl Primitive for $typ {
            fn read(data: &[u8]) -> Result<Self, ReaderError> {
                let len = core::mem::size_of::<Self>();
                let bytes = data.get(..len).ok_or(ReaderError::InsufficientBytes(len, data.len()))?;
                let value = <$typ>::from_le_bytes(bytes.try_into()?);
                Ok(value)
            }
            fn size_on_disk(&self) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    };
    // Array handling
    ($typ:ty, $size:literal) => {
        impl Primitive for [$typ; $size] {
            fn read(data: &[u8]) -> Result<Self, ReaderError> {
                let mut reader = Reader::from(data);
                let mut res = [0; $size];
                for elem in res.iter_mut() {
                    *elem = reader.read::<$typ>()?;
                }
                Ok(res)
            }
            fn size_on_disk(&self) -> usize {
                core::mem::size_of::<Self>()
            }
        }
    };
}

// TODO: Macro to auto generate for arrays

read_impl!(u8);
read_impl!(u8, 1);
read_impl!(u8, 2);
read_impl!(u8, 8);
read_impl!(u8, 10);
read_impl!(u16);
read_impl!(u16, 6);
read_impl!(u32);
read_impl!(u32, 1);
read_impl!(u32, 2);
read_impl!(u64);
read_impl!(i8);
read_impl!(i16);
read_impl!(i32);
read_impl!(i64);
