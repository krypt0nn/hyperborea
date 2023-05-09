use super::Standard;
use super::indexing::Strategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Params {
    /// Controller standard
    /// 
    /// Default is `Standard::default()`
    pub standard: Standard,

    /// Nodes indexing params
    pub indexing: IndexingParams,

    /// Support V1 nodes standard
    /// 
    /// Default is `true`
    pub support_v1: bool,

    /// Replace endpoint addresses in packet nodes
    /// by socket address which sent you this packet
    /// 
    /// Note that this can break compatibility with some systems
    /// 
    /// Default is `false`
    pub use_real_endpoint: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            standard: Standard::default(),
            indexing: IndexingParams::default(),
            support_v1: true,
            use_real_endpoint: false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexingParams {
    /// Algorithm used to store and share remote nodes
    /// 
    /// Default is `Strategy::default()`
    pub strategy: Strategy,

    /// Use naive indexing strategy (always index `Introduce` nodes)
    /// without verifying them using `AuthRequest` packet
    /// 
    /// This feature can be abused by malicious nodes, but it also
    /// significantly reduces amount of sent UDP packets
    /// 
    /// Default is `false`
    pub naive: bool,

    /// Index nodes from every incoming packet
    /// 
    /// This feature won't verify indexing nodes using `AuthRequest` packet
    /// 
    /// Default is `false`
    pub aggressive: bool
}

#[allow(clippy::derivable_impls)]
impl Default for IndexingParams {
    fn default() -> Self {
        Self {
            strategy: Strategy::default(),
            naive: false,
            aggressive: false
        }
    }
}
