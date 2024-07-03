use std::time::Duration;

use hyperborealib::crypto::SecretKey;

#[derive(Debug, Clone)]
pub struct ServerAppParams {
    /// Current server's secret key.
    pub secret_key: SecretKey,

    /// Local server address on which we should run
    /// the HTTP server.
    pub local_address: String,

    /// Address by which other clients can access
    /// current server through the Internet.
    pub remote_address: String,

    /// Bootstrap addresses used to gather
    /// initial information about the network.
    /// 
    /// Usually some static server addresses.
    pub bootstrap: Vec<String>,

    /// Announce current server to other servers.
    /// 
    /// This is needed to allow other servers
    /// to lookup clients of the current server.
    /// 
    /// You want to disable this option if
    /// your server can't be accessed through the internet.
    pub announce: bool,

    /// Network traversing delay.
    /// 
    /// Traversing is performed to gather information
    /// about the network and to fill the routing table.
    /// 
    /// You don't need to perform this too often
    /// because this is a heavy operation.
    pub traverse_delay: Duration
}
