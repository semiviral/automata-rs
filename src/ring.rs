pub struct RingIndex {
    max: usize,
    current: usize,
}

impl RingIndex {
    pub fn new(max: usize) -> Self {
        Self { max, current: 0 }
    }

    pub fn index(&self) -> usize {
        self.current
    }

    pub fn increment(&mut self) {
        self.current = self.next_index();
    }

    pub fn next_index(&self) -> usize {
        (self.current + 1) % self.max
    }
}
