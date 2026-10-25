//! Watcher `ext-data-control-v1` / `wlr-data-control-unstable-v1` sur thread dédié.

use std::io::{Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self as std_mpsc, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nix::poll::{poll, PollFd, PollFlags, PollTimeout};
use shared_types::RunaError;
use shared_utils::should_emit_impulse;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use wayland_client::protocol::{wl_registry, wl_seat::WlSeat};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::{self, ExtDataControlDeviceV1},
    ext_data_control_manager_v1::ExtDataControlManagerV1,
    ext_data_control_offer_v1::{self, ExtDataControlOfferV1},
    ext_data_control_source_v1::{self, ExtDataControlSourceV1},
};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1::{self, ZwlrDataControlDeviceV1},
    zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    zwlr_data_control_offer_v1::{self, ZwlrDataControlOfferV1},
    zwlr_data_control_source_v1::{self, ZwlrDataControlSourceV1},
};

use crate::clipboard::ClipboardImpulse;
use crate::linux_select::ClipboardBackend;

const COALESCE_MS: u64 = 40;
const SUPPRESS_WRITE_MS: u64 = 100;
const PREFERRED_MIME: &[&str] = &["text/plain;charset=utf-8", "text/plain", "TEXT", "STRING"];

pub enum ClipCmd {
    Read(Sender<Result<String, RunaError>>),
    Write {
        text: String,
        reply: Sender<Result<(), RunaError>>,
    },
    Stop,
}

enum Manager {
    Ext(ExtDataControlManagerV1),
    Wlr(ZwlrDataControlManagerV1),
}

enum Device {
    Ext(ExtDataControlDeviceV1),
    Wlr(ZwlrDataControlDeviceV1),
}

enum Offer {
    Ext(ExtDataControlOfferV1),
    Wlr(ZwlrDataControlOfferV1),
}

struct OfferInfo {
    offer: Offer,
    mimes: Vec<String>,
}

struct State {
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    suppress_until_ms: Arc<AtomicU64>,
    has_emitted: bool,
    last_at: Instant,
    seat: Option<WlSeat>,
    manager: Option<Manager>,
    device: Option<Device>,
    current: Option<OfferInfo>,
    pending: Option<OfferInfo>,
    serving: Option<Vec<u8>>,
    stop: bool,
    prefer_ext: bool,
}

impl State {
    fn emit_impulse(&mut self) {
        let epoch = now_epoch_ms();
        if epoch < self.suppress_until_ms.load(Ordering::SeqCst) {
            debug!("wayland selection supprimée (écriture interne)");
            return;
        }
        let now = Instant::now();
        if !should_emit_impulse(self.has_emitted, self.last_at, now, COALESCE_MS) {
            debug!("wayland selection coalescée");
            return;
        }
        self.has_emitted = true;
        self.last_at = now;
        let impulse = ClipboardImpulse {
            sequence_hint: None,
            immediate: false,
        };
        if let Err(err) = self.impulse_tx.try_send(impulse) {
            debug!(?err, "impulsion wayland non envoyée");
        } else {
            debug!("data-control selection → impulsion");
        }
    }

