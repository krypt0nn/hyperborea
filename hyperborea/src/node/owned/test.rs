use super::*;

use crate::node::Node as RemoteNode;
use crate::node::SharedSecretExt as _;

use crate::node::test::{
    ENDPOINTS,
    STANDARDS as REMOTE_STANDARDS
};

lazy_static::lazy_static! {
    pub static ref STANDARDS: Vec<Standard> = vec![
        #[cfg(feature = "node-v1")]
        Standard::V1 {
            secret_key: k256::SecretKey::random(&mut rand::thread_rng())
        }
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

#[test]
fn test_shared_secret() -> anyhow::Result<()> {
    for endpoint in ENDPOINTS.iter() {
        for standard in STANDARDS.iter() {
            let node = Node::new(*endpoint, standard.to_owned());

            for standard in REMOTE_STANDARDS.iter() {
                let salt = &rand::random::<u128>().to_be_bytes();

                let secret_owned = node.shared_secret(standard, Some(salt))?;
                let secret_remote = standard.shared_secret(&node.standard, Some(salt))?;

                assert_eq!(secret_owned, secret_remote);
            }
        }
    }

    Ok(())
}
