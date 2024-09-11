pub mod spec;

#[macro_export]
macro_rules! include_json {
    ($path:literal, $ptr:literal) => {{
        let value: ::serde_json::Value = serde_json::from_str(include_str!($path)).unwrap();
        value.pointer($ptr).unwrap().clone()
    }};
    ($path:literal) => {{
        let value: ::serde_json::Value = serde_json::from_str(include_str!($path)).unwrap();
        value
    }};
}