    fn read_current(&self, conn: &Connection, event_queue: &mut EventQueue<Self>) -> Result<String, RunaError> {
        let info = self.current.as_ref().ok_or_else(|| RunaError::Clipboard {
            message: "aucune offre presse-papiers Wayland".into(),
        })?;
        let mime = pick_mime(&info.mimes).ok_or_else(|| RunaError::Clipboard {
            message: "pas de mime texte sur l'offre Wayland".into(),
        })?;
        let (read_fd, write_fd) = pipe_pair()?;
        match &info.offer {
            Offer::Ext(o) => o.receive(mime.to_string(), write_fd.as_fd()),
            Offer::Wlr(o) => o.receive(mime.to_string(), write_fd.as_fd()),
        }
        drop(write_fd);
        let _ = event_queue.flush();
        // Laisser le compositor écrire dans le pipe.
        let _ = conn.flush();
        std::thread::sleep(Duration::from_millis(5));
        let mut file = std::fs::File::from(read_fd);
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| RunaError::Clipboard {
            message: format!("lecture pipe Wayland: {e}"),
        })?;
        String::from_utf8(buf).map_err(|e| RunaError::Clipboard {
            message: format!("utf-8 presse-papiers: {e}"),
        })
    }

    fn set_selection_text(&mut self, text: String, qh: &QueueHandle<Self>) -> Result<(), RunaError> {
        let mgr = self.manager.as_ref().ok_or_else(|| RunaError::Clipboard {
            message: "manager data-control absent".into(),
        })?;
        let device = self.device.as_ref().ok_or_else(|| RunaError::Clipboard {
            message: "device data-control absent".into(),
        })?;
        match mgr {
            Manager::Ext(m) => {
                let source = m.create_data_source(qh, ());
                for mime in PREFERRED_MIME {
                    source.offer((*mime).to_string());
                }
                if let Device::Ext(d) = device {
                    d.set_selection(Some(&source));
                }
            }
            Manager::Wlr(m) => {
                let source = m.create_data_source(qh, ());
                for mime in PREFERRED_MIME {
                    source.offer((*mime).to_string());
                }
                if let Device::Wlr(d) = device {
                    d.set_selection(Some(&source));
                }
            }
        }
        self.serving = Some(text.into_bytes());
        let until = now_epoch_ms().saturating_add(SUPPRESS_WRITE_MS);
        self.suppress_until_ms.store(until, Ordering::SeqCst);
        Ok(())
    }
}

fn pick_mime(mimes: &[String]) -> Option<&str> {
    for pref in PREFERRED_MIME {
        if mimes.iter().any(|m| m == *pref) {
            return Some(*pref);
        }
    }
    mimes.first().map(String::as_str)
}

fn pipe_pair() -> Result<(OwnedFd, OwnedFd), RunaError> {
    nix::unistd::pipe().map_err(|e| RunaError::Clipboard {
        message: format!("pipe: {e}"),
    })
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn spawn_watcher(
    backend: ClipboardBackend,
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    suppress_until_ms: Arc<AtomicU64>,
) -> Result<(Sender<ClipCmd>, std::thread::JoinHandle<()>), RunaError> {
    let prefer_ext = matches!(backend, ClipboardBackend::ExtDataControl);
    let (cmd_tx, cmd_rx) = std_mpsc::channel::<ClipCmd>();
    let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), RunaError>>();

    let join = std::thread::Builder::new()
        .name("lin-clipboard".into())
        .spawn(move || {
            if let Err(err) = thread_main(prefer_ext, impulse_tx, suppress_until_ms, cmd_rx, &ready_tx)
            {
                let _ = ready_tx.send(Err(err));
            }
        })
        .map_err(|e| RunaError::Clipboard {
            message: format!("spawn lin-clipboard: {e}"),
        })?;

    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| RunaError::Clipboard {
            message: "timeout démarrage lin-clipboard".into(),
        })??;

    Ok((cmd_tx, join))
}

