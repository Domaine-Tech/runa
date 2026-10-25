//! Sondes data-control (ext / wlr) sans démarrer le watcher.

use tracing::debug;
use wayland_client::{
    protocol::wl_registry, Connection, Dispatch, QueueHandle,
};

use crate::linux_select::ProbeResult;

#[derive(Default)]
struct ProbeState {
    ext: bool,
    wlr: bool,
}

impl Dispatch<wl_registry::WlRegistry, ()> for ProbeState {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            interface,
            version: _,
            name: _,
        } = event
        {
            if interface == "ext_data_control_manager_v1" {
                state.ext = true;
            }
            if interface == "zwlr_data_control_manager_v1" {
                state.wlr = true;
            }
        }
    }
}

/// Connecte brièvement au compositor et liste les globals data-control.
pub fn probe_data_control() -> (ProbeResult, ProbeResult) {
    let Ok(conn) = Connection::connect_to_env() else {
        debug!("Wayland: connexion impossible pour sonde data-control");
        return (ProbeResult::Missing, ProbeResult::Missing);
    };
    let display = conn.display();
    let mut event_queue = conn.new_event_queue::<ProbeState>();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut state = ProbeState::default();
    if let Err(err) = event_queue.roundtrip(&mut state) {
        debug!(?err, "Wayland: roundtrip sonde échoué");
        return (ProbeResult::Missing, ProbeResult::Missing);
    }
    (
        if state.ext {
            ProbeResult::Available
        } else {
            ProbeResult::Missing
        },
        if state.wlr {
            ProbeResult::Available
        } else {
            ProbeResult::Missing
        },
    )
}
