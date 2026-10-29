//! Minimal Linux `bpf(2)` UAPI bindings and the socket-marking program builder.
//!
//! The `libc` crate does not expose the BPF syscall's `bpf_attr` union or its
//! constants,
//!  so the small stable-ABI subset used here is defined directly.
//! The program marks sockets with `SO_MARK` so a policy-routing rule can send
//! their traffic outside the WireGuard tunnel.
//!
//! The `dst_src` instruction byte follows Linux's little-endian bitfield layout.
//! Big-endian targets fail at compile time instead of loading a misencoded program.

#[cfg(not(target_endian = "little"))]
compile_error!("wg-quicker-exempt currently supports little-endian Linux targets only");

/// Standard I/O error type used for syscall failures.
use std::io;
/// Owned descriptor that closes automatically when its value leaves scope.
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
/// Stable Linux command constants,
///  attributes,
///  and syscall wrapper.
use crate::bpf_uapi::{
    bpf,
    BpfAttr,
    LinkCreateAttr,
    MapCreateAttr,
    MapUpdateAttr,
    ObjPinAttr,
    ProgLoadAttr,
    BPF_CGROUP_INET4_CONNECT,
    BPF_CGROUP_INET6_CONNECT,
    BPF_CGROUP_UDP4_SENDMSG,
    BPF_CGROUP_UDP6_SENDMSG,
    BPF_LINK_CREATE,
    BPF_MAP_CREATE,
    BPF_MAP_TYPE_ARRAY,
    BPF_MAP_UPDATE_ELEM,
    BPF_OBJ_PIN,
    BPF_PROG_LOAD,
    BPF_PROG_TYPE_CGROUP_SOCK_ADDR,
};

/// eBPF instruction matching kernel `struct bpf_insn`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BpfInsn {
    /// Opcode.
    code: u8,
    /// Destination (low nibble) and source (high nibble) registers.
    dst_src: u8,
    /// Signed offset.
    off: i16,
    /// Signed immediate.
    imm: i32,
}

/// Instruction construction for [`BpfInsn`].
impl BpfInsn {
    /// Builds one instruction.
    const fn new(code: u8, dst: u8, src: u8, off: i16, imm: i32) -> Self {
        return Self {
            code,
            dst_src: (src << 4) | dst,
            off,
            imm,
        };
    }
}

/// Opcode:
///  64-bit immediate load (`ld_imm64`).
const BPF_LD_IMM64: u8 = 0x18;
/// Opcode:
///  64-bit register move.
const BPF_MOV64_REG: u8 = 0xbf;
/// Opcode:
///  32-bit immediate move.
const BPF_MOV32_IMM: u8 = 0xb4;
/// Opcode:
///  64-bit register add immediate.
const BPF_ALU64_ADD: u8 = 0x07;
/// Opcode:
///  store 32-bit immediate to memory (`BPF_ST | BPF_MEM | BPF_W`).
const BPF_ST_MEM32: u8 = 0x62;
/// Opcode:
///  helper call (`BPF_JMP | BPF_CALL | BPF_X` = 0x85).
const BPF_CALL: u8 = 0x85;
/// Opcode:
///  jump-if-equal immediate.
const BPF_JEQ_IMM: u8 = 0x15;
/// Opcode:
///  program exit.
const BPF_EXIT: u8 = 0x95;
/// Pseudo source marking an `ld_imm64` that references a map fd.
const BPF_PSEUDO_MAP_FD: u8 = 1;
/// Helper number:
///  `bpf_map_lookup_elem`.
const FN_MAP_LOOKUP: i32 = 1;
/// Helper number:
///  `bpf_setsockopt`.
const FN_SETSOCKOPT: i32 = 49;
/// `SOL_SOCKET` socket level.
const SOL_SOCKET: i32 = 1;
/// `SO_MARK` socket option.
const SO_MARK: i32 = 36;
/// Byte width of one `SO_MARK` value passed to `bpf_setsockopt`.
const SO_MARK_VALUE_SIZE: i32 = 4;

