use crate::models::Message;
use std::collections::HashMap;
use tokio::sync::broadcast::{
    channel as BroadcastChannel, Receiver as BroadcastReceiver, Sender as BroadcastSender,
};

#[derive(Clone)]
pub enum ManagerAction {
    Stop,
}

#[derive(Clone)]
pub enum ManagerMessage {
    Message(Message),
    Action(ManagerAction),
}

pub struct Manager {
    broadcast: (
        BroadcastSender<ManagerMessage>,
        BroadcastReceiver<ManagerMessage>,
    ),
}
impl Manager {
    pub fn new() -> Self {
        Manager {
            broadcast: BroadcastChannel(1000),
        }
    }
}
