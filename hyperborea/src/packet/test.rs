use super::*;
use crate::node::{Node, Address};
use crate::node::test::{ENDPOINTS, STANDARDS};

lazy_static::lazy_static! {
    pub static ref PACKETS: Vec<standards::V1> = {
        let mut base = vec![
            standards::V1::AuthRequest(rand::random::<u128>().to_be_bytes().to_vec()),
            standards::V1::AuthResponse(rand::random::<u128>().to_be_bytes().to_vec())
        ];

        for endpoint in ENDPOINTS.iter().copied() {
            for standard in STANDARDS.iter().copied() {
                let node = Node::new(endpoint, standard);

                base.push(standards::V1::Introduce(node.clone()));
                base.push(standards::V1::SearchRequest { ttl: rand::random(), address: Address::from(standard), respond_to: node.clone() });
                base.push(standards::V1::SearchResponse(node));
            }
        }

        base
    };
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
