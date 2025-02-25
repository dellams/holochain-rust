use crate::{action::ActionWrapper, consistency::ConsistencySignal};
use crossbeam_channel::{unbounded, Receiver, Sender};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_wasm_utils::api_serialization::emit_signal::EmitSignalArgs;
use serde::{Deserialize, Deserializer};
use std::thread;

#[derive(Clone, Debug, Serialize, DefaultJson)]
#[serde(tag = "signal_type")]
pub enum Signal {
    Trace(ActionWrapper),
    Consistency(ConsistencySignal),
    User(UserSignal),
}

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson, PartialEq)]
pub struct UserSignal {
    pub name: String,
    pub arguments: JsonString,
}

impl From<EmitSignalArgs> for UserSignal {
    fn from(args: EmitSignalArgs) -> UserSignal {
        UserSignal {
            name: args.name,
            arguments: args.arguments,
        }
    }
}

impl<'de> Deserialize<'de> for Signal {
    fn deserialize<D>(_deserializer: D) -> Result<Signal, D::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}

pub type SignalSender = Sender<Signal>;
pub type SignalReceiver = Receiver<Signal>;

pub fn signal_channel() -> (SignalSender, SignalReceiver) {
    unbounded()
}

/// Pass on messages from multiple receivers into a single receiver
/// A potentially useful utility, but currently unused.
pub fn _combine_receivers<T>(rxs: Vec<Receiver<T>>) -> Receiver<T>
where
    T: 'static + Send,
{
    let (master_tx, master_rx) = unbounded::<T>();
    for rx in rxs {
        let tx = master_tx.clone();
        thread::spawn(move || {
            while let Ok(item) = rx.recv() {
                tx.send(item).unwrap_or(());
            }
        });
    }
    master_rx
}
