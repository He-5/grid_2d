#[macro_export]
macro_rules! some {
    {if $pred:expr => $value:expr} => {
        if $pred {
            Some($value)
        } else { None }
    };
}

#[macro_export]
macro_rules! short_cut {
    ($variant:ident($expr:expr)) => { short_cut!($variant($expr)?()) };
    ($variant:ident($expr:expr)?$default:expr) => {
        if let $variant(value) = $expr {
            value
        } else {
            return $default;
        }
    }
}