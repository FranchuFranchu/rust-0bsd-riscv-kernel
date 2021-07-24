use log::{Record, Level, Metadata};



pub struct ColorfulLogger;

impl log::Log for ColorfulLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
        	let prefix = match record.level() {
			    Level::Error => {
			    	"\x1b[91;1mERROR\x1b[0m"
			    }
			    Level::Warn => {
			    	"\x1b[93;1mWARN \x1b[0m"
			    }
			    Level::Info => {
			    	"\x1b[1mINFO \x1b[0m"
			    }
			    Level::Debug => {
			    	"\x1b[1;96mDEBUG\x1b[0m"
			    }
			    Level::Trace => {
			    	"\x1b[96mTRACE\x1b[0m"
			    }
        	};
        	
            println!("{} [{}] {}", record.module_path().unwrap_or(""), prefix, record.args());
        }
    }

    fn flush(&self) {}
}

pub static KERNEL_LOGGER: ColorfulLogger = ColorfulLogger;