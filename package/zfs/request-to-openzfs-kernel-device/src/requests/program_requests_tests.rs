//! What:
//!  The device-free proof of the fixed-program request:
//! the program path,
//! the
//!  argument and the limits land in the input list under the kernel's own key
//!  names,
//! and the optional ones are omitted when not given.
//! Why:
//!  The kernel's `zfs_keys_channel_program` table accepts exactly these names;
//!  a misspelt key is a rejected request,
//! and the primitive has no production
//!  caller whose tests would find it.

use super::*;
use crate::nvlist::{NvData, NvList};
use crate::requests::Operation;

/// The request with every optional member set.
fn full_request() -> Request {
    return channel_program(ChannelProgramRequest {
        pool: "pool",
        program: "/usr/lib/zfs/monochromatic-destroy-program.lua",
        argument: NvData::Str {
            value: "pool/clone".to_string(),
        },
        synchronous: true,
        instruction_limit: Some(1_000),
        memory_limit: Some(2_048),
    })
    .expect("the name fits");
}

#[test]
fn the_program_request_names_the_pool_and_carries_every_member() {
    let request: Request = full_request();
    assert_eq!(request.operation(), Operation::ChannelProgram);
    assert_eq!(request.command().name(), "pool");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(
        input.get_str("program"),
        Some("/usr/lib/zfs/monochromatic-destroy-program.lua")
    );
    assert_eq!(
        input.get("arg"),
        Some(&NvData::Str {
            value: "pool/clone".to_string()
        })
    );
    assert_eq!(input.get("sync"), Some(&NvData::Boolean { value: true }));
    assert_eq!(input.get_u64("instrlimit"), Some(1_000));
    assert_eq!(input.get_u64("memlimit"), Some(2_048));
}

#[test]
fn an_unsynchronous_program_request_omits_the_sync_member() {
    let request: Request = channel_program(ChannelProgramRequest {
        pool: "pool",
        program: "/usr/lib/zfs/program.lua",
        argument: NvData::BooleanFlag,
        synchronous: false,
        instruction_limit: None,
        memory_limit: None,
    })
    .expect("the name fits");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input.get("sync"), None);
    assert_eq!(input.get("instrlimit"), None);
    assert_eq!(input.get("memlimit"), None);
    assert_eq!(input.get("arg"), Some(&NvData::BooleanFlag));
}

#[test]
fn a_nul_in_the_program_path_is_refused_while_packing() {
    let error = channel_program(ChannelProgramRequest {
        pool: "pool",
        program: "/usr/lib/zfs/pro\0gram.lua",
        argument: NvData::BooleanFlag,
        synchronous: false,
        instruction_limit: None,
        memory_limit: None,
    })
    .expect_err("a NUL would shorten the program path");
    assert_eq!(error.operation(), Operation::ChannelProgram);
    assert!(error.to_string().contains("embedded NUL"));
}
