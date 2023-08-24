use std::{ffi::CStr, slice::from_raw_parts};

use rayon::prelude::*;

mod generated;

fn foo() {
    // let t = from_raw_parts::<u64>(data, len);
    // let p = t.par_iter().map(map_op).sum::<u64>();
}
