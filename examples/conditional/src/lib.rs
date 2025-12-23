/// only in place to allow both build to build sid by side in CI
#[rustfmt::skip]
#[cfg(feature = "float-values")]
mod generated_float;

#[rustfmt::skip]
#[cfg(not(feature = "float-values"))]
mod generated_int;

#[cfg_attr(not(debug_assertions), derive(Debug))]
struct KeyValuePair {
    pub key: String,
    #[cfg(feature = "float-values")]
    pub value: f64,
    #[cfg(not(feature = "float-values"))]
    pub value: i32,
}

impl KeyValuePair {
    fn self_size() -> usize {
        std::mem::size_of::<Self>()
    }
    fn self_align() -> usize {
        std::mem::align_of::<Self>()
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for KeyValuePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KeyValuePair {{ key: {:?}, value: {:?}, }}(with debug_assertions)",
            &self.key, &self.value
        )
    }
}
