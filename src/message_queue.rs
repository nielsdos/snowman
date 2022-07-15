use std::sync::mpsc::{channel, Sender, Receiver};
use crate::handle_table::Handle;
use crate::two_d::Point;

pub struct WindowMessage {
    pub h_wnd: Handle,
    pub message: u16,
    pub w_param: u16,
    pub l_param: u32,
    pub time: u32,
    pub point: Point,
}

pub struct MessageQueue {
    tx: Sender<WindowMessage>,
    rx: Receiver<WindowMessage>,
}

impl MessageQueue {
    pub fn new() -> Self {
        let (tx, rx) = channel::<WindowMessage>();
        Self { tx, rx }
    }

    pub fn send(&self, msg: WindowMessage) -> bool {
        self.tx.send(msg).is_ok()
    }

    pub fn receive(&self) -> Option<WindowMessage> {
        self.rx.recv().ok()
    }
}


