use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_channel::{Receiver, Sender, unbounded};

use lune_utils::TableBuilder;
use mlua::prelude::*;

#[derive(Debug, Clone)]
pub struct ServeHandle {
    addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
    sender: Sender<()>,
}

impl ServeHandle {
    pub fn new(addr: SocketAddr) -> (Self, Receiver<()>) {
        let (sender, receiver) = unbounded();
        let this = Self {
            addr,
            shutdown: Arc::new(AtomicBool::new(false)),
            sender,
        };
        (this, receiver)
    }

    /**
        Creates the handle a script gets from `net.serve`.

        `stop` is bound to this handle and ignores its arguments, so both
        `handle:stop()`, as Lune 0.11 calls it, and `handle.stop()`, as
        earlier releases did, stop the server.
    */
    pub fn into_lua_table(self, lua: Lua) -> LuaResult<LuaTable> {
        let shutdown = self.shutdown.clone();
        let sender = self.sender.clone();
        TableBuilder::new(lua)?
            .with_value("ip", self.addr.ip().to_string())?
            .with_value("port", self.addr.port())?
            .with_function("stop", move |_, _: LuaMultiValue| {
                if shutdown.load(Ordering::SeqCst) {
                    Err(LuaError::runtime("Server already stopped"))
                } else {
                    shutdown.store(true, Ordering::SeqCst);
                    sender.try_send(()).ok();
                    sender.close();
                    Ok(())
                }
            })?
            .build()
    }
}
