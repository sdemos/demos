//! macros contains all the rust macros in one place, so that they can be
//! imported before everything and used across the project.

/// println is essentially a copy of the println macro normally provided by the
/// rust standard library, except this one uses our own print! macro, which
/// prints to the vga buffer instead of standard out.
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// print prints a formatted string to the vga buffer
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga::print(format_args!($($arg)*));
    })
}
