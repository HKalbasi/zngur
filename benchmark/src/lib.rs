mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub fn build_vec_by_push_rust(n: u64) -> Vec<u64> {
    let mut v = Vec::new();
    for i in 0..n {
        v.push(i);
    }
    assert_eq!(5, v[5]);
    v
}

pub fn build_vec_by_push_cpp(n: u64) -> Vec<u64> {
    let v = generated::build_vec_by_push_cpp(n);
    assert_eq!(5, v[5]);
    v
}
