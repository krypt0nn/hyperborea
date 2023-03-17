use std::net::{SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};

use super::*;

const STANDARDS: &[Standard] = &[
    Standard::V1
];

#[test]
fn test_standard() -> anyhow::Result<()> {
    for standard in STANDARDS {
        assert_eq!(Standard::from_bytes(standard.to_bytes())?, *standard);
    }

    Ok(())
}

#[test]
fn test_node() -> anyhow::Result<()> {
    let endpoints: &[SocketAddr] = &[
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 12345).into(),
        SocketAddrV6::new(Ipv6Addr::LOCALHOST, 12345, 0, 0).into()
    ];

    for endpoint in endpoints {
        for standard in STANDARDS {
            let node = Node::new(*endpoint, *standard);

            assert_eq!(Node::from_bytes(node.to_bytes())?, node);
        }
    }

    Ok(())
}
