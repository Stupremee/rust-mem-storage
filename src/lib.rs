//! # mem-storage
//!
//! mem-storage is an abstraction over a chunk of memory, that is readable and writable.
//! It can be used in in everything, that requires some sort of memory, e.g. the RAM in an emulator.
//! This crate can also be used in no_std environment.
//!
//! ## Motivation
//!
//! Every time I write an emulator, I don't like to make
//! a `struct Memory` over and over again and always copy paste methods like
//! `read_u8`, `read_u16`, etc. So I came up with a generic solution for this problem.
//!
//! ## Usage
//!
//! ### Use the Memory trait
//!
//! ```rust
//! use mem_storage::{Memory, VecMemory};
//!
//! /// Create 1 KiB of memory
//! let mem = VecMemory::new(1024 * 1024);
//!
//! /// The `read` and `write` method will read / write data using little endian format.
//! /// For big endian format use `read_be` and `write_be`.
//! mem.write(0xABCD, 123);
//!
//! let value = mem.read::<u8>(0xABCD);
//! assert_eq!(123u8, value);
//!
//! mem.write(0x1000, 12345u64);
//!
//! let value = mem.read::<u64>(0x1000);
//! assert_eq!(12345u64, value);
//!
//! mem.write_be(0x2000, 1234567u64);
//!
//! let value = mem.read_be::<u64>(0x2000);
//! assert_eq!(1234567u64, value);
//! ```
//!
//! ### Implement the Memory trait
//!
//! ```rust
//! /// This time your struct is responsible for storing the data.
//! struct Memory {
//!   ram: Vec<u8>,
//! }
//!
//! impl Memory {
//!   fn new() -> Self {
//!     // Create 1KiB of zero initialized memory
//!     Self { ram: vec![0u8; 1024 * 1024] }
//!   }
//! }
//!
//! impl mem_storage::Memory for Memory {
//!   /// If an `Err` is returned, the addr is out of bounds
//!   type Error = ();
//!
//!   fn try_read_byte(&self, addr: usize) -> Result<u8, Self::Error> {
//!     self.ram.get(addr).map_err(|_| ())
//!   }
//!
//!   fn try_write_byte(&self, addr: usize, value: u8) -> Result<(), Self::Error> {
//!     let mut value = self.ram.get(addr).map_err(|_| ())?;
//!     *value = value;
//!   }
//!
//!   /// The trait will provide a generic `read` and `read_be` method for you.
//! }
//! ```
//!
//! ## License
//!
//! This project is double-licensed under the Zlib or Apache2.0 license.

#![no_std]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(clippy::all)]

use core::slice::SliceIndex;

/// The `Memory` trait represents a chunk of memory that can read from,
/// or written to.
pub trait Memory {
    /// The `Error` type can be used to indicate if memory access was invalid.
    ///
    /// Usually this is just `()` and if `Err(())` is returned, it means that the address is out of bounds.
    type Error: core::fmt::Debug;

    /// Returns a reference to an element or subslice depending on the type of
    /// index.
    fn get<I>(&self, index: I) -> Result<&I::Output, Self::Error>
    where
        I: SliceIndex<[u8]>;

    /// Returns a mutable reference to an element or subslice depending on the type of
    /// index.
    fn get_mut<I>(&self, index: I) -> Result<&mut I::Output, Self::Error>
    where
        I: SliceIndex<[u8]>;

    /// Tries to read a byte at the given address.
    ///
    /// Returns `Err(x)` if the method failed to read a byte from the address.
    fn try_read_byte(&self, addr: usize) -> Result<u8, Self::Error>;

    /// Tries to write a byte to the given address.
    ///
    /// Returns `Err(x)` if the method failed to write a byte to the address.
    fn try_write_byte(&mut self, addr: usize, byte: u8) -> Result<(), Self::Error>;

    /// Reads a byte at the given address.
    ///
    /// Panics if the read failed
    fn read_byte(&self, addr: usize) -> u8 {
        self.try_read_byte(addr)
            .expect("failed to read from memory")
    }

