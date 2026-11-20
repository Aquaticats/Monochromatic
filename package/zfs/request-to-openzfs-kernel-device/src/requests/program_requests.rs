//! What:
//!  `ZFS_IOC_CHANNEL_PROGRAM`:
//! run one fixed root-owned program in the kernel
//!  with an argument and limits,
//! for the conditional destruction the root
//!  removal alternative would need.
//! Why:
//!  The project did not select that root alternative,
//! so nothing in this
//!  repository sends this request;
//! the primitive exists in the binding because
//!  the design asks for it to be testable in the disposable VM,
//! never as a
//!  production caller.
//!  The program is a caller-supplied string in the input list,
//! so a live caller
//!  must name a root-owned file;
//! this crate has no way to guess one.

/// The failure type these requests return.
use super::error::Error;
/// The typed request numbers.
use super::operation::Operation;
/// The request value and the name check.
use super::request::{Request, checked_name};
/// The field a request name is bound for.
use crate::names::NameField;
/// The codec list and value types.
use crate::nvlist::{NvData, NvList};

/// One `ZFS_IOC_CHANNEL_PROGRAM` request's inputs.
pub struct ChannelProgramRequest<'a> {
    /// The pool the program runs against;
    /// it travels in `zc_name`.
    pub pool: &'a str,
    /// The fixed root-owned program's path.
    pub program: &'a str,
    /// The argument the program receives,
    /// as an nvlist value.
    pub argument: NvData,
    /// Whether the program runs in syncing context.
    pub synchronous: bool,
    /// Most Lua instructions the program may run,
    /// when the caller bounds it.
    pub instruction_limit: Option<u64>,
    /// Most bytes the program may allocate,
    /// when the caller bounds it.
    pub memory_limit: Option<u64>,
}

/// `ZFS_IOC_CHANNEL_PROGRAM` with the inputs of `request`.
///
/// # Throws
///
/// [`Error::Name`] when the pool does not fit `zc_name`,
/// and
/// [`Error::Encode`] when the program path,
/// argument or limits cannot be
/// packed.
pub fn channel_program(
    ChannelProgramRequest {
        pool,
        program,
        argument,
        synchronous,
        instruction_limit,
        memory_limit,
    }: ChannelProgramRequest<'_>,
) -> Result<Request, Error> {
    let name = checked_name(
        Operation::ChannelProgram,
        pool,
        NameField::CommandName,
        pool,
    )?;
    let mut input: NvList = NvList::new();
    input.add_str("program", program);
    input.push("arg", argument);
    if synchronous {
        input.add_boolean("sync", true);
    }
    if let Some(limit) = instruction_limit {
        input.add_u64("instrlimit", limit);
    }
    if let Some(limit) = memory_limit {
        input.add_u64("memlimit", limit);
    }
    let mut request: Request = Request::new(Operation::ChannelProgram, name);
    request.attach_input(&input)?;
    return Ok(request);
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "program_requests_tests.rs"]
mod tests;
