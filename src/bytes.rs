/// An infinite stream of `N` nibbles (where two nibbles make up a byte). The
/// stream cycles through the provided data, and produces all zeroes if that is
/// empty.
#[derive(Clone, Debug)]
pub struct NibbleStream<const N: usize> {
    data: Vec<u8>,
    total: u8,
    index: usize,
    wrap: usize,
    _phantom: std::marker::PhantomData<[(); N]>,
}

impl<const N: usize> NibbleStream<N> {
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            total: 0,
            index: 0,
            wrap: 2 * data.len(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_new_data(&self, data: &[u8]) -> Self {
        let wrap = 2 * data.len();
        Self {
            data: data.to_vec(),
            total: 0,
            index: self.index % wrap,
            wrap,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the next `N` nibbles from this stream.
    pub fn next_nibbles(&mut self) -> [u8; N] {
        if self.data.is_empty() {
            return [0; N];
        }

        let mut data = [0; N];
        for (i, data) in data.iter_mut().enumerate() {
            let index = self.index + i;

            // note: self.wrap is always even, so this should be correct
            let byte = self.data[(index % self.wrap) / 2] ^ self.total;
            *data = if index % 2 == 0 {
                self.total = self.total.rotate_left(5).wrapping_add(byte);
                byte >> 4
            } else {
                byte & 0x0f
            };
        }

        self.index = (self.index + N) % self.wrap;

        data
    }
}

impl NibbleStream<1> {
    pub fn next_nibble(&mut self) -> u8 {
        self.next_nibbles()[0]
    }
}

impl<const N: usize> Iterator for NibbleStream<N> {
    type Item = [u8; N];

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_nibbles())
    }
}
