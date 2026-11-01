//! Stable Linux `bpf(2)` UAPI subset used by socket marker loader.

/// Standard syscall error result.
use std::io;

/// `bpf(2)` command:
///  create a map.
pub const BPF_MAP_CREATE: i32 = 0;
/// `bpf(2)` command:
///  update a map element.
pub const BPF_MAP_UPDATE_ELEM: i32 = 2;
/// `bpf(2)` command:
///  load a program.
pub const BPF_PROG_LOAD: i32 = 5;
/// `bpf(2)` command:
///  pin a map,
///  program,
///  or link.
pub const BPF_OBJ_PIN: i32 = 6;
/// `bpf(2)` command:
///  create link against cgroup target.
pub const BPF_LINK_CREATE: i32 = 28;
/// Fixed-size array map type.
pub const BPF_MAP_TYPE_ARRAY: u32 = 2;
/// Cgroup socket-address program type.
pub const BPF_PROG_TYPE_CGROUP_SOCK_ADDR: u32 = 18;
/// IPv4 connect attach type.
pub const BPF_CGROUP_INET4_CONNECT: u32 = 10;
/// IPv6 connect attach type.
pub const BPF_CGROUP_INET6_CONNECT: u32 = 11;
/// IPv4 UDP sendmsg attach type.
pub const BPF_CGROUP_UDP4_SENDMSG: u32 = 14;
/// IPv6 UDP sendmsg attach type.
pub const BPF_CGROUP_UDP6_SENDMSG: u32 = 15;

/// `bpf_attr` map-create arm prefix.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MapCreateAttr {
    /// Map type.
    pub map_type: u32,
    /// Key size in bytes.
    pub key_size: u32,
    /// Value size in bytes.
    pub value_size: u32,
    /// Maximum entry count.
    pub max_entries: u32,
    /// Map creation flags.
    pub map_flags: u32,
    /// Inner map descriptor.
    pub inner_map_fd: u32,
    /// NUMA node.
    pub numa_node: u32,
    /// Kernel-visible map name.
    pub map_name: [i8; 16],
}

/// `bpf_attr` map-update arm.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MapUpdateAttr {
    /// Map descriptor.
    pub map_fd: u32,
    /// Key pointer.
    pub key: u64,
    /// Value pointer.
    pub value: u64,
    /// Update flags.
    pub flags: u64,
}

/// `bpf_attr` program-load arm through expected attach type.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProgLoadAttr {
    /// Program type.
    pub prog_type: u32,
    /// Instruction count.
    pub insn_cnt: u32,
    /// Instruction array pointer.
    pub insns: u64,
    /// License string pointer.
    pub license: u64,
    /// Verifier log level.
    pub log_level: u32,
    /// Verifier log size.
    pub log_size: u32,
    /// Verifier log pointer.
    pub log_buf: u64,
    /// Kernel version for legacy program types.
    pub kern_version: u32,
    /// Program load flags.
    pub prog_flags: u32,
    /// Kernel-visible program name.
    pub prog_name: [i8; 16],
    /// Offload interface index.
    pub prog_ifindex: u32,
    /// Attach type verified with program.
    pub expected_attach_type: u32,
}

/// `bpf_attr` link-create arm.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LinkCreateAttr {
    /// Program descriptor.
    pub prog_fd: u32,
    /// Target cgroup descriptor.
    pub target_fd: u32,
    /// Cgroup attach type.
    pub attach_type: u32,
    /// Link flags.
    pub flags: u32,
}

/// `bpf_attr` object-pin arm.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ObjPinAttr {
    /// Destination path pointer.
    pub pathname: u64,
    /// Object descriptor.
    pub bpf_fd: u32,
    /// Open flags.
    pub file_flags: u32,
    /// Relative path descriptor.
    pub path_fd: u32,
}

/// Minimal union with complete kernel ABI size.
#[repr(C)]
pub union BpfAttr {
    /// Map-create arm.
    pub map_create: MapCreateAttr,
    /// Map-update arm.
    pub map_update: MapUpdateAttr,
    /// Program-load arm.
    pub prog_load: ProgLoadAttr,
    /// Link-create arm.
    pub link_create: LinkCreateAttr,
    /// Object-pin arm.
    pub obj_pin: ObjPinAttr,
    /// Zero-initialization storage.
    pub bytes: [u8; 168],
}

/// Complete `bpf_attr` union size assertion.
const _: () = assert!(std::mem::size_of::<BpfAttr>() == 168);
/// Map-create arm prefix size assertion.
const _: () = assert!(std::mem::size_of::<MapCreateAttr>() == 44);
/// Map-update arm size assertion.
const _: () = assert!(std::mem::size_of::<MapUpdateAttr>() == 32);
/// Program-load arm prefix size assertion.
const _: () = assert!(std::mem::size_of::<ProgLoadAttr>() == 72);
/// Expected attach type ABI offset assertion.
const _: () = assert!(std::mem::offset_of!(ProgLoadAttr, expected_attach_type) == 68);
/// Link-create arm size assertion.
const _: () = assert!(std::mem::size_of::<LinkCreateAttr>() == 16);
/// Object-pin arm size assertion.
const _: () = assert!(std::mem::size_of::<ObjPinAttr>() == 24);

/// Invokes `bpf(2)` with complete union size.
///
/// # Safety
///
/// Caller must initialize union arm selected by `command` and retain pointer inputs through call.
pub unsafe fn bpf(command: i32, attr: *mut BpfAttr) -> io::Result<i64> {
    // SAFETY: caller proves command arm and referenced pointers are valid.
    let result = unsafe {
        libc::syscall(
            libc::SYS_bpf,
            command,
            attr,
            std::mem::size_of::<BpfAttr>(),
        )
    };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(result);
}
