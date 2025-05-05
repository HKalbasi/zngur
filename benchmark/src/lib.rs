mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub fn build_vec_by_push_rust(n: u64) -> Vec<u64> {
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i);
    }
    v
}

pub fn build_vec_by_push_cpp(n: u64) -> Vec<u64> {
    generated::build_vec_by_push_cpp(n)
}
