//! What:
//!  One request in the state the kernel needs:
//! the operation,
//! the target it
//!  names,
//! the zeroed command buffer,
//! the packed input list,
//! and the reply
//!  buffer,
//! all owned together so their addresses stay valid for the call.
//! Why:
//!  The kernel is given two addresses into the caller's memory and writes one
//!  of them in place.
//! If the packed list or the reply buffer were a temporary,
//!  the address would outlive the bytes;
//! owning them in one value makes the
//!  lifetime the request's,
//! not the caller's.
//!  The addresses are fixed-width `u64`,
//! and every one is re-derived from the
//!  owned bytes after the reply grows,
//! never hand-computed at a call site.

// The failure type these requests return.
use super::error::Error;
// The typed request numbers.
use super::operation::Operation;
// The reply buffer and what it is for.
use super::reply::{ReplyBuffer, ReplyUse};
// The buffer and struct a request writes into.
use crate::layout::{CommandBuffer, ZfsCmd};
// The checked name types and their field bounds.
use crate::names::{CheckedName, NameField};
// The codec list type.
use crate::nvlist::NvList;
// The formatting traits the diagnostics implement.
use std::fmt;

/// A request ready to be carried to `/dev/zfs` by a live call.
pub struct Request {
    /// The request the kernel is asked for.
    operation: Operation,
    /// The dataset or pool the request names.
    target: String,
    /// The command struct handed to the kernel.
    command: CommandBuffer,
    /// The packed input list,
    /// kept alive for the call.
    input: Vec<u8>,
    /// The reply buffer,
    /// kept alive for the call.
    reply: ReplyBuffer,
}

/// The methods of `Request`.
impl Request {
    /// Build a request whose `zc_name` is `name`,
    /// with a reply buffer sized
    /// for the request's use.
    pub fn new(operation: Operation, name: CheckedName) -> Request {
        let mut command: CommandBuffer = CommandBuffer::new();
        name.write_into(command.command_mut());
        let use_of_reply: ReplyUse = reply_use(operation);
        let reply: ReplyBuffer = ReplyBuffer::new(use_of_reply, 0);
        let target: String = name.as_str().to_string();
        let mut request: Request = Request {
            operation,
            target,
            command,
            input: Vec::new(),
            reply,
        };
        request.refresh_addresses();
        return request;
    }

    /// The operation the kernel is asked for.
    pub fn operation(&self) -> Operation {
        return self.operation;
    }

    /// The dataset or pool the request names.
    pub fn target(&self) -> &str {
        return &self.target;
    }

    /// The command struct as it will be handed over.
    pub fn command(&self) -> &ZfsCmd {
        return self.command.command();
    }

    /// The command struct,
    /// for the fields a live call must restore after a
    /// listing's reply was too small.
    pub fn command_mut(&mut self) -> &mut ZfsCmd {
        return self.command.command_mut();
    }

    /// The fixed-width address of the whole command buffer,
    /// spare room
    /// included:
    /// the ioctl call's third argument.
    pub fn command_abi_address(&mut self) -> u64 {
        return self.command.as_mut_abi_address();
    }

    /// The packed input list exactly as the kernel will read it.
    pub fn input_bytes(&self) -> &[u8] {
        return &self.input;
    }

    /// The reply buffer the kernel writes into.
    pub fn reply(&self) -> &ReplyBuffer {
        return &self.reply;
    }

    /// The reply buffer,
    /// for growing a read's reply after the kernel asked for
    /// more room.
    pub fn reply_mut(&mut self) -> &mut ReplyBuffer {
        return &mut self.reply;
    }

    /// Pack `list` as this request's input and size a write's reply for it.
    ///
    /// # Throws
    ///
    /// [`Error::Encode`] when the list cannot be packed.
    pub fn attach_input(&mut self, list: &NvList) -> Result<(), Error> {
        let packed: Vec<u8> = list.pack().map_err(|source| {
            return Error::Encode {
                operation: self.operation,
                target: self.target.clone(),
                source,
            };
        })?;
        self.input = packed;
        self.reply = ReplyBuffer::new(reply_use(self.operation), self.input.len());
        self.refresh_addresses();
        return Ok(());
    }