fn thread_main(
    prefer_ext: bool,
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    suppress_until_ms: Arc<AtomicU64>,
    cmd_rx: Receiver<ClipCmd>,
    ready_tx: &Sender<Result<(), RunaError>>,
) -> Result<(), RunaError> {
    let conn = Connection::connect_to_env().map_err(|e| RunaError::Clipboard {
        message: format!("Wayland connect: {e}"),
    })?;
    let mut event_queue: EventQueue<State> = conn.new_event_queue();
    let qh = event_queue.handle();
    let display = conn.display();
    let _registry = display.get_registry(&qh, ());

    let mut state = State {
        impulse_tx,
        suppress_until_ms,
        has_emitted: false,
        last_at: Instant::now(),
        seat: None,
        manager: None,
        device: None,
        current: None,
        pending: None,
        serving: None,
        stop: false,
        prefer_ext,
    };

    event_queue
        .roundtrip(&mut state)
        .map_err(|e| RunaError::Clipboard {
            message: format!("Wayland roundtrip registry: {e}"),
        })?;

    let seat = state.seat.clone().ok_or_else(|| RunaError::Clipboard {
        message: "wl_seat absent".into(),
    })?;
    let mgr = state.manager.take().ok_or_else(|| RunaError::Clipboard {
        message: if prefer_ext {
            "ext_data_control_manager_v1 absent".into()
        } else {
            "zwlr_data_control_manager_v1 absent".into()
        },
    })?;

    let device = match &mgr {
        Manager::Ext(m) => Device::Ext(m.get_data_device(&seat, &qh, ())),
        Manager::Wlr(m) => Device::Wlr(m.get_data_device(&seat, &qh, ())),
    };
    state.manager = Some(mgr);
    state.device = Some(device);

    event_queue
        .roundtrip(&mut state)
        .map_err(|e| RunaError::Clipboard {
            message: format!("Wayland roundtrip device: {e}"),
        })?;

    let _ = ready_tx.send(Ok(()));

    while !state.stop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                ClipCmd::Stop => {
                    state.stop = true;
                    break;
                }
                ClipCmd::Read(reply) => {
                    let _ = event_queue.roundtrip(&mut state);
                    let result = state.read_current(&conn, &mut event_queue);
                    let _ = reply.send(result);
                }
                ClipCmd::Write { text, reply } => {
                    let result = state.set_selection_text(text, &qh);
                    let _ = event_queue.flush();
                    let _ = reply.send(result);
                }
            }
        }
        if state.stop {
            break;
        }

        let _ = event_queue.flush();
        let Some(read_guard) = conn.prepare_read() else {
            let _ = event_queue.dispatch_pending(&mut state);
            continue;
        };

        let mut fds = [PollFd::new(conn.as_fd(), PollFlags::POLLIN)];
        match poll(&mut fds, PollTimeout::from(50u16)) {
            Ok(0) => {
                drop(read_guard);
            }
            Ok(_) => {
                if let Err(err) = read_guard.read() {
                    debug!(?err, "Wayland read");
                }
                let _ = event_queue.dispatch_pending(&mut state);
            }
            Err(err) => {
                drop(read_guard);
                warn!(?err, "poll Wayland");
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }

    Ok(())
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "wl_seat" && state.seat.is_none() {
                let seat: WlSeat = registry.bind(name, version.min(8), qh, ());
                state.seat = Some(seat);
            }
            if state.manager.is_none() {
                if state.prefer_ext && interface == "ext_data_control_manager_v1" {
                    let mgr: ExtDataControlManagerV1 =
                        registry.bind(name, version.min(1), qh, ());
                    state.manager = Some(Manager::Ext(mgr));
                } else if !state.prefer_ext && interface == "zwlr_data_control_manager_v1" {
                    let mgr: ZwlrDataControlManagerV1 =
                        registry.bind(name, version.min(2), qh, ());
                    state.manager = Some(Manager::Wlr(mgr));
                }
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: wayland_client::protocol::wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtDataControlManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ExtDataControlManagerV1,
        _: wayland_protocols::ext::data_control::v1::client::ext_data_control_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrDataControlManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwlrDataControlManagerV1,
        _: wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtDataControlDeviceV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ExtDataControlDeviceV1,
        event: ext_data_control_device_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_data_control_device_v1::Event::DataOffer { id } => {
                state.pending = Some(OfferInfo {
                    offer: Offer::Ext(id),
                    mimes: Vec::new(),
                });
            }
            ext_data_control_device_v1::Event::Selection { id } => {
                if let Some(offer) = id {
                    if let Some(mut pending) = state.pending.take() {
                        pending.offer = Offer::Ext(offer);
                        state.current = Some(pending);
                    } else {
                        state.current = Some(OfferInfo {
                            offer: Offer::Ext(offer),
                            mimes: Vec::new(),
                        });
                    }
                    state.emit_impulse();
                } else {
                    state.current = None;
                }
            }
            ext_data_control_device_v1::Event::Finished => {}
            _ => {}
        }
    }

    wayland_client::event_created_child!(State, ExtDataControlDeviceV1, [
        ext_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ExtDataControlOfferV1, ())
    ]);
}

