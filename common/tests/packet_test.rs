use common::packet::{PacketBuilder, PacketType, ReadablePacket};

#[test]
fn readable_packet() {
    let mock_packet = [0x1, 0x2, 0x3, 0x4];

    let mut packet = ReadablePacket::from_bytes(&mock_packet);

    assert!(packet.r#type == PacketType::Info);
    assert_eq!(packet.read(), 0x2);
    assert_eq!(packet.remaining(), 2);
    assert_eq!(packet.read_u16_le(), 1027);
}

#[test]
fn packet_builder() {
    let mut packet = PacketBuilder::new(PacketType::Info);

    packet.write(0x3);
    packet.write_u16_le(8);

    let packet = packet.build();

    assert_eq!(packet, vec![4, 0, 0x1, 0x3, 8, 0]);
}
