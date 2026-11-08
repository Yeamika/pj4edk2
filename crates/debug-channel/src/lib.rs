//! Opt-in, bounded opaque data exchange. Device transports belong to the BSP.
#![no_std]
extern crate alloc;

use alloc::{collections::VecDeque, vec::Vec};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_packet_bytes: usize,
    pub max_queued_packets: usize,
    /// Independent byte budget for each direction.
    pub max_queued_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_packet_bytes: 4096,
            max_queued_packets: 8,
            max_queued_bytes: 16384,
        }
    }
}

/// Local connection token. A callback from a previous connection cannot reuse it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Session(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLimits,
    Disabled,
    StaleSession,
    SessionExhausted,
    PacketTooLarge,
    QueueFull,
}

#[derive(Default)]
struct Queue {
    packets: VecDeque<Vec<u8>>,
    bytes: usize,
}

impl Queue {
    fn push(&mut self, bytes: &[u8], limits: Limits) -> Result<(), Error> {
        if bytes.len() > limits.max_packet_bytes {
            return Err(Error::PacketTooLarge);
        }
        if self.packets.len() >= limits.max_queued_packets
            || bytes.len() > limits.max_queued_bytes.saturating_sub(self.bytes)
        {
            return Err(Error::QueueFull);
        }
        self.packets.push_back(bytes.to_vec());
        self.bytes += bytes.len();
        Ok(())
    }

    fn pop(&mut self) -> Option<Vec<u8>> {
        let packet = self.packets.pop_front()?;
        self.bytes -= packet.len();
        Some(packet)
    }
}

/// Own on the runtime thread. Polling never waits or executes received data.
pub struct DebugChannel {
    enabled: bool,
    epoch: u64,
    limits: Limits,
    incoming: Queue,
    outgoing: Queue,
}

impl Default for DebugChannel {
    fn default() -> Self {
        Self::build(Limits::default())
    }
}

impl DebugChannel {
    pub fn new(limits: Limits) -> Result<Self, Error> {
        if limits.max_packet_bytes == 0
            || limits.max_queued_packets == 0
            || limits.max_queued_bytes < limits.max_packet_bytes
        {
            return Err(Error::InvalidLimits);
        }
        Ok(Self::build(limits))
    }

    fn build(limits: Limits) -> Self {
        Self {
            enabled: false,
            epoch: 0,
            limits,
            incoming: Queue::default(),
            outgoing: Queue::default(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn session(&self) -> Option<Session> {
        self.enabled.then_some(Session(self.epoch))
    }

    /// Called for an explicit user choice. Disabling drops data and allocations.
    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), Error> {
        if enabled && !self.enabled {
            self.epoch = self.epoch.checked_add(1).ok_or(Error::SessionExhausted)?;
        }
        self.enabled = enabled;
        if !enabled {
            self.incoming = Queue::default();
            self.outgoing = Queue::default();
        }
        Ok(())
    }

    fn check(&self, session: Session) -> Result<(), Error> {
        if !self.enabled {
            return Err(Error::Disabled);
        }
        if session.0 != self.epoch {
            return Err(Error::StaleSession);
        }
        Ok(())
    }

    /// Runtime -> BSP. The BSP chooses the wire protocol and physical transport.
    pub fn send(&mut self, session: Session, packet: &[u8]) -> Result<(), Error> {
        self.check(session)?;
        self.outgoing.push(packet, self.limits)
    }

    /// BSP -> runtime. No parser, evaluator or command handler is invoked here.
    pub fn deliver(&mut self, session: Session, packet: &[u8]) -> Result<(), Error> {
        self.check(session)?;
        self.incoming.push(packet, self.limits)
    }

    pub fn receive(&mut self, session: Session) -> Result<Option<Vec<u8>>, Error> {
        self.check(session)?;
        Ok(self.incoming.pop())
    }

    pub fn take_outgoing(&mut self, session: Session) -> Result<Option<Vec<u8>>, Error> {
        self.check(session)?;
        Ok(self.outgoing.pop())
    }
}

#[cfg(test)]
mod tests;
