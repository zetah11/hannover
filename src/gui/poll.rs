use single_value_channel::Receiver;

#[derive(Debug)]
pub struct InputPoller {
    prev: String,
    recv: Receiver<String>,
}

impl InputPoller {
    pub fn new(recv: Receiver<String>) -> Self {
        Self {
            prev: String::new(),
            recv,
        }
    }

    /// Poll the GUI for new input. Returns `None` if it hasn't changed since
    /// last poll.
    pub fn poll(&mut self) -> Option<&str> {
        let curr = self.recv.latest();
        if curr != &self.prev {
            self.prev = curr.clone();
            Some(&self.prev)
        } else {
            None
        }
    }
}

pub struct WavetablePoller {
    prev: Vec<u8>,
    recv: Receiver<Vec<u8>>,
}

impl WavetablePoller {
    pub fn new(recv: Receiver<Vec<u8>>) -> Self {
        Self { prev: vec![], recv }
    }

    /// Poll the audio processor for the current wave. Returns `None` if it
    /// hasn't changed since last poll.
    pub fn poll(&mut self) -> Option<&[u8]> {
        let curr = self.recv.latest();
        if curr != &self.prev {
            self.prev = curr.clone();
            Some(&self.prev)
        } else {
            None
        }
    }
}
