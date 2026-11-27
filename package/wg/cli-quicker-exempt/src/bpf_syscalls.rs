//! The `bpf(2)` calls one attach is made of:
//! creating the mark map,
//! loading a program,
//! pinning an object,
//! and attaching a link.

/// The program this module loads.
use super::mark_program::mark_program;
/// Stable Linux command constants,
///  attributes,
///  and syscall wrapper.
use crate::bpf_uapi::{
    BPF_LINK_CREATE,
    BPF_MAP_CREATE,
    BPF_MAP_TYPE_ARRAY,
    BPF_MAP_UPDATE_ELEM,
    BPF_OBJ_PIN,
    BPF_PROG_LOAD,
    BPF_PROG_TYPE_CGROUP_SOCK_ADDR,
    BpfAttr,
    LinkCreateAttr,
    MapCreateAttr,
    MapUpdateAttr,
    ObjPinAttr,
    ProgLoadAttr,
    bpf,
};
/// Standard I/O error type used for syscall failures.
use std::io;
/// Owned descriptor that closes automatically when its value leaves scope.
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};

/// Creates the one-entry array map holding the mark value.
pub(super) fn create_mark_map(mark: u32) -> io::Result<OwnedFd> {
    let mut attr = BpfAttr {
        bytes: [0; 168],
    };
    let mut map_name = [0i8; 16];
    map_name[..4].copy_from_slice(&[b'm' as i8, b'a' as i8, b'r' as i8, b'k' as i8]);
    attr.map_create = MapCreateAttr {
        map_type: BPF_MAP_TYPE_ARRAY,
        key_size: 4,
        value_size: 4,
        max_entries: 1,
        map_flags: 0,
        inner_map_fd: 0,
        numa_node: 0,
        map_name,
    };
    // SAFETY: map_create arm initialized.
    let map_fd = unsafe { bpf(BPF_MAP_CREATE, &mut attr)? } as i32;
    // SAFETY: successful BPF_MAP_CREATE returns a new descriptor owned by this process.
    let owned_map_fd = unsafe { OwnedFd::from_raw_fd(map_fd) };

    let key: u32 = 0;
    let mut uattr = BpfAttr {
        bytes: [0; 168],
    };
    uattr.map_update = MapUpdateAttr {
        map_fd: owned_map_fd.as_raw_fd() as u32,
        key: &key as *const u32 as u64,
        value: &mark as *const u32 as u64,
        flags: 0,
    };
    // SAFETY: map_update arm initialized; key/mark outlive the call.
    unsafe { bpf(BPF_MAP_UPDATE_ELEM, &mut uattr)? };
    return Ok(owned_map_fd);
}

/// Loads the mark program for one hook and returns its fd.
pub(super) fn load_prog(
    map_fd: &OwnedFd,
    attach_type: u32,
) -> io::Result<OwnedFd> {
    let prog = mark_program(map_fd.as_raw_fd());
    let mut logbuf = vec![0u8; 65536];
    let license = b"GPL\0";
    let mut attr = BpfAttr {
        bytes: [0; 168],
    };
    attr.prog_load = ProgLoadAttr {
        prog_type: BPF_PROG_TYPE_CGROUP_SOCK_ADDR,
        insn_cnt: prog.len() as u32,
        insns: prog.as_ptr() as u64,
        license: license.as_ptr() as u64,
        log_level: 1,
        log_size: logbuf.len() as u32,
        log_buf: logbuf.as_mut_ptr() as u64,
        kern_version: 0,
        prog_flags: 0,
        prog_name: [0; 16],
        prog_ifindex: 0,
        expected_attach_type: attach_type,
    };
    // SAFETY: prog_load arm initialized; prog/license/logbuf outlive the call.
    let fd = match unsafe { bpf(BPF_PROG_LOAD, &mut attr) } {
        Ok(fd) => fd,
        Err(error) => {
            let log = String::from_utf8_lossy(&logbuf);
            return Err(io::Error::new(
                error.kind(),
                format!("{error}; verifier log: {log}"),
            ));
        }
    };
    // SAFETY: successful BPF_PROG_LOAD returns a new descriptor owned by this process.
    return Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) });
}

/// Reports the link pin path error for an embedded NUL byte.
pub(super) fn pin_path_contains_nul(_error: std::ffi::NulError) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidInput, "pin path contains NUL");
}

/// Pins a bpf object fd to a path in the bpf filesystem so it survives process exit.
pub(super) fn pin_obj(
    fd: &OwnedFd,
    path: &std::ffi::CStr,
) -> io::Result<()> {
    let mut attr = BpfAttr {
        bytes: [0; 168],
    };
    attr.obj_pin = ObjPinAttr {
        pathname: path.as_ptr() as u64,
        bpf_fd: fd.as_raw_fd() as u32,
        file_flags: 0,
        path_fd: 0,
    };
    // SAFETY: obj_pin arm initialized; path outlives the call.
    let result = unsafe { bpf(BPF_OBJ_PIN, &mut attr) };
    if let Err(error) = result {
        if error.kind() == io::ErrorKind::InvalidInput {
            return Err(crate::bpf_error::pin_object_invalid(
                path.to_string_lossy().into_owned(),
                error,
            ));
        }
        return Err(io::Error::new(
            error.kind(),
            format!("BPF_OBJ_PIN {}: {error}", path.to_string_lossy()),
        ));
    }
    return Ok(());
}

/// Attaches a program to a cgroup via `BPF_LINK_CREATE`,
///  returning the link fd.
pub(super) fn attach_link(
    prog_fd: &OwnedFd,
    cgroup_fd: i32,
    attach_type: u32,
) -> io::Result<OwnedFd> {
    let mut attr = BpfAttr {
        bytes: [0; 168],
    };
    attr.link_create = LinkCreateAttr {
        prog_fd: prog_fd.as_raw_fd() as u32,
        target_fd: cgroup_fd as u32,
        attach_type,
        flags: 0,
    };
    // SAFETY: link_create arm initialized.
    let fd = unsafe { bpf(BPF_LINK_CREATE, &mut attr)? };
    // SAFETY: successful BPF_LINK_CREATE returns a new descriptor owned by this process.
    return Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) });
}
