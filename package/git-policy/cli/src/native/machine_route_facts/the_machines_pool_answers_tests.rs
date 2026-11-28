//! What:
//!  The controls of the layout gate's verdict,
//!  read into the report's three states without a kernel file or a device.
//! Why:
//!  The gate's refusal names the step it stopped at,
//!  and a refusal at the device proves nothing about the layout,
//!  so the mapping from a verdict to a check's state is the one place that
//!  difference is decided and a control can show both arms.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(abiFactsOf(refusedAt('Device'))).toEqual({ kind: 'unreadable', why });
//! expect(abiFactsOf(refusedAt('Type'))).toEqual({ kind: 'refused', words });
//! ```

/// The mapping a control judges.
use super::abi_facts_of;
/// The report's three states for the layout.
use crate::doctor_zfs_route::zfs_checks::AbiFacts;
/// The facts a pass of the gate read.
use crate::zfs_compatibility_gate::Facts;
/// The gate's verdict and its files.
use crate::zfs_compatibility_gate::Verdict;
/// The binding's refusal and the step it names.
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    CompatibilityRefusal,
    CompatibilityStep,
};

/// One refusal at one step,
/// with the detail a report prints.
fn refused_at(step: CompatibilityStep) -> Verdict {
    return Verdict::Refused(CompatibilityRefusal {
        step,
        file: None,
        detail: String::from("a member moved by four bytes"),
    });
}

/// One pass,
/// with the sizes the gate compared.
fn passed() -> Verdict {
    return Verdict::Compatible(Facts {
        target: "x86_64-unknown-linux-gnu",
        version: Some(String::from("2.4.4-1")),
        absent: Vec::new(),
        base_btf_bytes: 4096,
        module_btf_bytes: 2048,
        header_bytes: (24, 24),
    });
}

/// What:
///  A pass answers a compatible state naming the module's release text and the
///  two sizes.
/// Why:
///  A person reads which module was judged,
///  and no rule compares that text,
///  so it is reported and never a term.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(abiFactsOf(passed())).toMatchObject({ kind: 'compatible', module: '2.4.4-1' });
/// ```
#[test]
fn a_pass_names_the_module_and_the_sizes_it_compared() {
    let facts: AbiFacts = abi_facts_of(passed());
    let AbiFacts::Compatible {
        module,
        details,
    } = facts
    else {
        panic!("a pass is a compatible state: {facts:?}");
    };
    assert_eq!(module.as_deref(), Some("2.4.4-1"));
    assert!(details[0].contains("4096"), "{details:?}");
    assert!(details[1].contains("2048"), "{details:?}");
}

/// What:
///  A refusal at the device answers unknown,
///  and a refusal at any later step answers a finding.
/// Why:
///  The device check already reports a missing or unopenable device,
///  so a second finding about the layout would blame a comparison that never
///  ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(abiFactsOf(refusedAt('Device'))).kind === 'unreadable';
/// expect(abiFactsOf(refusedAt('Member'))).kind === 'refused';
/// ```
#[test]
fn a_refusal_at_the_device_is_unknown_and_a_later_one_is_a_finding() {
    let device: AbiFacts = abi_facts_of(refused_at(CompatibilityStep::Device));
    assert!(
        matches!(device, AbiFacts::Unreadable { .. }),
        "a device refusal is unknown: {device:?}"
    );
    let member: AbiFacts = abi_facts_of(refused_at(CompatibilityStep::Member));
    let AbiFacts::Refused {
        words,
    } = member
    else {
        panic!("a member refusal is a finding: {member:?}");
    };
    assert!(words.contains("four bytes"), "{words}");
}
