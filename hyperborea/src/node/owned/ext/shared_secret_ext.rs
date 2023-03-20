use crate::node::SharedSecretExt as _;
use crate::node::owned::{Node, Standard, RemoteStandard};

pub trait SharedSecretExt {
    /// Generate shared secret with remote node
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>) -> anyhow::Result<[u8; 1024]>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>;
}

impl<S: AsRef<Standard>> SharedSecretExt for S {
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>) -> anyhow::Result<[u8; 1024]>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>
    {
        standard.as_ref().shared_secret(self, salt)
    }
}

impl SharedSecretExt for &Node {
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>) -> anyhow::Result<[u8; 1024]>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>
    {
        standard.as_ref().shared_secret(&self.standard, salt)
    }
}
