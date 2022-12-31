/// A fixed size queue. When pushing a new item, the oldest one gets returned.
#[derive(Debug)]
pub struct FixedQueue<T> {
    data: Vec<T>,
    at: usize,
}

impl<T> FixedQueue<T> {
    pub fn new_with(f: impl FnMut() -> T, capacity: usize) -> Self {
        let mut data = Vec::with_capacity(capacity);
        data.extend(std::iter::repeat_with(f).take(capacity));

        Self { data, at: 0 }
    }

    /// Push a value to this queue, returning the oldest one.
    pub fn push(&mut self, item: T) -> T {
        let value = std::mem::replace(&mut self.data[self.at], item);
        self.at = self.at.wrapping_add(1) % self.data.len();
        value
    }

    pub fn get(&self) -> &T {
        self.data.get(self.at).unwrap()
    }
}

impl<T: Default> FixedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self::new_with(T::default, capacity)
    }
}
