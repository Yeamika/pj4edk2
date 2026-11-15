//! Bounded input buffering. Button edges and commands are never overwritten.
use alloc::{collections::VecDeque, vec::Vec};
use pocketjs_boot_ui::UiEvent;

pub const CAPACITY: usize = 128;
// events() reads at most 16 keys and 16 touch reports (release plus click).
pub const POLL_RESERVE: usize = 48;
struct Entry { event: UiEvent, at: u64, motion: bool }
pub struct EventQueue {
    entries: VecDeque<Entry>,
    down: bool,
    pub coalesced: u64,
}
impl EventQueue {
    pub fn new() -> Self {
        Self { entries: VecDeque::with_capacity(CAPACITY), down: false, coalesced: 0 }
    }
    pub fn can_poll(&self) -> bool { CAPACITY - self.entries.len() >= POLL_RESERVE }
    /// On saturation return ownership; the producer must stop reading input.
    pub fn push(&mut self, event: UiEvent, at: u64) -> Result<(), UiEvent> {
        let motion = matches!(&event, UiEvent::Pointer { down, .. } if *down == self.down);
        if motion && self.entries.back().is_some_and(|e| e.motion) {
            *self.entries.back_mut().unwrap() = Entry { event, at, motion };
            self.coalesced += 1;
            return Ok(());
        }
        if self.entries.len() == CAPACITY { return Err(event); }
        if let UiEvent::Pointer { down, .. } = &event { self.down = *down; }
        self.entries.push_back(Entry { event, at, motion });
        Ok(())
    }
    pub fn drain(&mut self, now: u64, output: &mut Vec<UiEvent>) -> u64 {
        let age = self.entries.front().map_or(0, |e| now.saturating_sub(e.at));
        output.extend(self.entries.drain(..).map(|e| e.event));
        age
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pointer(x: f32, down: bool) -> UiEvent { UiEvent::Pointer { x, y: 0., down } }
    #[test]
    fn drag_preserves_edges_and_latest_motion() {
        let mut q = EventQueue::new();
        for e in [pointer(0., true), pointer(1., true), pointer(2., true),
            pointer(2., false), UiEvent::Click { x: 2., y: 0. }] { q.push(e, 10).unwrap(); }
        let mut out = Vec::new();
        assert_eq!(q.drain(25, &mut out), 15);
        assert_eq!(out.len(), 4);
        assert!(matches!(out[0], UiEvent::Pointer { x: 0., down: true, .. }));
        assert!(matches!(out[1], UiEvent::Pointer { x: 2., down: true, .. }));
        assert!(matches!(out[2], UiEvent::Pointer { down: false, .. }));
        assert!(matches!(out[3], UiEvent::Click { .. }));
    }
    #[test]
    fn saturation_returns_command_and_does_not_reorder() {
        let mut q = EventQueue::new();
        for i in 0..CAPACITY { q.push(UiEvent::Key { key: alloc::format!("{i}") }, i as u64).unwrap(); }
        assert!(!q.can_poll());
        assert!(matches!(q.push(UiEvent::ShutdownFailed, 200), Err(UiEvent::ShutdownFailed)));
        let mut out = Vec::new(); q.drain(200, &mut out);
        for (i, e) in out.iter().enumerate() {
            assert!(matches!(e, UiEvent::Key { key } if *key == alloc::format!("{i}")));
        }
        assert!(q.can_poll());
    }
    #[test]
    fn command_separates_motion_and_press_survives_drain() {
        let mut q = EventQueue::new(); let mut out = Vec::new();
        q.push(pointer(0., true), 0).unwrap(); q.drain(1, &mut out); out.clear();
        for e in [pointer(1., true), UiEvent::Key { key: "Enter".into() }, pointer(2., true), pointer(3., false)] {
            q.push(e, 2).unwrap();
        }
        q.drain(3, &mut out); assert_eq!(out.len(), 4);
    }
}