/// Builds the socket-marking program referencing a mark map fd.
///
/// Sequence:
///  save ctx in r6,
///  look up `mark_map[0]`,
///  and when present call
/// `setsockopt(ctx, SOL_SOCKET, SO_MARK, value_ptr, 4)`,
///  then return 1 (allow).
fn mark_program(map_fd: i32) -> [BpfInsn; 19] {
    return [
        BpfInsn::new(BPF_MOV64_REG, 6, 1, 0, 0),
        BpfInsn::new(BPF_LD_IMM64, 1, BPF_PSEUDO_MAP_FD, 0, map_fd),
        BpfInsn::new(0, 0, 0, 0, 0),
        BpfInsn::new(BPF_MOV64_REG, 2, 10, 0, 0),
        BpfInsn::new(BPF_ALU64_ADD, 2, 0, 0, -4),
        BpfInsn::new(BPF_ST_MEM32, 2, 0, 0, 0),
        BpfInsn::new(BPF_CALL, 0, 0, 0, FN_MAP_LOOKUP),
        // A missing map value skips nine instructions and lands on the allow return.
        BpfInsn::new(BPF_JEQ_IMM, 0, 0, 9, 0),
        BpfInsn::new(BPF_MOV64_REG, 1, 6, 0, 0),
        BpfInsn::new(BPF_MOV32_IMM, 2, 0, 0, SOL_SOCKET),
        BpfInsn::new(BPF_MOV32_IMM, 3, 0, 0, SO_MARK),
        BpfInsn::new(BPF_MOV64_REG, 4, 0, 0, 0),
        BpfInsn::new(BPF_MOV32_IMM, 5, 0, 0, SO_MARK_VALUE_SIZE),
        BpfInsn::new(BPF_CALL, 0, 0, 0, FN_SETSOCKOPT),
        // A zero helper result skips deny and lands on the allow return.
        BpfInsn::new(BPF_JEQ_IMM, 0, 0, 2, 0),
        BpfInsn::new(BPF_MOV32_IMM, 0, 0, 0, 0),
        BpfInsn::new(BPF_EXIT, 0, 0, 0, 0),
        BpfInsn::new(BPF_MOV32_IMM, 0, 0, 0, 1),
        BpfInsn::new(BPF_EXIT, 0, 0, 0, 0),
    ];
}

/// Returns stable opcode,
///  offset,
///  and immediate triples for unit verification.
#[cfg(test)]
pub fn mark_program_snapshot() -> [(u8, i16, i32); 19] {
    let program = mark_program(123);
    let mut snapshot = [(0_u8, 0_i16, 0_i32); 19];
    for (index, instruction) in program.iter().enumerate() {
        snapshot[index] = (instruction.code, instruction.off, instruction.imm);
    }
    return snapshot;
}

