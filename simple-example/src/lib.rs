mod generated;

pub struct St {
    pub field1: i32,
    pub field2: Vec<i32>,
}

fn collect_vec(iter: Box<dyn Iterator<Item = i32>>) -> Vec<i32> {
    iter.collect()
}
