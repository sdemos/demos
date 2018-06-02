//! klog implements a global kernel logger.

use log::{self, Record, Metadata, LevelFilter};

static KLOGGER: KernelLogger = KernelLogger;

struct KernelLogger;

/// init initializes global logging functions. the set_logger function returns
/// an error if the logger is already set, but since we are the only ones doing
/// anything and we preface the function with an assert that claims it's only
/// called once, we just expect it.
///
/// it isn't actually an error to log before this call - the function the
/// logging macros call to get the global logger just returns a no-op logging
/// implementation. still, it's not great if log messages go down the drain, so
/// call this early.
pub fn init() {
    assert_has_not_been_called!("log::init must only be called once");

    log::set_logger(&KLOGGER).expect("failed to set global logger");
    log::set_max_level(LevelFilter::Trace);
}

impl log::Log for KernelLogger {
    /// enabled returns whether or not the logger is supposed to log for the
    /// reported level. I'm choosing to control this at compile time instead of
    /// runtime, so just always say yes.
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    /// this is the function called by the logging macros provided by the log
    /// crate. it's the thing that actually does the logging work.
    fn log(&self, record: &Record) {
        let file = record.file().unwrap_or("???");
        let line = record.line().unwrap_or(0);
        println!("[{}:{}] ({}:{}):\n    {}",
                 record.level(),
                 record.target(),
                 file,
                 line,
                 record.args(),
        );
    }

    /// meant to flush buffered records. the current implementation of vga
    /// doesn't do any buffering so no operation is performed.
    fn flush(&self) {}
}