impl Dispatch<ZwlrDataControlDeviceV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwlrDataControlDeviceV1,
        event: zwlr_data_control_device_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_device_v1::Event::DataOffer { id } => {
                state.pending = Some(OfferInfo {
                    offer: Offer::Wlr(id),
                    mimes: Vec::new(),
                });
            }
            zwlr_data_control_device_v1::Event::Selection { id } => {
                if let Some(offer) = id {
                    if let Some(mut pending) = state.pending.take() {
                        pending.offer = Offer::Wlr(offer);
                        state.current = Some(pending);
                    } else {
                        state.current = Some(OfferInfo {
                            offer: Offer::Wlr(offer),
                            mimes: Vec::new(),
                        });
                    }
                    state.emit_impulse();
                } else {
                    state.current = None;
                }
            }
            zwlr_data_control_device_v1::Event::Finished => {}
            _ => {}
        }
    }

    wayland_client::event_created_child!(State, ZwlrDataControlDeviceV1, [
        zwlr_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ZwlrDataControlOfferV1, ())
    ]);
}

impl Dispatch<ExtDataControlOfferV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ExtDataControlOfferV1,
        event: ext_data_control_offer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_data_control_offer_v1::Event::Offer { mime_type } = event {
            if let Some(pending) = state.pending.as_mut() {
                pending.mimes.push(mime_type);
            }
        }
    }
}

impl Dispatch<ZwlrDataControlOfferV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwlrDataControlOfferV1,
        event: zwlr_data_control_offer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_data_control_offer_v1::Event::Offer { mime_type } = event {
            if let Some(pending) = state.pending.as_mut() {
                pending.mimes.push(mime_type);
            }
        }
    }
}

impl Dispatch<ExtDataControlSourceV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ExtDataControlSourceV1,
        event: ext_data_control_source_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_data_control_source_v1::Event::Send { mime_type: _, fd } => {
                if let Some(data) = state.serving.as_ref() {
                    let mut file = std::fs::File::from(fd);
                    let _ = file.write_all(data);
                }
            }
            ext_data_control_source_v1::Event::Cancelled => {
                state.serving = None;
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrDataControlSourceV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ZwlrDataControlSourceV1,
        event: zwlr_data_control_source_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_source_v1::Event::Send { mime_type: _, fd } => {
                if let Some(data) = state.serving.as_ref() {
                    let mut file = std::fs::File::from(fd);
                    let _ = file.write_all(data);
                }
            }
            zwlr_data_control_source_v1::Event::Cancelled => {
                state.serving = None;
            }
            _ => {}
        }
    }
}

pub fn stop_watcher(cmd_tx: Sender<ClipCmd>, join: std::thread::JoinHandle<()>) {
    let _ = cmd_tx.send(ClipCmd::Stop);
    if let Err(err) = join.join() {
        warn!(?err, "join lin-clipboard échoué");
    }
}

pub fn cmd_read(cmd_tx: &Sender<ClipCmd>) -> Result<String, RunaError> {
    let (tx, rx) = std_mpsc::channel();
    cmd_tx
        .send(ClipCmd::Read(tx))
        .map_err(|_| RunaError::Clipboard {
            message: "thread clipboard arrêté".into(),
        })?;
    rx.recv_timeout(Duration::from_secs(2))
        .map_err(|_| RunaError::Clipboard {
            message: "timeout lecture Wayland".into(),
        })?
}

pub fn cmd_write(cmd_tx: &Sender<ClipCmd>, text: String) -> Result<(), RunaError> {
    let (tx, rx) = std_mpsc::channel();
    cmd_tx
        .send(ClipCmd::Write { text, reply: tx })
        .map_err(|_| RunaError::Clipboard {
            message: "thread clipboard arrêté".into(),
        })?;
    rx.recv_timeout(Duration::from_secs(2))
        .map_err(|_| RunaError::Clipboard {
            message: "timeout écriture Wayland".into(),
        })?
}
