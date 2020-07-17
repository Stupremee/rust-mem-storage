# mem-storage

mem-storage is an abstraction over a chunk of memory, that is readable and writable.
It can be used in in everything, that requires some sort of memory, e.g. the RAM in an emulator.
This crate can also be used in no_std environment.

## Motivation

Every time I write an emulator, I don't like to make
a `struct Memory` over and over again and always copy paste methods like
`read_u8`, `read_u16`, etc. So I came up with a generic solution for this problem.

## Usage

### Use the Memory trait

```compile_fail
use mem_storage::Memory;

let mem = MyMemory::new();

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

### Implement the Memory trait

```
use mem_storage::Memory;

/// This time your struct is responsible for storing the data.
struct MyMemory {
  ram: Vec<u8>,
}

impl MyMemory {
  fn new() -> Self {
    // Create 1KiB of zero initialized memory
    Self { ram: vec![0u8; 1024 * 1024] }
  }
}

impl Memory for MyMemory {
  /// If an `Err` is returned, the addr is out of bounds
  type Error = ();

  fn get<I>(&self, index: I) -> Result<&I::Output, Self::Error>
  where
      I: std::slice::SliceIndex<[u8]>,
  {
      self.ram.get(index).ok_or(())
  }

  fn get_mut<I>(&mut self, index: I) -> Result<&mut I::Output, Self::Error>
  where
      I: std::slice::SliceIndex<[u8]>,
  {
      self.ram.get_mut(index).ok_or(())
  }

  fn try_read_byte(&self, addr: usize) -> Result<u8, Self::Error> {
    self.ram.get(addr).copied().ok_or(())
  }

  fn try_write_byte(&mut self, addr: usize, value: u8) -> Result<(), Self::Error> {
    let mut value = self.ram.get_mut(addr).ok_or(())?;
    *value = *value;
    Ok(())
  }

  // The trait will provide a generic `read` and `read_be` method for you.
}
```

## License

This project is double-licensed under the Zlib or Apache2.0 license.
