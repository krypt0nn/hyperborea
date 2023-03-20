pub use crate::node::VerifyExt;

use crate::node::owned::{Node, Standard, RemoteStandard};

impl VerifyExt for Standard {
    fn verify<T: AsRef<[u8]>>(&self, data: T, sign: T) -> anyhow::Result<()> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "node-v1")]
            Standard::V1 { secret_key } => RemoteStandard::V1 { public_key: secret_key.public_key() }.verify(data, sign),

            _ => unreachable!()
        }
    }
}

impl VerifyExt for Node {
    #[inline]
    fn verify<T: AsRef<[u8]>>(&self, data: T, sign: T) -> anyhow::Result<()> {
        self.standard.verify(data, sign)
    }
}
