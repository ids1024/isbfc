/// Add line of assembly to output, with indentation and newline, using
/// format! syntax.
macro_rules! push_asm {
    ($state:expr, $fmt:expr) => {
        (writeln!(&mut $state.output, concat!("{}", $fmt),
               " ".repeat($state.level * 4))).unwrap()
    };
    ($state:expr, $fmt:expr, $($arg:tt)*) => {
        (writeln!(&mut $state.output, concat!("{}", $fmt),
               " ".repeat($state.level * 4),
               $($arg)*)).unwrap()
    };
}
