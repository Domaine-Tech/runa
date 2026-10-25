//! Fallback XFixes + lecture/écriture CLIPBOARD + focus / XTest.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self as std_mpsc, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use shared_types::{RunaError, WindowToken};
use shared_utils::should_emit_impulse;
use tokio::sync::mpsc;
use tracing::{debug, warn};
use x11rb::connection::Connection;
use x11rb::protocol::xfixes::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ClientMessageEvent, ConnectionExt as _, CreateWindowAux, EventMask, PropMode,
    SelectionNotifyEvent, SelectionRequestEvent, Window, CLIENT_MESSAGE_EVENT, KEY_PRESS_EVENT,
    KEY_RELEASE_EVENT, SELECTION_NOTIFY_EVENT,
};
use x11rb::protocol::xtest::ConnectionExt as _;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::clipboard::ClipboardImpulse;

const COALESCE_MS: u64 = 40;
const SUPPRESS_WRITE_MS: u64 = 100;
const PROP_NAME: &str = "RUNA_CLIPBOARD";

pub enum ClipCmd {
    Read(Sender<Result<String, RunaError>>),
    Write {
        text: String,
        reply: Sender<Result<(), RunaError>>,
    },
    Stop,
}

struct Atoms {
    clipboard: Atom,
    utf8: Atom,
    targets: Atom,
    property: Atom,
    incr: Atom,
}

fn intern(conn: &RustConnection, name: &str) -> Result<Atom, RunaError> {
    Ok(conn
        .intern_atom(false, name.as_bytes())
        .map_err(x_err)?
        .reply()
        .map_err(x_err)?
        .atom)
}

fn x_err(e: impl std::fmt::Display) -> RunaError {
    RunaError::Clipboard {
        message: format!("X11: {e}"),
    }
}

fn focus_err(e: impl std::fmt::Display) -> RunaError {
    RunaError::WindowFocus {
        message: format!("X11: {e}"),
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn spawn_watcher(
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    suppress_until_ms: Arc<AtomicU64>,
) -> Result<(Sender<ClipCmd>, std::thread::JoinHandle<()>), RunaError> {
    let (cmd_tx, cmd_rx) = std_mpsc::channel::<ClipCmd>();
    let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), RunaError>>();

    let join = std::thread::Builder::new()
        .name("lin-clipboard".into())
        .spawn(move || {
            if let Err(err) = thread_main(impulse_tx, suppress_until_ms, cmd_rx, &ready_tx) {
                let _ = ready_tx.send(Err(err));
            }
        })
        .map_err(|e| RunaError::Clipboard {
            message: format!("spawn x11 clipboard: {e}"),
        })?;

    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| RunaError::Clipboard {
            message: "timeout démarrage x11 clipboard".into(),
        })??;

    Ok((cmd_tx, join))
}

