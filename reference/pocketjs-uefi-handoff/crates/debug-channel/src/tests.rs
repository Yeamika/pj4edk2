use super::*;

#[test]
fn opt_in_bidirectional_bytes_and_disconnect_clear_both_queues() {
    let mut channel = DebugChannel::default();
    assert!(!channel.enabled() && channel.session().is_none());
    assert_eq!(
        channel.deliver(Session(0), b"cannot open a channel"),
        Err(Error::Disabled)
    );
    channel.set_enabled(true).unwrap();
    let session = channel.session().unwrap();
    channel.send(session, &[0, 255, 10]).unwrap();
    channel.deliver(session, b"opaque input").unwrap();
    assert_eq!(
        channel.take_outgoing(session).unwrap().unwrap(),
        [0, 255, 10]
    );
    assert_eq!(channel.receive(session).unwrap().unwrap(), b"opaque input");
    channel.send(session, b"discard output").unwrap();
    channel.deliver(session, b"discard input").unwrap();
    channel.set_enabled(false).unwrap();
    assert_eq!(channel.receive(session), Err(Error::Disabled));
    channel.set_enabled(true).unwrap();
    assert_eq!(
        channel.deliver(session, b"late callback"),
        Err(Error::StaleSession)
    );
    let next = channel.session().unwrap();
    assert_eq!(channel.receive(next).unwrap(), None);
    assert_eq!(channel.take_outgoing(next).unwrap(), None);
}

#[test]
fn each_direction_has_independent_packet_and_byte_limits() {
    let mut channel = DebugChannel::new(Limits {
        max_packet_bytes: 4,
        max_queued_packets: 2,
        max_queued_bytes: 6,
    })
    .unwrap();
    channel.set_enabled(true).unwrap();
    let session = channel.session().unwrap();
    assert_eq!(
        channel.deliver(session, b"large"),
        Err(Error::PacketTooLarge)
    );
    channel.deliver(session, b"four").unwrap();
    assert_eq!(channel.deliver(session, b"123"), Err(Error::QueueFull));
    channel.deliver(session, b"12").unwrap();
    assert_eq!(channel.deliver(session, b""), Err(Error::QueueFull));
    channel.send(session, b"data").unwrap();
    channel.receive(session).unwrap();
    channel.deliver(session, b"next").unwrap();
    assert_eq!(channel.receive(session).unwrap().unwrap(), b"12");
    assert_eq!(channel.receive(session).unwrap().unwrap(), b"next");
    assert_eq!(channel.take_outgoing(session).unwrap().unwrap(), b"data");
}
