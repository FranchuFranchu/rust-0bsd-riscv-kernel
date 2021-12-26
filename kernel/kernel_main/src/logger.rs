//! A logger for the kernel

use log::{Level, Metadata, Record};

use crate::{lock::shared::Mutex, trap::in_interrupt_context};

pub struct ColorfulLogger {
    lock: Mutex<()>,
}

impl log::Log for ColorfulLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let prefix = match record.level() {
                Level::Error => "\x1b[91;1mERROR\x1b[0m",
                Level::Warn => "\x1b[93;1mWARN \x1b[0m",
                Level::Info => "\x1b[1mINFO \x1b[0m",
                Level::Debug => "\x1b[1;96mDEBUG\x1b[0m",
                Level::Trace => "\x1b[96mTRACE\x1b[0m",
            };

            let _guard = if !in_interrupt_context() {
                Some(self.lock.lock())
            } else {
                None
            };
            println!(
                "{} [{}] {}",
                &record.module_path().unwrap_or("")["rust_0bsd_riscv_kernel".len()..],
                prefix,
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub static KERNEL_LOGGER: ColorfulLogger = ColorfulLogger {
    lock: Mutex::new(()),
};