fn thread_main(
    impulse_tx: mpsc::Sender<ClipboardImpulse>,
    suppress_until_ms: Arc<AtomicU64>,
    cmd_rx: Receiver<ClipCmd>,
    ready_tx: &Sender<Result<(), RunaError>>,
) -> Result<(), RunaError> {
    let (conn, screen_num) = RustConnection::connect(None).map_err(x_err)?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let win = conn.generate_id().map_err(x_err)?;
    conn.create_window(
        x11rb::COPY_DEPTH_FROM_PARENT,
        win,
        root,
        0,
        0,
        1,
        1,
        0,
        x11rb::protocol::xproto::WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )
    .map_err(x_err)?;
    conn.flush().map_err(x_err)?;

    let atoms = Atoms {
        clipboard: intern(&conn, "CLIPBOARD")?,
        utf8: intern(&conn, "UTF8_STRING")?,
        targets: intern(&conn, "TARGETS")?,
        property: intern(&conn, PROP_NAME)?,
        incr: intern(&conn, "INCR")?,
    };

    xfixes::query_version(&conn, 5, 0)
        .map_err(x_err)?
        .reply()
        .map_err(x_err)?;
    conn.xfixes_select_selection_input(
        win,
        atoms.clipboard,
        xfixes::SelectionEventMask::SET_SELECTION_OWNER
            | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY
            | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE,
    )
    .map_err(x_err)?;
    conn.flush().map_err(x_err)?;

    let _ = ready_tx.send(Ok(()));

    let mut has_emitted = false;
    let mut last_at = Instant::now();
    let mut owned_text: Option<Vec<u8>> = None;

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                ClipCmd::Stop => return Ok(()),
                ClipCmd::Read(reply) => {
                    let result = read_clipboard(&conn, win, &atoms);
                    let _ = reply.send(result);
                }
                ClipCmd::Write { text, reply } => {
                    let until = now_epoch_ms().saturating_add(SUPPRESS_WRITE_MS);
                    suppress_until_ms.store(until, Ordering::SeqCst);
                    owned_text = Some(text.into_bytes());
                    let result = conn
                        .set_selection_owner(win, atoms.clipboard, x11rb::CURRENT_TIME)
                        .map_err(x_err)
                        .and_then(|_| conn.flush().map_err(x_err));
                    let _ = reply.send(result.map(|_| ()));
                }
            }
        }

        match conn.poll_for_event().map_err(x_err)? {
            Some(Event::XfixesSelectionNotify(_)) => {
                let epoch = now_epoch_ms();
                if epoch < suppress_until_ms.load(Ordering::SeqCst) {
                    debug!("XFixes selection supprimée (écriture interne)");
                } else {
                    let now = Instant::now();
                    if should_emit_impulse(has_emitted, last_at, now, COALESCE_MS) {
                        has_emitted = true;
                        last_at = now;
                        let impulse = ClipboardImpulse {
                            sequence_hint: None,
                            immediate: false,
                        };
                        if let Err(err) = impulse_tx.try_send(impulse) {
                            debug!(?err, "impulsion XFixes non envoyée");
                        } else {
                            debug!("XFixes → impulsion");
                        }
                    } else {
                        debug!("XFixes selection coalescée");
                    }
                }
            }
            Some(Event::SelectionRequest(ev)) => {
                if let Some(ref data) = owned_text {
                    handle_selection_request(&conn, &atoms, &ev, data)?;
                }
            }
            Some(Event::SelectionClear(_)) => {
                owned_text = None;
            }
            Some(_) => {}
            None => {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn handle_selection_request(
    conn: &RustConnection,
    atoms: &Atoms,
    ev: &SelectionRequestEvent,
    data: &[u8],
) -> Result<(), RunaError> {
    let property = if ev.target == atoms.targets {
        let values: [u32; 2] = [atoms.utf8, atoms.targets];
        conn.change_property32(
            PropMode::REPLACE,
            ev.requestor,
            ev.property,
            AtomEnum::ATOM,
            &values,
        )
        .map_err(x_err)?;
        ev.property
    } else if ev.target == atoms.utf8 || ev.target == Atom::from(AtomEnum::STRING) {
        conn.change_property8(
            PropMode::REPLACE,
            ev.requestor,
            ev.property,
            atoms.utf8,
            data,
        )
        .map_err(x_err)?;
        ev.property
    } else {
        Atom::from(AtomEnum::NONE)
    };

    let event = SelectionNotifyEvent {
        response_type: SELECTION_NOTIFY_EVENT,
        sequence: 0,
        time: ev.time,
        requestor: ev.requestor,
        selection: ev.selection,
        target: ev.target,
        property,
    };
    conn.send_event(false, ev.requestor, EventMask::NO_EVENT, event)
        .map_err(x_err)?;
    conn.flush().map_err(x_err)?;
    Ok(())
}

fn read_clipboard(conn: &RustConnection, win: Window, atoms: &Atoms) -> Result<String, RunaError> {
    conn.delete_property(win, atoms.property).map_err(x_err)?;
    conn.convert_selection(
        win,
        atoms.clipboard,
        atoms.utf8,
        atoms.property,
        x11rb::CURRENT_TIME,
    )
    .map_err(x_err)?;
    conn.flush().map_err(x_err)?;

    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        if Instant::now() > deadline {
            return Err(RunaError::Clipboard {
                message: "timeout ConvertSelection".into(),
            });
        }
        match conn.poll_for_event().map_err(x_err)? {
            Some(Event::SelectionNotify(ev))
                if ev.property != Atom::from(AtomEnum::NONE) =>
            {
                break;
            }
            Some(Event::SelectionNotify(_)) => {
                return Ok(String::new());
            }
            Some(_) | None => std::thread::sleep(Duration::from_millis(5)),
        }
    }

    let prop = conn
        .get_property(false, win, atoms.property, AtomEnum::ANY, 0, u32::MAX)
        .map_err(x_err)?
        .reply()
        .map_err(x_err)?;

    if prop.type_ == atoms.incr {
        return Err(RunaError::Clipboard {
            message: "transfert INCR non supporté (texte trop long)".into(),
        });
    }

    String::from_utf8(prop.value).map_err(|e| RunaError::Clipboard {
        message: format!("utf-8 X11: {e}"),
    })
}

pub fn stop_watcher(cmd_tx: Sender<ClipCmd>, join: std::thread::JoinHandle<()>) {
    let _ = cmd_tx.send(ClipCmd::Stop);
    if let Err(err) = join.join() {
        warn!(?err, "join x11 clipboard échoué");
    }
}

pub fn cmd_read(cmd_tx: &Sender<ClipCmd>) -> Result<String, RunaError> {
    let (tx, rx) = std_mpsc::channel();
    cmd_tx
        .send(ClipCmd::Read(tx))
        .map_err(|_| RunaError::Clipboard {
            message: "thread x11 arrêté".into(),
        })?;
    rx.recv_timeout(Duration::from_secs(2))
        .map_err(|_| RunaError::Clipboard {
            message: "timeout lecture X11".into(),
        })?
}

pub fn cmd_write(cmd_tx: &Sender<ClipCmd>, text: String) -> Result<(), RunaError> {
    let (tx, rx) = std_mpsc::channel();
    cmd_tx
        .send(ClipCmd::Write { text, reply: tx })
        .map_err(|_| RunaError::Clipboard {
            message: "thread x11 arrêté".into(),
        })?;
    rx.recv_timeout(Duration::from_secs(2))
        .map_err(|_| RunaError::Clipboard {
            message: "timeout écriture X11".into(),
        })?
}

/// Capture la fenêtre active via `_NET_ACTIVE_WINDOW` → PID.
pub fn capture_foreground_pid() -> Result<WindowToken, RunaError> {
    let (conn, screen_num) = RustConnection::connect(None).map_err(focus_err)?;
    let root = conn.setup().roots[screen_num].root;
    let net_active = intern(&conn, "_NET_ACTIVE_WINDOW").map_err(|e| focus_err(e))?;
    let net_pid = intern(&conn, "_NET_WM_PID").map_err(|e| focus_err(e))?;

    let reply = conn
        .get_property(false, root, net_active, AtomEnum::WINDOW, 0, 1)
        .map_err(focus_err)?
        .reply()
        .map_err(focus_err)?;

    let Some(active) = reply.value32().and_then(|mut i| i.next()) else {
        return Ok(WindowToken::Unknown);
    };
    if active == 0 {
        return Ok(WindowToken::Unknown);
    }

    let pid_reply = conn
        .get_property(false, active, net_pid, AtomEnum::CARDINAL, 0, 1)
        .map_err(focus_err)?
        .reply()
        .map_err(focus_err)?;

    let pid = pid_reply
        .value32()
        .and_then(|mut i| i.next())
        .unwrap_or(0);
    Ok(WindowToken::Linux { pid })
}

/// Restaure le focus via message client `_NET_ACTIVE_WINDOW`.
///
/// Cherche la fenêtre par PID dans `_NET_CLIENT_LIST`, puis
/// `_NET_CLIENT_LIST_STACKING` (WM qui reparentent hors enfants directs de root).
pub fn restore_focus(token: &WindowToken) -> Result<(), RunaError> {
    let WindowToken::Linux { pid } = token else {
        return Ok(());
    };
    if *pid == 0 {
        return Ok(());
    }
    let (conn, screen_num) = RustConnection::connect(None).map_err(focus_err)?;
    let root = conn.setup().roots[screen_num].root;
    let net_pid = intern(&conn, "_NET_WM_PID").map_err(|e| focus_err(e))?;
    let net_active = intern(&conn, "_NET_ACTIVE_WINDOW").map_err(|e| focus_err(e))?;

    let win = find_window_by_pid(&conn, root, net_pid, *pid)?.ok_or_else(|| {
        RunaError::WindowFocus {
            message: format!("fenêtre pid={pid} introuvable"),
        }
    })?;

    let data = [1u32, x11rb::CURRENT_TIME, 0, 0, 0];
    let event = ClientMessageEvent {
        response_type: CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window: win,
        type_: net_active,
        data: data.into(),
    };
    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
        event,
    )
    .map_err(focus_err)?;
    conn.flush().map_err(focus_err)?;
    Ok(())
}

