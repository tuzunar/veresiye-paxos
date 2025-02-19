#[derive(Debug, Clone, Default)]
pub struct Proposal {
    seq_id: usize,
    key: String,
    value: String,
}

impl Proposal {
    pub fn new(seq_id: usize, key: String, value: String) -> Self {
        Self { seq_id, key, value }
    }

    pub fn get_seq_id(&self) -> usize {
        self.seq_id
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}
