use serde::de;
use std::collections::HashMap;
use tokio::sync::broadcast::{
    channel as BroadcastChannel,
    error::{RecvError, SendError},
    Receiver as BroadcastReceiver, Sender as BroadcastSender,
};

#[derive(Clone)]
pub enum ManagerAction {
    Stop,
}

#[derive(Clone)]
pub enum ManagerMessage<T: de::DeserializeOwned + Clone> {
    Message(T),
    Action(ManagerAction),
}

pub type MangerSender<T> = BroadcastSender<ManagerMessage<T>>;
pub type MangerReceiver<T> = BroadcastReceiver<ManagerMessage<T>>;

pub struct Manager<T: de::DeserializeOwned + Clone> {
    tx: MangerSender<T>,
    rx: MangerReceiver<T>,
}
impl<T: de::DeserializeOwned + Clone> Manager<T> {
    pub fn new() -> Self {
        let (tx, rx) = BroadcastChannel::<ManagerMessage<T>>(1000);
        Manager { tx, rx }
    }

    pub fn subscribe(&self) -> Self {
        let rx = self.tx.subscribe();
        Manager {
            tx: self.tx.clone(),
            rx,
        }
    }

    pub async fn read(&mut self) -> Result<ManagerMessage<T>, RecvError> {
        self.rx.recv().await
    }

    pub async fn send(&mut self, message: T) -> Result<usize, SendError<ManagerMessage<T>>> {
        self.tx.send(ManagerMessage::Message(message))
    }

    pub async fn stop(self) -> Result<usize, SendError<ManagerMessage<T>>> {
        self.tx.send(ManagerMessage::Action(ManagerAction::Stop))
    }
}
