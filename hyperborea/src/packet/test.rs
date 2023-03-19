use super::*;

lazy_static::lazy_static! {
    pub static ref PACKETS: Vec<standards::V1> = vec![
        standards::V1::AuthRequest(rand::random::<u128>().to_be_bytes().to_vec()),
        standards::V1::AuthResponse(rand::random::<u128>().to_be_bytes().to_vec())
    ];
}

#[test]
#[cfg(feature = "packet-v1")]
fn test_v1_packets() -> anyhow::Result<()> {
    for packet in PACKETS.iter().cloned() {
        assert_eq!(packet, standards::V1::from_bytes(packet.to_bytes())?);

        let packet = Packet::from(packet);

        assert_eq!(packet, Packet::from_bytes(packet.to_bytes())?);
    }

    Ok(())
}
