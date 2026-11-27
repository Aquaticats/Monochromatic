//! The channel to a user's service manager,
//! and the check that the manager is the process on the other end of it.
//! The watcher calls the manager over that manager's own private socket,
//! `/run/user/<uid>/systemd/private`,
//! which the manager itself created and which only the manager answers:
//! no name lookup happens,
//! so no process can take `org.freedesktop.systemd1` and be called instead.
//! On 2026-11-11 a nested `systemd --user` an agent's probe started took that name on the owner's session bus for hours,
//! and every watcher call went to it,
//! which is why the watcher never asks a bus who the manager is.
//! Before the first byte of a call is written,
//! `SO_PEERCRED` gives the pid behind the socket,
//! and it must be the main process of `user@<uid>.service`
//! as the system manager reports it.
//! A mismatch refuses the call loudly,
//! naming both pids,
//! and nothing is moved.
//! The reason the peer check is enough:
//! the pid comes from the kernel for the connected socket,
//! so a socket the user planted at that path answers with its own pid and is refused.

/// One call over that socket:
/// the login,
/// the peer check,
/// the reply read to the call's one deadline,
/// and the error a call that produced no reply returns.
#[path = "manager_socket_client.rs"]
pub mod client;
/// Where a user's manager answers,
/// and which process must be the one behind that socket.
#[path = "manager_socket_endpoint.rs"]
pub mod endpoint;

/// The client of one user's manager,
/// and what a call that produced no reply returns.
pub use client::{
    ManagerClient,
    ManagerError,
};
/// Where the user's manager of `uid` answers,
/// and which process must be the one behind its socket.
pub use endpoint::{
    ManagerEndpoint,
    manager_main_pid,
};

/// The message layout.
use crate::dbus_wire::{
    Message,
    MethodCall,
};

/// The seam that runs one call against a user's manager,
/// so the policy is testable without a manager and the production path stays in one place.
pub trait ManagerCaller {
    /// Runs one call for the watched user `uid`.
    fn call(
        &self,
        uid: u32,
        call: &MethodCall,
    ) -> Result<Message, ManagerError>;
}

/// The production seam:
/// the manager's private socket,
/// with the endpoint resolved once and again after a refusal or a lost connection.
pub struct PrivateSocketManager {
    /// The client of that socket.
    pub client: ManagerClient,
    /// The endpoint of the watched user's manager,
    /// kept between calls.
    pub endpoint: std::cell::RefCell<Option<ManagerEndpoint>>,
}

/// Construction and the call.
impl PrivateSocketManager {
    /// Builds the production seam.
    pub fn new() -> PrivateSocketManager {
        return PrivateSocketManager {
            client: ManagerClient::new(),
            endpoint: std::cell::RefCell::new(None),
        };
    }

    /// The endpoint of the user's manager,
    /// resolved on the first call and kept until a refusal shows the resolved pid is no longer the manager's.
    fn endpoint(
        &self,
        uid: u32,
    ) -> Result<ManagerEndpoint, ManagerError> {
        let mut slot = self.endpoint.borrow_mut();
        if let Some(endpoint) = slot.as_ref() {
            return Ok(endpoint.clone());
        }
        let endpoint = ManagerEndpoint::resolve(uid).map_err(ManagerError::Failed)?;
        *slot = Some(endpoint.clone());
        return Ok(endpoint);
    }

    /// Forgets the endpoint,
    /// so the next call asks the system manager which process runs the user's manager again.
    fn forget_endpoint(&self) {
        self.endpoint.borrow_mut().take();
    }
}

/// The production seam calls over the manager's private socket.
impl ManagerCaller for PrivateSocketManager {
    /// Runs one call,
    /// with the peer check that refuses a socket answered by anything but the user's manager.
    fn call(
        &self,
        uid: u32,
        call: &MethodCall,
    ) -> Result<Message, ManagerError> {
        let endpoint = self.endpoint(uid)?;
        let outcome = self.client.call(&endpoint, call);
        if matches!(outcome, Err(ManagerError::Refused(_))) {
            self.forget_endpoint();
        }
        return outcome;
    }
}
