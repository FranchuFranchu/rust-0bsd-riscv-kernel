use crate::Handle;

pub fn wait_for_interrupt(id: u32) -> Result<(), (usize, [usize; 2])> {
    let handle = Handle::open(4, &[id as usize])?;
    handle.read(&mut [], &[])?;
    Ok(())
}
