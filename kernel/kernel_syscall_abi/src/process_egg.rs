use flat_bytes::Flat;

#[derive(Flat)]
#[repr(u8)]
pub enum ProcessEggPacketHeader {
    Entry(usize),
    Memory(usize),
    Name(),
    Hatch,
}

#[derive(Debug, AsRegister)]
pub enum ProcessEggError {
    Dummy,
}
