use crate::node::*;

pub trait VerifyExt {
    /// Verify signed data
    fn verify<T: AsRef<[u8]>>(&self, data: T, sign: T) -> anyhow::Result<()>;
}

impl VerifyExt for Standard {
    fn verify<T: AsRef<[u8]>>(&self, data: T, sign: T) -> anyhow::Result<()> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => {
                use k256::ecdsa::signature::Verifier;

                let sign = k256::ecdsa::Signature::from_der(sign.as_ref())?;

                k256::ecdsa::VerifyingKey::from_affine(*public_key.as_affine())?
                    .verify(data.as_ref(), &sign)
                    .map_err(|e| e.into())
            }

            _ => unreachable!()
        }
    }
}

impl VerifyExt for Address {
    fn verify<T: AsRef<[u8]>>(&self, data: T, sign: T) -> anyhow::Result<()> {
        #[allow(unreachable_patterns)]
        match &self {
            #[cfg(feature = "node-v1")]
            Address::V1(bytes) => Standard::V1 { public_key: k256::PublicKey::from_sec1_bytes(bytes)? }.verify(data, sign),

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
