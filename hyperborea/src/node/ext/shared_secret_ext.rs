use crate::node::*;

pub trait SharedSecretExt {
    /// Generate shared secret with owned node
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>) -> anyhow::Result<[u8; 1024]>
    where
        T: AsRef<owned::Standard>,
        F: AsRef<[u8]>;
}

impl<S: AsRef<Standard>> SharedSecretExt for S {
    fn shared_secret<T, F>(&self, standard: T, salt: Option<F>) -> anyhow::Result<[u8; 1024]>
    where
        T: AsRef<owned::Standard>,
        F: AsRef<[u8]> 
    {
        #[allow(unreachable_patterns)]
        match self.as_ref() {
            #[cfg(feature = "node-v1")]
            Standard::V1 { public_key } => {
                #[allow(unreachable_patterns)]
                match standard.as_ref() {
                    #[cfg(feature = "node-v1")]
                    owned::Standard::V1 { secret_key } => {
                        use k256::sha2::Sha512;

                        let mut secret = [0; 1024];

                        let result = k256::ecdh::diffie_hellman(secret_key.to_nonzero_scalar(), public_key.as_affine());

                        let result = match salt {
                            Some(salt) => result.extract::<Sha512>(Some(salt.as_ref())),
                            None => result.extract::<Sha512>(None)
                        };

                        let result = result.expand(&[], &mut secret);

                        if let Err(err) = result {
                            anyhow::bail!("Couldn't create shared secret: {err}");
                        }

                        Ok(secret)
                    }

                    _ => anyhow::bail!("Couldn't create shared secret: incompatible standards")
                }
            }

            _ => unreachable!()
        }
    }
}
