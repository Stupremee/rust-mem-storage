# mem-storage

mem-storage is an abstraction over a chunk of memory, that is readable and writable.
It can be used in in everything, that requires some sort of memory, e.g. the RAM in an emulator.

## Motivation

Every time I write an emulator, I don't like to make 
a `struct Memory` over and over again and always copy paste methods like
`read_u8`, `read_u16`, etc. So I came up with a generic solution for this problem.

## Usage

There are two ways to use the crate.

First one wraps all methods of a `Memory` in a new struct,
that also handles paging, alignment, whatever.

The second one is to implement `Memory` for your own struct.

### Use the Memory trait

```rust
use mem_storage::{Memory, VecMemory};

/// Create 1 KiB of memory
let mem = VecMemory::new(1024 * 1024);

/// The `read` and `write` method will read / write data using little endian format.
/// For big endian format use `read_be` and `write_be`.
mem.write(0xABCD, 123);

let value = mem.read::<u8>(0xABCD);
assert_eq!(123u8, value);

mem.write(0x1000, 12345u64);

let value = mem.read::<u64>(0x1000);
assert_eq!(12345u64, value);

mem.write_be(0x2000, 1234567u64);

let value = mem.read_be::<u64>(0x2000);
assert_eq!(1234567u64, value);
```

### Wrap a Memory implementation
```rust
use mem_storage::{Memory, VecMemory, Value};

/// This is your memory struct for the emulator, that wraps methods
/// of the `Memory` trait, and can check for alignment, paging, etc.
struct Memory {
  /// The `VecMemory` implementaiton stores the whole memory in a `Vec`.
  inner: VecMemory,
}

impl Memory {
  fn new() -> Self {
    /// Create 1KiB of zero initialized memory.
    Self { memory: VecMemory::new(1024 * 1024) }
  }

  /// The `Value` trait is implemented for all primitive number types that can be
  /// read from or written to memory.
  fn read<V: Value>(&self, addr: usize) -> V {
    // translate to physical address
    // ...

    self.inner.read::<V>(addr)
  }

  fn write<V: Value>(&mut self, addr: usize, value: V) {
    // translate to physical address
    // ...

    self.inner.write(addr, value);
  }
}
```

### Implement the Memory trait

```rust
/// This time your struct is responsible for storing the data.
struct Memory {
  ram: Vec<u8>,
}

impl Memory {
  fn new() -> Self {
    // Create 1KiB of zero initialized memory
    Self { ram: vec![0u8; 1024 * 1024] }
  }
}

impl mem_storage::Memory for Memory {
  /// If an `Err` is returned, the addr is out of bounds
  type Error = ();

  fn try_read_byte(&self, addr: usize) -> Result<u8, Self::Error> {
    self.ram.get(addr).map_err(|_| ())
  } 

  fn try_write_byte(&self, addr: usize, value: u8) -> Result<(), Self::Error> {
    let mut value = self.ram.get(addr).map_err(|_| ())?;
    *value = value;
  } 

  /// The trait will provide a generic `read` and `read_be` method for you.
}
```

## License

This project is double-licensed under the Zlib or Apache2.0 license.
