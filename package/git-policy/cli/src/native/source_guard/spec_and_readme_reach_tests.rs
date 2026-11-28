//! What:
//!  The passage each piece of the ZFS route that was built ahead of the command
//!  that would reach it has in `SPEC.md` and in `README.md`,
//!  saying what is built and that no command reaches it in this release.
//! Why:
//!  The release pass found steps 17 to 21 merged with no passage for any of
//!  them,
//!  so a reader of either document could not learn that the cancel command,
//!  the forced arm,
//!  the route's `doctor` checks,
//!  the boot units,
//!  Sanoid's coverage and the dependency repair exist,
//!  or that nothing reaches them
//!  (alpha.47's release pass,
//!  finding 8,
//!  and `doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
//!  A later round that wires one of them must say so in both documents,
//!  and this control fails until it does.
//!  The markers are headings and whole sentences the passages carry,
//!  never a phrase a paragraph could hold by accident.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const [topic, inSpec, inReadme] of reachPassages) {
//!   expect(spec).toContain(inSpec);
//!   expect(readme).toContain(inReadme);
//! }
//! ```

/// One piece built ahead of its command,
/// with the text each document carries for it.
struct ReachPassage {
    /// What the piece is,
    /// named as the release pass named it.
    topic: &'static str,
    /// The text `SPEC.md` carries for it.
    in_spec: &'static str,
    /// The text `README.md` carries for it.
    in_readme: &'static str,
}

/// The passages this release documents for the pieces no command reaches.
const REACH_PASSAGES: &[ReachPassage] = &[
    ReachPassage {
        topic: "the cancel command",
        in_spec: "#### The cancel command, built and reachable from no command",
        in_readme: "The cancel command is built and no management grammar admits it in this release:",
    },
    ReachPassage {
        topic: "the forced arm and its never-end set",
        in_spec: "#### The forced arm, built and reachable from no command",
        in_readme: "The forced arm is built beside the cancel command and reached by no command.",
    },
    ReachPassage {
        topic: "the route's `doctor` checks",
        in_spec: "#### The ZFS route's checks, built and reached by no command",
        in_readme: "The route's checks are built in `src/native/health/doctor_zfs_route/`",
    },
    ReachPassage {
        topic: "the boot and shutdown units",
        in_spec: "#### The boot and shutdown units, built and reached by no command",
        in_readme: "#### The boot and shutdown units, built and reached by no command",
    },
    ReachPassage {
        topic: "Sanoid coverage and the space guard's scope",
        in_spec: "#### Sanoid coverage and the space guard's scope, built and reached by no command",
        in_readme: "#### Sanoid coverage and the space guard's scope, built and reached by no command",
    },
    ReachPassage {
        topic: "the clone route's repair and its watches",
        in_spec: "#### The clone route's repair and its watches, built and reached by no command",
        in_readme: "#### The clone route's repair and its watches, built and reached by no command",
    },
    ReachPassage {
        topic: "the lost-event triggers' event filter and merge device",
        in_spec: "#### The event filter and the merge device, built and reached by no command",
        in_readme: "#### The event filter and the merge device, built and reached by no command",
    },
];

/// What:
///  Both release documents carry one passage for every piece no command
///  reaches.
/// Why:
///  The plan's "SPEC.md passages" row asks each step for one,
///  and the release that merges a step is the one that owes it;
///  a passage deleted from one document fails here instead of being found by the
///  next pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(missing).toEqual([]);
/// ```
#[test]
fn every_piece_built_ahead_has_its_passage_in_both_documents() {
    let spec: String = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/SPEC.md"))
        .expect("the specification");
    let readme: String = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("the release's readme");
    for passage in REACH_PASSAGES {
        assert!(
            spec.contains(passage.in_spec),
            "SPEC.md carries no passage for {}: it needs {:?}",
            passage.topic,
            passage.in_spec
        );
        assert!(
            readme.contains(passage.in_readme),
            "README.md carries no passage for {}: it needs {:?}",
            passage.topic,
            passage.in_readme
        );
    }
}
