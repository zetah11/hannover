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
        let curr = self.recv.latest().clone();
        if curr != self.prev {
            self.prev = curr;
            Some(&self.prev)
        } else {
            None
        }
    }
}