/// Creates the one-entry array map holding the mark value.
fn create_mark_map(mark: u32) -> io::Result<OwnedFd> {
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
fn load_prog(map_fd: &OwnedFd, attach_type: u32) -> io::Result<OwnedFd> {
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
    let fd = unsafe { bpf(BPF_PROG_LOAD, &mut attr) }.map_err(|e| {
        let log = String::from_utf8_lossy(&logbuf);
        return io::Error::new(e.kind(), format!("{e}; verifier log: {log}"));
    })?;
    // SAFETY: successful BPF_PROG_LOAD returns a new descriptor owned by this process.
    return Ok(unsafe { OwnedFd::from_raw_fd(fd as i32) });
}

/// Pins a bpf object fd to a path in the bpf filesystem so it survives process exit.
fn pin_obj(fd: &OwnedFd, path: &std::ffi::CStr) -> io::Result<()> {
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
fn attach_link(prog_fd: &OwnedFd, cgroup_fd: i32, attach_type: u32) -> io::Result<OwnedFd> {
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

/// All hooks the marker installs,
///  covering TCP connect and UDP sendmsg,
///  v4 and v6.
pub const HOOKS: [u32; 4] = [
    BPF_CGROUP_INET4_CONNECT,
    BPF_CGROUP_INET6_CONNECT,
    BPF_CGROUP_UDP4_SENDMSG,
    BPF_CGROUP_UDP6_SENDMSG,
];

/// Hook names used for pinned-link file names,
///  parallel to [`HOOKS`].
pub const HOOK_NAMES: [&str; 4] = [
    "connect4",
    "connect6",
    "udp4_sendmsg",
    "udp6_sendmsg",
];

/// Owns unpinned links so dropping this value atomically detaches its marker.
pub struct MarkerLinks {
    /// Link descriptors retain programs and their shared mark map.
    _links: Vec<OwnedFd>,
}

/// Loads and attaches one hook,
///  returning link descriptor that owns attachment.
fn create_link(map_fd: &OwnedFd, cgroup_fd: i32, hook: u32) -> io::Result<OwnedFd> {
    let prog_fd = load_prog(map_fd, hook)?;
    return attach_link(&prog_fd, cgroup_fd, hook);
}

/// Loads,
///  attaches,
///  and pins one hook while descriptors remain automatically owned.
fn attach_one(
    map_fd: &OwnedFd,
    cgroup_fd: i32,
    hook: u32,
    hook_name: &str,
    pin_dir: &str,
) -> io::Result<String> {
    let link_fd = create_link(map_fd, cgroup_fd, hook)?;
    let path = format!("{pin_dir}/{hook_name}");
    let c_path = std::ffi::CString::new(path.as_str()).map_err(|_| {
        return io::Error::new(io::ErrorKind::InvalidInput, "pin path contains NUL");
    })?;
    pin_obj(&link_fd, &c_path)?;
    return Ok(path);
}

/// Removes only link pins created by an interrupted four-hook transaction.
fn rollback_pins(pinned: &[String]) -> io::Result<()> {
    let mut first_error: Option<io::Error> = None;
    for path in pinned {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }
    if let Some(error) = first_error {
        return Err(error);
    }
    return Ok(());
}

/// Attaches links and optionally injects test failure after specified completed count.
fn attach_marker_unpinned_until(
    cgroup_fd: i32,
    mark: u32,
    fail_after: Option<usize>,
) -> io::Result<MarkerLinks> {
    let map_fd = create_mark_map(mark)?;
    let mut links = Vec::with_capacity(HOOKS.len());
    for hook in HOOKS {
        links.push(create_link(&map_fd, cgroup_fd, hook)?);
        if fail_after == Some(links.len()) {
            return Err(io::Error::other("injected partial-link failure"));
        }
    }
    return Ok(MarkerLinks { _links: links });
}

/// Attaches all marker hooks without persistence for privileged protocol verification.
pub fn attach_marker_unpinned(cgroup_fd: i32, mark: u32) -> io::Result<MarkerLinks> {
    return attach_marker_unpinned_until(cgroup_fd, mark, None);
}

/// Injects partial attach failure so descriptor rollback can be observed.
#[cfg(test)]
pub fn attach_marker_unpinned_failing_after(
    cgroup_fd: i32,
    mark: u32,
    link_count: usize,
) -> io::Result<MarkerLinks> {
    return attach_marker_unpinned_until(cgroup_fd, mark, Some(link_count));
}

/// Attaches the socket-marking program to one cgroup for every hook and pins each
/// resulting link under an empty staging directory.
///
/// A failed hook removes every earlier pin,
///  so no partial attachment survives.
pub fn attach_marker(cgroup_fd: i32, mark: u32, pin_dir: &str) -> io::Result<Vec<String>> {
    let map_fd = create_mark_map(mark)?;
    let mut pinned = Vec::with_capacity(HOOKS.len());
    for (index, hook) in HOOKS.iter().enumerate() {
        let result = attach_one(&map_fd, cgroup_fd, *hook, HOOK_NAMES[index], pin_dir);
        match result {
            Ok(path) => pinned.push(path),
            Err(error) => {
                if let Err(rollback_error) = rollback_pins(&pinned) {
                    return Err(io::Error::new(
                        error.kind(),
                        format!("{error}; partial-pin rollback also failed: {rollback_error}"),
                    ));
                }
                return Err(error);
            }
        }
    }
    return Ok(pinned);
}
