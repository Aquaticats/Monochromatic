//! What:
//!  The name lookup of the standards-review call,
//! run on the blocking pool through `run_blocking_job`.
//! Why:
//!  hyper-util's own resolver calls `tokio::task::spawn_blocking` inside its `call`,
//! and tokio panics there when the system refuses a thread ("OS can't spawn worker thread");
//! the call runs inside a future that the commit service's work loop drives with `block_on`,
//! so that panic would end the work loop's thread.
//! `run_blocking_job` catches the panic and gives an error,
//! so a lookup that cannot get a thread is a failed connection with a plain message,
//! and the review ends with no verdict as it does for any other failed call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const addresses = await dns.promises.lookup(host, { all: true });
//! ```

use crate::blocking_job::run_blocking_job;
use hyper_util::client::legacy::connect::dns::Name;
use std::future::Future;
use std::net::{SocketAddr, ToSocketAddrs};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A resolver for `HttpConnector` that looks a name up on the blocking pool.
#[derive(Clone, Copy, Debug, Default)]
pub struct PoolResolver;

impl tower_service::Service<Name> for PoolResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        return Poll::Ready(Ok(()));
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let host: String = name.as_str().to_string();
        return Box::pin(async move {
            return match run_blocking_job(move || { return (host.as_str(), 0).to_socket_addrs(); }).await {
                Ok(looked_up) => looked_up,
                Err(error) => Err(std::io::Error::other(error.to_string())),
            };
        });
    }
}

/// The lookup is controlled in the test module.
#[cfg(test)]
#[path = "standards_review_client_name_lookup_on_the_blocking_pool_tests.rs"]
mod tests;
