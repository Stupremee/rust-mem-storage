use mem_storage::Memory;

struct TestMemory {
    ram: Vec<u8>,
}

impl TestMemory {
    fn new<S: AsRef<[u8]>>(slice: S) -> Self {
        Self {
            ram: slice.as_ref().into(),
        }
    }
}

impl Memory for TestMemory {
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
        self.get(addr).map(Clone::clone)
    }

    fn try_write_byte(&mut self, addr: usize, byte: u8) -> Result<(), Self::Error> {
        let entry = self.get_mut(addr)?;
        *entry = byte;
        Ok(())
    }
}

#[test]
fn test_read_le() {
    let mem = TestMemory::new([0xBA, 0xCD, 0xAB, 0x00, 0x00]);
    assert_eq!(mem.read::<u8>(0), 0xBAu8);
    assert_eq!(mem.read::<u32>(1), 0xABCDu32);
}

#[test]
fn test_write_le() {
    let mut mem = TestMemory::new([0u8; 16]);

    mem.write::<u8>(0, 0xFF);
    assert_eq!(mem.read::<u8>(0), 0xFFu8);

    mem.write::<u32>(4, 0xDDFFEEAAu32);
    let bytes = mem.get(4..8).unwrap();
    assert_eq!(bytes, &[0xAAu8, 0xEE, 0xFF, 0xDD]);

    assert_eq!(mem.read::<u32>(4), 0xDDFFEEAAu32);
}

#[test]
fn test_read_be() {
    let mem = TestMemory::new([0xBA, 0xCD, 0xAB, 0x00, 0x00]);
    assert_eq!(mem.read_be::<u8>(0), 0xBAu8);
    assert_eq!(mem.read_be::<u32>(1), 0xCDAB0000u32);
}

#[test]
fn test_write_be() {
    let mut mem = TestMemory::new([0u8; 16]);

    mem.write_be::<u8>(0, 0xFF);
    assert_eq!(mem.read_be::<u8>(0), 0xFFu8);

    mem.write_be::<u32>(4, 0xDDFFEEAAu32);
    let bytes = mem.get(4..8).unwrap();
    assert_eq!(bytes, &[0xDDu8, 0xFF, 0xEE, 0xAA]);

    assert_eq!(mem.read_be::<u32>(4), 0xDDFFEEAAu32);
}
