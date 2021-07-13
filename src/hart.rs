use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;

use crate::{cpu::load_hartid, plic::{Plic0}};


// Data associated with a hart
pub struct HartMeta {
	pub plic: Plic0,
}

pub static HART_META: RwLock<BTreeMap<usize, Arc<HartMeta>>> = RwLock::new(BTreeMap::new());

pub fn get_hart_meta(hartid: usize) -> Option<Arc<HartMeta>> {
	HART_META.read().get(&hartid).map(|s| s.clone())
}

// Only run this from the boot hart
pub fn add_boot_hart() {
	HART_META.write().insert(load_hartid(), Arc::new(HartMeta { plic: Plic0::new_with_fdt() }));
}

pub fn get_this_hart_meta() -> Option<Arc<HartMeta>> {
	get_hart_meta(load_hartid())
}