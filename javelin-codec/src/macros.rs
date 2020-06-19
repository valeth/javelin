macro_rules! ensure {
    ($check:expr, $err:expr) => {
        if !($check) { return Err($err) }
    };
}
