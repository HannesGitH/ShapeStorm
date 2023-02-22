#[macro_export]
macro_rules! x4 {
    ($x:expr) => {
        [$x, $x, $x, $x]
    };
}

#[macro_export]
macro_rules! x3 {
    ($x:expr) => {
        [$x, $x, $x]
    };
}