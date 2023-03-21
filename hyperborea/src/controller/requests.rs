use std::collections::HashMap;
use std::time::{Instant, Duration};

use crate::node::{Address, VerifyExt};
use crate::packet::Packet;
use crate::packet::standards;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    AuthRequest(Vec<u8>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    AuthResponse(Vec<u8>)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Requests {
    pub requests: HashMap<Address, Vec<(Request, Instant)>>
}

impl Requests {
    pub fn get<T: AsRef<Address>>(&self, address: T) -> Option<&Vec<(Request, Instant)>> {
        self.requests.get(address.as_ref())
    }

    pub fn index<T: AsRef<Address>>(&mut self, address: T, packet: Packet) -> Option<&Request> {
        if !self.requests.contains_key(address.as_ref()) {
            self.requests.insert(address.as_ref().to_owned(), vec![]);
        }

        // Always Some
        if let Some(requests) = self.requests.get_mut(address.as_ref()) {
            let request = match packet {
                #[cfg(feature = "packet-v1")]
                Packet::V1(packet) => match packet {
                    standards::V1::AuthRequest(slice) => Request::AuthRequest(slice),

                    _ => return None
                }

                _ => unreachable!()
            };

            requests.push((request, Instant::now()));

            return requests.last().map(|r| &r.0);
        }

        unreachable!()
    }

    pub fn resolve<T: AsRef<Address>>(&mut self, address: T, packet: Packet) -> Option<(Request, Response, Duration)> {
        if let Some(requests) = self.requests.get_mut(address.as_ref()) {
            #[allow(unreachable_patterns)]
            let response = match packet {
                #[cfg(feature = "packet-v1")]
                Packet::V1(packet) => match packet {
                    standards::V1::AuthResponse(slice) => Response::AuthResponse(slice),

                    _ => return None
                }

                _ => unreachable!()
            };

            let address = address.as_ref();

            let found = requests.iter().position(|(request, _)| {
                match (request, &response) {
                    (Request::AuthRequest(original), Response::AuthResponse(signed)) => {
                        address.verify(original, signed).is_ok()
                    }
                }
            });

            if let Some(pos) = found {
                let (request, instant) = requests.remove(pos);

                return Some((
                    request,
                    response,
                    instant.elapsed()
                ));
            }
        }

        None
    }
}