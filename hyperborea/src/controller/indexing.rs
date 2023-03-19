use std::collections::HashMap;

use crate::node::{Node, Address};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Save last N nodes, removing old ones once storage is full
    Storage(Option<usize>),

    // TODO:
    // KBucket(usize)
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Storage(None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Storage {
    Storage {
        map: HashMap<Address, Node>,
        size: Option<usize>
    }
}

impl Storage {
    pub fn insert(&mut self, node: Node) {
        match self {
            Storage::Storage { map, size } => {
                if let Some(size) = *size {
                    assert!(size > 0);

                    map.shrink_to(size - 1);
                }

                map.insert(node.address(), node);
            }
        }
    }

    pub fn get<T: AsRef<Address>>(&self, address: T) -> Option<&Node> {
        match self {
            Storage::Storage { map, .. } => map.get(address.as_ref())
        }
    }

    pub fn contains<T: AsRef<Address>>(&self, address: T) -> bool {
        match self {
            Storage::Storage { map, .. } => map.contains_key(address.as_ref())
        }
    }
}

impl From<Strategy> for Storage {
    fn from(strategy: Strategy) -> Self {
        match strategy {
            Strategy::Storage(size) => Self::Storage {
                map: match size {
                    Some(size) => HashMap::with_capacity(size),
                    None => HashMap::new()
                },
                size
            }
        }
    }
}
