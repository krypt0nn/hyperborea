use crate::node::owned::*;

pub trait SignExt {
    /// Sign data
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> anyhow::Result<Vec<u8>>;
}

impl SignExt for Standard {
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> anyhow::Result<Vec<u8>> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "node-v1")]
            Standard::V1 { secret_key } => {
                use k256::ecdsa::signature::Signer;

                let signer = k256::ecdsa::SigningKey::from_bytes(&secret_key.to_bytes())?;
                let sign: k256::ecdsa::Signature = signer.try_sign(data.as_ref())?;

                Ok(sign.to_der().to_bytes().to_vec())
            }

            _ => unreachable!()
        }
    }
}

impl SignExt for Node {
    #[inline]
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> anyhow::Result<Vec<u8>> {
        self.standard.sign(data)
    }
}
