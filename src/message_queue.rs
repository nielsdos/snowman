use crate::constants::MessageType;
use crate::handle_table::Handle;
use crate::two_d::Point;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct WindowMessage {
    pub h_wnd: Handle,
    pub message: MessageType,
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

    pub fn receive(&self, h_wnd: Handle) -> Option<WindowMessage> {
        loop {
            return if let Ok(message) = self.rx.recv() {
                // h_wnd acts as a filter, if the handle does not match, the message must be discarded
                // Null handle means that no filtering should take place.
                // Note: thread messages did not seem to exist back then...
                if h_wnd != Handle::null() && message.h_wnd != h_wnd {
                    continue;
                }

                Some(message)
            } else {
                None
            }
        }
    }
}