    /// Grow the reply buffer to the size the kernel asked for,
    /// then rewrite the
    /// command's reply address and size,
    /// which a reallocation moved.
    ///
    /// # Throws
    ///
    /// [`Error::Refuse`] when the reply may not grow.
    pub fn grow_reply(&mut self, requested: u64) -> Result<usize, Error> {
        let size: usize = self.reply.grow_to_requested(requested).map_err(|source| {
            return Error::Refuse {
                operation: self.operation,
                target: self.target.clone(),
                source,
            };
        })?;
        self.refresh_reply_address();
        return Ok(size);
    }

    /// Decode what the kernel wrote into the reply buffer.
    ///
    /// # Throws
    ///
    /// [`Error::Reply`] when the reply does not decode or claims more bytes
    /// than the buffer holds.
    pub fn take_reply(&self) -> Result<Option<NvList>, Error> {
        let command: &ZfsCmd = self.command();
        return self
            .reply
            .decode(
                command.zc_nvlist_dst_filled != 0,
                command.zc_nvlist_dst_size,
            )
            .map_err(|source| {
                return Error::Reply {
                    operation: self.operation,
                    target: self.target.clone(),
                    source,
                };
            });
    }

    /// Rewrite the command's input and reply addresses from the owned buffers.
    fn refresh_addresses(&mut self) {
        let command: &mut ZfsCmd = self.command.command_mut();
        command.zc_nvlist_src = address_of(&self.input);
        command.zc_nvlist_src_size = self.input.len() as u64;
        command.zc_nvlist_dst_filled = 0;
        self.refresh_reply_address();
    }

    /// Rewrite the command's reply address and size from the owned buffer.
    fn refresh_reply_address(&mut self) {
        let address: u64 = self.reply.as_mut_abi_address();
        let size: u64 = self.reply.as_abi_size();
        let command: &mut ZfsCmd = self.command.command_mut();
        command.zc_nvlist_dst = address;
        command.zc_nvlist_dst_size = size;
    }
}

/// How the value prints in a debug dump.
impl fmt::Debug for Request {
    /// Shows the request identity and buffer sizes,
    /// not their bytes.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return formatter
            .debug_struct("Request")
            .field("operation", &self.operation)
            .field("target", &self.target)
            .field("input_bytes", &self.input.len())
            .field("reply_bytes", &self.reply.len())
            .finish();
    }
}

/// Whether a request's reply may be read again with a bigger buffer.
fn reply_use(operation: Operation) -> ReplyUse {
    if operation.mutates() {
        return ReplyUse::Mutation;
    }
    return ReplyUse::Read;
}

/// The fixed-width ABI address of a buffer's first byte,
/// or zero when it has no
/// bytes:
/// the kernel reads `(address, size)`,
/// and a zero address with size zero
/// is "no list".
fn address_of(bytes: &[u8]) -> u64 {
    if bytes.is_empty() {
        return 0;
    }
    return bytes.as_ptr() as u64;
}

/// Check a name for `field`,
/// reporting the request it was being built for.
///
/// # Throws
///
/// [`Error::Name`] when the text does not fit the field.
pub fn checked_name(
    operation: Operation,
    target: &str,
    field: NameField,
    text: &str,
) -> Result<CheckedName, Error> {
    return CheckedName::new(field, text).map_err(|source| {
        return Error::Name {
            operation,
            target: target.to_string(),
            source,
        };
    });
}

/// Build a list whose only pairs are boolean flags named by `names`,
/// the shape
/// `snaps` and a bookmark property list both take.
pub fn flag_list(names: &[&str]) -> NvList {
    let mut list: NvList = NvList::new();
    for name in names {
        list.add_bool_flag(*name);
    }
    return list;
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;
