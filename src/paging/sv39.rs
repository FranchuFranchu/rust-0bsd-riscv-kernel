use super::*;

// SAFETY: It's safe if root is a valid pointer
// and paging is disabled
// Otherwise, it can remap things the wrong way and break everything
pub unsafe fn identity_map(root: *mut Table) {
	for (idx, i) in ((*root).entries).iter_mut().enumerate() {
		i.value = (EntryBits::Global as usize) | (EntryBits::Read as usize) | (EntryBits::Execute as usize) | (EntryBits::Valid as usize) | (EntryBits::Write as usize) | GIGAPAGE_SIZE * idx * 4;
	}
}