use super::*;

/// SAFETY: It's safe if root is a valid pointer
/// and paging is disabled
/// Otherwise, it can remap things the wrong way and break everything
pub unsafe fn identity_map(root: *mut Table) {
	for (idx, i) in ((*root).entries).iter_mut().enumerate() {
		i.value = (EntryBits::Read as usize) | (EntryBits::Write as usize) | (EntryBits::Execute as usize) | (EntryBits::Valid as usize)| (GIGAPAGE_SIZE / 4 * idx)
	}
}