    /// Writes a byte to the given address.
    ///
    /// Panics if the write failed
    fn write_byte(&mut self, addr: usize, byte: u8) {
        self.try_write_byte(addr, byte)
            .expect("failed to write to memory")
    }

    /// Tries to read a generic `Value` at the given address using little endian format.
    ///
    /// Returns `Err(x)` if the method failed to read a value at the address.
    fn try_read<V: Value>(&self, addr: usize) -> Result<V, Self::Error> {
        let size = core::mem::size_of::<V>();
        let slice = self.get(addr..addr + size)?;

        // Safety: `Value` is only implemented for all primitive number types, and can not be implemented
        // for any other types. Thus a transmute between raw bytes and a `Value` is safe.
        // The length of the `slice` is checked before this method is called.
        let value = unsafe {
            debug_assert_eq!(core::mem::size_of::<V>(), slice.len());
            let slice = core::slice::from_raw_parts(slice.as_ptr() as *const V, 1);
            slice[0].to_le()
        };

        Ok(value)
    }

    /// Reads a generic `Value` at the given address using little endian format.
    ///
    /// Panics if the method failed to read a value at the address.
    fn read<V: Value>(&self, addr: usize) -> V {
        self.try_read::<V>(addr).expect("failed to read memory")
    }

    /// Tries to read a generic `Value` at the given address using big endian format.
    ///
    /// Returns `Err(x)` if the method failed to read a value at the address.
    fn try_read_be<V: Value>(&self, addr: usize) -> Result<V, Self::Error> {
        self.try_read(addr).map(Value::to_be)
    }

    /// Reads a generic `Value` at the given address using big endian format.
    ///
    /// Panics if the method failed to read a value at the address.
    fn read_be<V: Value>(&self, addr: usize) -> V {
        self.read::<V>(addr).to_be()
    }

    /// Tries to write a generic `Value` to the given address using little endian format.
    ///
    /// Returns `Err(x)` if the method failed to write a value to the address.
    fn try_write<V: Value>(&self, addr: usize, val: V) -> Result<(), Self::Error> {
        let size = core::mem::size_of::<V>();
        let val = val.to_le();
        let slice = self.get_mut(addr..addr + size)?;

        // Safety: `Value` is only implemented for all primitive number types, and can not be implemented
        // for any other types. Thus a transmute between raw bytes and a `Value` is safe.
        let raw_value = unsafe {
            let ptr: *const V = &val;
            core::slice::from_raw_parts(ptr as *const u8, size)
        };
        slice.copy_from_slice(raw_value);
        Ok(())
    }

    /// Writes a generic `Value` to the given address using little endian format.
    ///
    /// Panics if the method failed to write a value to the address.
    fn write<V: Value>(&self, addr: usize, val: V) {
        self.try_write::<V>(addr, val)
            .expect("failed to write memory")
    }

    /// Tries to write a generic `Value` to the given address using big endian format.
    ///
    /// Returns `Err(x)` if the method failed to write a value to the address.
    fn try_write_be<V: Value>(&self, addr: usize, val: V) -> Result<(), Self::Error> {
        self.try_write(addr, val.to_be())
    }

    /// Writes a generic `Value` to the given address using big endian format.
    ///
    /// Panics if the method failed to write a value to the address.
    fn write_be<V: Value>(&self, addr: usize, val: V) {
        self.write(addr, val.to_be());
    }
}

macro_rules! impl_trait {
    ($($ty:path),*) => {
        $(
            impl Value for $ty {
                fn to_le(self) -> Self {
                    self.to_le()
                }

                fn to_be(self) -> Self {
                    self.to_be()
                }
            }
        )*
    };
}

/// A marker trait that is implemented for all number types that can be read from and written to
/// a `Memory`.
pub trait Value: private::Sealed + Sized + Copy {
    /// Converts `self` to little endian format.
    fn to_le(self) -> Self;

    /// Converts `self` to big endian format.
    fn to_be(self) -> Self;
}

impl_trait!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

mod private {
    pub trait Sealed {}

    macro_rules! impl_trait {
        ($($ty:path),*) => {
            $(impl Sealed for $ty {})*
        };
    }

    impl_trait!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
}
