//! Verifies socket-marker eBPF control flow and instruction widths without root.

/// Program snapshot helper from sibling BPF module.
use crate::bpf::mark_program_snapshot;

/// Store opcode must write only four key bytes at stack offset minus four.
#[test]
fn key_store_is_mem32() {
    let program = mark_program_snapshot();
    assert_eq!(program[5].0, 0x62);
    assert_eq!(program[5].1, 0);
}

/// Null map lookup must jump directly to allow return and skip setsockopt call.
#[test]
fn null_lookup_jumps_to_allow() {
    let program = mark_program_snapshot();
    assert_eq!(program[7].1, 9);
    assert_eq!(program[17], (0xb4, 0, 1));
}

/// Setsockopt helper result zero reaches allow while nonzero reaches deny.
#[test]
fn setsockopt_result_controls_verdict() {
    let program = mark_program_snapshot();
    assert_eq!(program[13], (0x85, 0, 49));
    assert_eq!(program[14], (0x15, 2, 0));
    assert_eq!(program[15], (0xb4, 0, 0));
    assert_eq!(program[16], (0x95, 0, 0));
    assert_eq!(program[18], (0x95, 0, 0));
}
