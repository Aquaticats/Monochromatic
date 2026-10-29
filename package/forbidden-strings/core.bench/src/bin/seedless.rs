//! Diagnostic:
//!  prints the parsed node of every seedless ported rule.
//!
//! What:
//!  ports the real ruleset,
//!  keeps the rules `regex` accepts,
//!  and for each one
//! that this engine parses but finds seedless,
//!  prints its `ours` source and node.
//! Why:
//!  the CsA work needs the exact shapes of the literal-free rules that force
//! the second pass,
//!  so it can decide how to fold or determinize them.

/// The corpus module is unused here but the bench crate shares one module tree.
#[path = "../normalize.rs"]
mod normalize;

/// The per-rule porter.
#[path = "../port.rs"]
mod port;

/// The ruleset loader.
#[path = "../rules.rs"]
mod rules;

/// Prints each seedless ported rule's source and parsed node.
///
/// What:
///  walks the ported pairs and prints the ones this engine deems seedless.
/// Why:
///  a one-shot probe,
///  not a shipped tool.
fn main() {
    let pairs = rules::load_rules();
    let mut count = 0;
    for (ours, _bare) in &pairs {
        if regex::bytes::Regex::new(_bare).is_err() {
            continue;
        }
        if let Some(node) = forbidden_regex::debug_seedless(ours) {
            count += 1;
            println!("--- seedless #{count} ---");
            println!("ours: {ours}");
            println!("node: {node}");
        }
    }
    println!("\ntotal seedless: {count}");
}
