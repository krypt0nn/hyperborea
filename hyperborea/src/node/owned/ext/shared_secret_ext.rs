use crate::node::SharedSecretExt as _;
use crate::node::owned::{Node, Standard, RemoteStandard};

pub trait SharedSecretExt {
    /// Generate shared secret with remote node
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>, secret_output: &mut [u8]) -> anyhow::Result<()>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>;
}

impl<S: AsRef<Standard>> SharedSecretExt for S {
    #[inline]
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>, secret_output: &mut [u8]) -> anyhow::Result<()>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>
    {
        standard.as_ref().shared_secret(self, salt, secret_output)
    }
}

impl SharedSecretExt for &Node {
    #[inline]
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>, secret_output: &mut [u8]) -> anyhow::Result<()>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>
    {
        standard.as_ref().shared_secret(&self.standard, salt, secret_output)
    }
}

impl SharedSecretExt for Node {
    #[inline]
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>, secret_output: &mut [u8]) -> anyhow::Result<()>
    where
        T: AsRef<RemoteStandard>,
        F: AsRef<[u8]>
    {
        standard.as_ref().shared_secret(&self.standard, salt, secret_output)
    }
}
