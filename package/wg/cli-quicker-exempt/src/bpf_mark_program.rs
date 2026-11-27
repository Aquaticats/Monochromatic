//! The socket-marking program itself:
//! its instruction record,
//! the opcodes and helper numbers it is built from,
//! and the nineteen instructions that mark a socket.

/// eBPF instruction matching kernel `struct bpf_insn`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(super) struct BpfInsn {
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
    const fn new(
        code: u8,
        dst: u8,
        src: u8,
        off: i16,
        imm: i32,
    ) -> Self {
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
pub(super) fn mark_program(map_fd: i32) -> [BpfInsn; 19] {
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
