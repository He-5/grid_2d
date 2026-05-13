#[macro_export]
macro_rules! some {
    {if $pred:expr => $value:expr} => {
        if $pred {
            Some($value)
        } else { None }
    };
}
