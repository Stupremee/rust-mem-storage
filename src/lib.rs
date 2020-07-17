//! # mem-storage
//!
//! mem-storage is an abstraction over a chunk of memory, that is readable and writable.
//! It can be used in in everything, that requires some sort of memory, e.g. the RAM in an emulator.
//!
//! ## Motivation
//!
//! Every time I write an emulator, I don't like to make
//! a `struct Memory` over and over again and always copy paste methods like
//! `read_u8`, `read_u16`, etc. So I came up with a generic solution for this problem.
//!
//! ## Usage
//!
//! There are two ways to use the crate.
//!
//! First one wraps all methods of a `Memory` in a new struct,
//! that also handles paging, alignment, whatever.
//!
//! The second one is to implement `Memory` for your own struct.
//!
//! ### Use the Memory trait
//!
//! ```
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
//! ### Wrap a Memory implementation
//! ```
//! use mem_storage::{Memory, VecMemory, Value};
//!
//! /// This is your memory struct for the emulator, that wraps methods
//! /// of the `Memory` trait, and can check for alignment, paging, etc.
//! struct Memory {
//!   /// The `VecMemory` implementaiton stores the whole memory in a `Vec`.
//!   inner: VecMemory,
//! }
//!
//! impl Memory {
//!   fn new() -> Self {
//!     /// Create 1KiB of zero initialized memory.
//!     Self { memory: VecMemory::new(1024 * 1024) }
//!   }
//!
//!   /// The `Value` trait is implemented for all primitive number types that can be
//!   /// read from or written to memory.
//!   fn read<V: Value>(&self, addr: usize) -> V {
//!     // translate to physical address
//!     // ...
//!
//!     self.inner.read::<V>(addr)
//!   }
//!
//!   fn write<V: Value>(&mut self, addr: usize, value: V) {
//!     // translate to physical address
//!     // ...
//!
//!     self.inner.write(addr, value);
//!   }
//! }
//! ```
//!
//! ### Implement the Memory trait
//!
//! ```
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

#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(clippy::all)]

/// The `Memory` trait represents a chunk of memory that can read from,
/// or written to.
pub trait Memory {
    /// The `Error` type can be used to indicate if memory access was invalid.
    ///
    /// Usually this is just `()` and if `Err(())` is returned, it means that the address is out of bounds.
    type Error: std::fmt::Debug;

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
    fn read<V: Value>(&self, addr: usize) -> Result<V, Self::Error> {
        todo!()
    }

    /// Tries to write a generic `Value` to the given address using little endian format.
    ///
    /// Returns `Err(x)` if the method failed to write a value to the address.
    fn write<V: Value>(&self, addr: usize, val: V) -> Result<(), Self::Error> {
        todo!()
    }
}

macro_rules! impl_trait {
    ($trait:path, $($ty:path),*) => {
        $(
        impl $trait for $ty {}
        )*
    };
}

/// A marker trait that is implemented for all number types that can be read from and written to
/// a `Memory`.
pub trait Value: private::Sealed {}

impl_trait!(Value, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

mod private {
    pub trait Sealed {}

    impl_trait!(Sealed, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
}
