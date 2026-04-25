use std::sync::Arc;
use std::thread;
use std::time::Instant;

use async_trait::async_trait;
use opensynapse_core::{
    AdapterError, EventStream, ForegroundEvent, ForegroundWatcher, Result as CoreResult,
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{debug, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, GetPropertyReply,
    PropertyNotifyEvent,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

const CHANNEL_CAPACITY: usize = 32;

pub struct X11ForegroundWatcher {
    sender: broadcast::Sender<ForegroundEvent>,
    _worker: Arc<thread::JoinHandle<()>>,
}

impl X11ForegroundWatcher {
    pub fn start() -> CoreResult<Self> {
        let (conn, screen_num) =
            RustConnection::connect(None).map_err(|e| AdapterError::fatal(Box::new(e)))?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let atoms = Atoms::intern(&conn).map_err(|e| AdapterError::fatal(Box::new(e)))?;

        conn.change_window_attributes(
            root,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )
        .map_err(|e| AdapterError::fatal(Box::new(e)))?
        .check()
        .map_err(|e| AdapterError::fatal(Box::new(e)))?;

        let (tx, _rx) = broadcast::channel(CHANNEL_CAPACITY);
        let tx_worker = tx.clone();

        let worker = thread::Builder::new()
            .name("fg-watcher-x11".into())
            .spawn(move || run_event_loop(conn, root, atoms, tx_worker))
            .map_err(|e| AdapterError::fatal(Box::new(e)))?;

        Ok(Self {
            sender: tx,
            _worker: Arc::new(worker),
        })
    }
}

#[async_trait]
impl ForegroundWatcher for X11ForegroundWatcher {
    async fn watch(&self) -> CoreResult<EventStream<ForegroundEvent>> {
        let rx = self.sender.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|r| r.ok());
        Ok(Box::pin(stream))
    }
}

#[derive(Clone, Copy)]
struct Atoms {
    net_active_window: Atom,
    net_wm_name: Atom,
    utf8_string: Atom,
}

impl Atoms {
    fn intern(conn: &RustConnection) -> Result<Self, x11rb::errors::ReplyError> {
        let net_active_window = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
            .reply()?
            .atom;
        let net_wm_name = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;
        let utf8_string = conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;
        Ok(Self {
            net_active_window,
            net_wm_name,
            utf8_string,
        })
    }
}

fn run_event_loop(
    conn: RustConnection,
    root: u32,
    atoms: Atoms,
    tx: broadcast::Sender<ForegroundEvent>,
) {
    // Emitir el estado inicial.
    if let Some(ev) = read_active_window(&conn, root, &atoms) {
        let _ = tx.send(ev);
    }

    loop {
        match conn.wait_for_event() {
            Ok(Event::PropertyNotify(PropertyNotifyEvent { atom, window, .. }))
                if window == root && atom == atoms.net_active_window =>
            {
                if let Some(ev) = read_active_window(&conn, root, &atoms) {
                    if tx.send(ev).is_err() {
                        debug!("no subscribers; continuing");
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                warn!(error = %e, "x11 connection error; exiting watcher");
                return;
            }
        }
    }
}

fn read_active_window(conn: &RustConnection, root: u32, atoms: &Atoms) -> Option<ForegroundEvent> {
    let active = get_property_window(conn, root, atoms.net_active_window)?;
    if active == 0 || active == x11rb::NONE {
        return None;
    }
    let wm_class = get_wm_class(conn, active).unwrap_or_default();
    let title = get_string(conn, active, atoms.net_wm_name, atoms.utf8_string)
        .or_else(|| {
            get_string(
                conn,
                active,
                AtomEnum::WM_NAME.into(),
                AtomEnum::STRING.into(),
            )
        })
        .unwrap_or_default();
    Some(ForegroundEvent {
        wm_class,
        title,
        timestamp: Instant::now(),
    })
}

fn get_property_window(conn: &RustConnection, win: u32, atom: Atom) -> Option<u32> {
    let reply = fetch_property(conn, win, atom, AtomEnum::WINDOW.into())?;
    reply.value32().and_then(|mut it| it.next())
}

fn get_wm_class(conn: &RustConnection, win: u32) -> Option<String> {
    let reply = fetch_property(
        conn,
        win,
        AtomEnum::WM_CLASS.into(),
        AtomEnum::STRING.into(),
    )?;
    // WM_CLASS = "instance\0class\0". El segundo es el "class" canónico.
    let bytes = reply.value;
    let mut parts = bytes.split(|b| *b == 0).filter(|s| !s.is_empty());
    let _instance = parts.next();
    let class = parts.next()?;
    Some(String::from_utf8_lossy(class).into_owned())
}

fn get_string(conn: &RustConnection, win: u32, atom: Atom, kind: Atom) -> Option<String> {
    let reply = fetch_property(conn, win, atom, kind)?;
    if reply.value.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&reply.value).into_owned())
    }
}

fn fetch_property(
    conn: &RustConnection,
    win: u32,
    property: Atom,
    type_: Atom,
) -> Option<GetPropertyReply> {
    conn.get_property(false, win, property, type_, 0, u32::MAX / 4)
        .ok()?
        .reply()
        .ok()
}
