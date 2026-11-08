//! Frame records follow upstream runtime-cabi/src/guest.rs (MIT). Hit testing
//! and contact capture use the official InputEncoder, including cancellation.
use core::ptr;
use pocket_drivers::{MotionState, TextEvent, abi::*};

pub fn text_record(event: &TextEvent) -> Option<PocketTextEvent> {
    let (kind, text, selection_start, selection_end, repeat) = match event {
        TextEvent::Key {
            key,
            pressed,
            repeat,
        } => (
            if *pressed {
                POCKET_TEXT_KEY_DOWN
            } else {
                POCKET_TEXT_KEY_UP
            },
            key.as_str(),
            0,
            0,
            u32::from(*repeat),
        ),
        TextEvent::Commit(text) => (POCKET_TEXT_COMMIT, text.as_str(), 0, 0, 0),
        TextEvent::Composition {
            text,
            selection_start,
            selection_end,
        } => (
            POCKET_TEXT_COMPOSITION,
            text.as_str(),
            *selection_start,
            *selection_end,
            0,
        ),
        TextEvent::CompositionEnd => (POCKET_TEXT_COMPOSITION_END, "", 0, 0, 0),
        _ => return None,
    };
    Some(PocketTextEvent {
        kind,
        text: text.as_ptr(),
        byte_length: text.len(),
        selection_start,
        selection_end,
        repeat,
        ..PocketTextEvent::empty()
    })
}
pub fn motion_header(source: &MotionState) -> PocketMotionState {
    PocketMotionState {
        timestamp: source.timestamp,
        gravity_direction: source.gravity_direction.as_ref().map_or(ptr::null(), |v| v),
        inclination: source.inclination.as_ref().map_or(ptr::null(), |v| v),
        linear_acceleration: source
            .linear_acceleration
            .as_ref()
            .map_or(ptr::null(), |v| v),
        rotation_rate: source.rotation_rate.as_ref().map_or(ptr::null(), |v| v),
        orientation: source.orientation.as_ref().map_or(ptr::null(), |v| v),
        heading: source.heading.as_ref().map_or(ptr::null(), |v| v),
        screen_rotation: source.screen_rotation.as_ref().map_or(ptr::null(), |v| v),
        tilt: source.tilt.as_ref().map_or(ptr::null(), |v| v),
        angles: source.angles.as_ref().map_or(ptr::null(), |v| v),
        ..PocketMotionState::empty()
    }
}
