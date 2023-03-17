use super::*;

#[test]
#[cfg(feature = "packet-v1")]
fn test_v1_packets() -> anyhow::Result<()> {
    let packets = [
        standards::V1::AuthRequest(rand::random()),
        standards::V1::AuthResponse(rand::random())
    ];

    for packet in packets {
        assert_eq!(packet, standards::V1::from_bytes(packet.to_bytes())?);

        let packet = Packet::from(packet);

        assert_eq!(packet, Packet::from_bytes(packet.to_bytes())?);
    }

    Ok(())
}
