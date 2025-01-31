struct Proposal {
    seq_id: usize,
    key: String,
    value: String,
}

impl Proposal {
    fn new(seq_id: usize, key: String, value: String) -> Self {
        Self { seq_id, key, value }
    }
}