fn find_window_by_pid(
    conn: &RustConnection,
    root: Window,
    net_pid: Atom,
    pid: u32,
) -> Result<Option<Window>, RunaError> {
    for atom_name in ["_NET_CLIENT_LIST", "_NET_CLIENT_LIST_STACKING"] {
        let atom = intern(conn, atom_name).map_err(|e| focus_err(e))?;
        let Ok(cookie) = conn.get_property(false, root, atom, AtomEnum::WINDOW, 0, 1024) else {
            continue;
        };
        let Ok(reply) = cookie.reply() else {
            continue;
        };
        let Some(windows) = reply.value32() else {
            continue;
        };
        for win in windows {
            if window_pid(conn, win, net_pid)? == Some(pid) {
                return Ok(Some(win));
            }
        }
    }
    Ok(None)
}

fn window_pid(
    conn: &RustConnection,
    win: Window,
    net_pid: Atom,
) -> Result<Option<u32>, RunaError> {
    let cookie = conn
        .get_property(false, win, net_pid, AtomEnum::CARDINAL, 0, 1)
        .map_err(focus_err)?;
    let reply = cookie.reply().map_err(focus_err)?;
    Ok(reply.value32().and_then(|mut i| i.next()))
}

/// Simule Ctrl+V via XTest (session X11 pure).
pub fn paste_ctrl_v() -> Result<(), RunaError> {
    let (conn, _) = RustConnection::connect(None).map_err(|e| RunaError::InjectionFallback {
        message: format!("X11 connect: {e}"),
    })?;

    // Keycodes X11 courants (layout QWERTY/AZERTY : Control_L=37, V=55).
    const KEY_CONTROL_L: u8 = 37;
    const KEY_V: u8 = 55;

    for (type_, detail) in [
        (KEY_PRESS_EVENT, KEY_CONTROL_L),
        (KEY_PRESS_EVENT, KEY_V),
        (KEY_RELEASE_EVENT, KEY_V),
        (KEY_RELEASE_EVENT, KEY_CONTROL_L),
    ] {
        conn.xtest_fake_input(type_, detail, x11rb::CURRENT_TIME, x11rb::NONE, 0, 0, 0)
            .map_err(|e| RunaError::InjectionFallback {
                message: format!("XTest: {e}"),
            })?;
    }
    conn.flush().map_err(|e| RunaError::InjectionFallback {
        message: format!("XTest flush: {e}"),
    })?;
    Ok(())
}
