use std::net::{SocketAddrV4, SocketAddrV6, Ipv4Addr, Ipv6Addr};

use super::*;

lazy_static::lazy_static! {
    pub static ref STANDARDS: Vec<Standard> = vec![
        #[cfg(feature = "node-v1")]
        Standard::V1 {
            secret_key: k256::SecretKey::random(&mut rand::thread_rng())
        }
    ];

    pub static ref ENDPOINTS: Vec<SocketAddr> = vec![
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 12345).into(),
        SocketAddrV6::new(Ipv6Addr::LOCALHOST, 12345, 0, 0).into()
    ];
}

#[test]
fn test_standard() -> anyhow::Result<()> {
    for standard in STANDARDS.iter() {
        assert_eq!(Standard::from_bytes(standard.to_bytes())?, *standard);
    }

    Ok(())
}

#[test]
fn test_node() -> anyhow::Result<()> {
    for endpoint in ENDPOINTS.iter() {
        for standard in STANDARDS.iter() {
            let node = Node::new(*endpoint, standard.to_owned());

            assert_eq!(Node::from_bytes(node.to_bytes())?, node);
        }
    }

    Ok(())
}

#[test]
fn test_address() -> anyhow::Result<()> {
    for standard in STANDARDS.iter() {
        let standard_owned: crate::node::owned::Standard = standard.to_owned();
        let standard: crate::node::Standard = standard.to_owned().into();

        let address_owned = Address::from(standard_owned);
        let address = Address::from(standard);

        assert_eq!(address_owned, address);
        assert_eq!(Address::try_from(address_owned.to_string().as_str())?, address_owned);
        assert_eq!(Address::try_from(address.to_string().as_str())?, address);
    }

    Ok(())
}

#[test]
fn test_signing() -> anyhow::Result<()> {
    for endpoint in ENDPOINTS.iter() {
        for standard in STANDARDS.iter() {
            let node = Node::new(*endpoint, standard.to_owned());

            let data = rand::random::<u128>().to_be_bytes();
            let sign = node.sign(data)?;

            node.verify(data.as_slice(), &sign)?;
        }
    }

    Ok(())
}
