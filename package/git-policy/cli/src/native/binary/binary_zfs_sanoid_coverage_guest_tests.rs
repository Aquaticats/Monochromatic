//! What:
//!  Plan step 20's Sanoid arms in a throwaway guest:
//!  recursive coverage,
//!  an exclusion's section order,
//!  the `recursive = zfs` control,
//!  the retention prune,
//!  each drift's finding and printed correction,
//! and the space guard's scope,
//!  measured against a real file-backed pool with Sanoid 2.3.0 and the
//!  Ubuntu package's 2.2.0 beside it.
//! Why:
//!  The section-order effect and the exclusion are behaviours of Sanoid
//!  itself,
//! not of this crate,
//!  so only a real Sanoid run on a real dataset tree shows them,
//!  and the project's 2.3.0 measurements needed the 2.2.0 guest's result
//!  beside them
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 20,
//!  "VM measurements";
//!  `doc/planning/cli-git-zfs-clones-mounted-under-the-checkout.md`,
//! item 4).
//!  The arm prints `SKIP` where `CLI_GIT_ZFS_SANOID_GUEST` does not name the
//!  guest,
//!  and it creates every pool,
//! dataset,
//! property and configuration it reads
//!  under one fixture folder,
//!  so a run leaves the guest as it found it and this machine is never
//!  touched.
//!  Every claim is a marker line the driver requires,
//!  so a section that silently stopped running fails the run instead of
//!  printing less.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (process.env.CLI_GIT_ZFS_SANOID_GUEST !== guest) return print('SKIP');
//! buildLayout(); expect(checkCoverage(gather()).outcome.kind).toBe('pass');
//! ```

use git_policy_cli::doctor_zfs_route::CheckOutcome;
use git_policy_cli::sanoid_coverage::{
    AUTO_SNAPSHOT_OFF,
    AUTO_SNAPSHOT_PROPERTY,
    CoverageAnswer,
    CoverageDecision,
    CoverageFacts,
    CoverageFinding,
    Exclusion,
    ExclusionMount,
    FIELD_EXCLUSION_MARKER,
    FIELD_RECURSION,
    FIELD_SECTION_ORDER,
    FIELD_STORE_COVERAGE,
    GENERATED_HEADER,
    MeasuredDataset,
    MeasuredTree,
    ScopeAnswer,
    SpaceGuardFacts,
    check_coverage,
    check_space_guard_scope,
    decided_coverage,
    generated_text,
    read_sanoid_conf,
    render_section,
    render_sections,
    sections_of,
};
use std::fs;
use std::process::Command;

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SANOID_GUEST";

/// Where the arm builds everything it reads,
/// under one name of its own so two runs in one guest share nothing.
const FIXTURE_BASE: &str = "/home/sanoid-step-20-fixture";

/// The pool image's size.
const IMAGE_SIZE: &str = "1G";

/// The Sanoid 2.3.0 script the driver installed.
const SANOID_MEASURED: &str = "/opt/sanoid-2.3.0/sanoid";

/// The defaults file that ships with Sanoid 2.3.0.
const DEFAULTS_MEASURED: &str = "/opt/sanoid-2.3.0/sanoid.defaults.conf";

/// The Sanoid script the Ubuntu package installed.
const SANOID_CONTROL: &str = "/usr/sbin/sanoid";

/// The defaults file that ships with the Ubuntu package,
/// which the package puts under `share` and not under `etc`.
const DEFAULTS_CONTROL: &str = "/usr/share/sanoid/sanoid.defaults.conf";

/// The prefix every snapshot Sanoid takes begins with.
const AUTOSNAP: &str = "autosnap_";

/// The prefix a clone's pinned origin snapshot begins with.
const ORIGIN: &str = "git-worktree-clone-origin-";

/// The nonce the arm's pinned origin uses.
const ORIGIN_NONCE: &str = "0123456789abcdef0123456789abcdef";

/// The one dataset the order arms name as an exclusion under the ancestor,
/// because the host's exclusions all stand outside its subtree and only a
/// child dataset shows Sanoid's order effect.
const SCRATCH: &str = "ROOT/cos/home/scratch";

/// The subtrees the arm builds,
/// each its own dataset tree so no two Sanoid runs take the same snapshot
/// name,
/// with whether the subtree holds the child dataset the order arms exclude.
const SUBTREES: &[(&str, bool)] = &[
    ("decided", false),
    ("order-ok", true),
    ("order-bad", true),
    ("zfs", true),
    ("control-ok", true),
    ("control-bad", true),
];

//region Commands

/// Run one program and print what it said when it fails.
fn run(
    program: &str,
    arguments: &[&str],
) -> String {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("{program} could not be started: {error}"));
    if !output.status.success() {
        panic!(
            "{program} {arguments:?} failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice())
        );
    }
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// Run one `zfs` request.
fn zfs(arguments: &[&str]) -> String {
    return run("zfs", arguments);
}

/// Whether one dataset exists.
fn exists(dataset: &str) -> bool {
    let output = Command::new("zfs")
        .args(["list", "-H", "-o", "name", dataset])
        .output()
        .expect("zfs runs");
    return output.status.success();
}

/// Create every dataset above one in the pool,
/// unmounted,
/// so `zfs create` finds a parent without mounting a container in the tree.
fn create_ancestors(
    pool: &Pool,
    dataset: &str,
) {
    let leaf: &str = dataset
        .strip_prefix(format!("{}/", pool.name).as_str())
        .expect("a dataset under the pool");
    let parts: Vec<&str> = leaf.split('/').collect();
    for index in 1..parts.len() {
        let ancestor: String = format!("{}/{}", pool.name, parts[..index].join("/"));
        if !exists(ancestor.as_str()) {
            zfs(&[
                "create",
                "-o",
                "mountpoint=none",
                "-o",
                "canmount=off",
                ancestor.as_str(),
            ]);
        }
    }
}

/// Create one dataset with the properties the arm gives it.
fn create(
    pool: &Pool,
    dataset: &str,
    properties: &[String],
) {
    create_ancestors(pool, dataset);
    let mut arguments: Vec<&str> = vec!["create"];
    for property in properties {
        arguments.push("-o");
        arguments.push(property.as_str());
    }
    arguments.push(dataset);
    zfs(arguments.as_slice());
}

/// The pool and its image.
struct Pool {
    /// The pool's name,
    /// which carries the process's number so a second run cannot reach it.
    name: String,
    /// The folder this run's fixtures live in.
    fixture: String,
}

impl Pool {
    /// A file-backed pool,
    /// so no device of the guest is touched.
    /// Nothing is destroyed first:
    /// the pool's name and the fixture folder carry the process's number,
    /// so a repeat run in one guest shares no dataset and no configuration.
    fn create() -> Pool {
        let number: u32 = std::process::id();
        let fixture: String = format!("{FIXTURE_BASE}-{number}");
        fs::create_dir_all(fixture.as_str()).expect("the fixture folder");
        let image: String = format!("{fixture}/pool.img");
        run("truncate", &["--size", IMAGE_SIZE, image.as_str()]);
        let name: String = format!("sanoidstep20x{number}");
        run("zpool", &["create", "-f", name.as_str(), image.as_str()]);
        return Pool {
            name,
            fixture,
        };
    }

    /// One dataset of one subtree.
    fn dataset(
        &self,
        subtree: &str,
        leaf: &str,
    ) -> String {
        return format!("{}/{subtree}/{leaf}", self.name);
    }
}

//endregion Commands

//region Layout

/// The properties one leaf's dataset is created with,
/// which give the host's own mount points inside the guest's fixture.
fn mount_properties(
    root: &str,
    leaf: &str,
) -> Vec<String> {
    let path: Option<String> = match leaf {
        "nosnap" | "nosnap/git-worktree-probe" => None,
        "ROOT/cos/varcache" => Some(format!("{root}/var/cache")),
        "nosnap/cargo-build" => Some(format!("{root}/home/.cargo/build")),
        "nosnap/cargo-registry" => Some(format!("{root}/home/.cargo/registry")),
        "nosnap/npm" => Some(format!("{root}/home/.npm")),
        "nosnap/user-cache" => Some(format!("{root}/home/.cache")),
        "nosnap/vm-builder-output" => Some(format!("{root}/home/vm-output")),
        "nosnap/libvirt-images" => Some(format!("{root}/var/lib/libvirt/images")),
        other => panic!("the layout gives {other} no mount point"),
    };
    return match leaf {
        "nosnap" => vec![
            String::from("mountpoint=none"),
            String::from("canmount=off"),
        ],
        "nosnap/git-worktree-probe" => vec![
            String::from("mountpoint=legacy"),
            String::from("canmount=off"),
        ],
        _ => vec![format!(
            "mountpoint={}",
            path.expect("a mounted leaf has a path")
        )],
    };
}

/// The coverage one guest subtree is laid out for,
/// with the host's decided names under the guest's pool.
fn decision_for(
    pool: &Pool,
    subtree: &str,
    with_scratch: bool,
) -> CoverageDecision {
    let mut exclusions: Vec<Exclusion> = Vec::new();
    for exclusion in decided_coverage().exclusions {
        let leaf: &str = exclusion
            .dataset
            .strip_prefix("zpcachyos/")
            .expect("the decided names stand under the host's pool");
        exclusions.push(Exclusion {
            dataset: pool.dataset(subtree, leaf),
            mount: exclusion.mount,
        });
    }
    if with_scratch {
        exclusions.push(Exclusion {
            dataset: pool.dataset(subtree, SCRATCH),
            mount: ExclusionMount::Path,
        });
    }
    return CoverageDecision {
        recursive_root: pool.dataset(subtree, "ROOT/cos/home"),
        template: String::from("home"),
        exclusions,
    };
}

/// One subtree's datasets and markers.
fn build_subtree(
    pool: &Pool,
    subtree: &str,
    with_scratch: bool,
) -> CoverageDecision {
    let root: String = format!("{}/{subtree}", pool.fixture);
    fs::create_dir_all(root.as_str()).expect("the subtree folder");
    let decision: CoverageDecision = decision_for(pool, subtree, with_scratch);
    create(
        pool,
        &pool.dataset(subtree, "ROOT/cos/home"),
        &[format!("mountpoint={root}/home")],
    );
    create(
        pool,
        &pool.dataset(subtree, "ROOT/cos/home/worktrees"),
        &[format!("mountpoint={root}/home/worktrees")],
    );
    if with_scratch {
        create(
            pool,
            &pool.dataset(subtree, SCRATCH),
            &[format!("mountpoint={root}/home/scratch")],
        );
    }
    let leaves: [&str; 9] = [
        "ROOT/cos/varcache",
        "nosnap",
        "nosnap/cargo-build",
        "nosnap/cargo-registry",
        "nosnap/npm",
        "nosnap/user-cache",
        "nosnap/vm-builder-output",
        "nosnap/libvirt-images",
        "nosnap/git-worktree-probe",
    ];
    for leaf in leaves {
        create(
            pool,
            &pool.dataset(subtree, leaf),
            &mount_properties(root.as_str(), leaf),
        );
    }
    for exclusion in &decision.exclusions {
        let marker: String = format!("{AUTO_SNAPSHOT_PROPERTY}={AUTO_SNAPSHOT_OFF}");
        zfs(&["set", marker.as_str(), exclusion.dataset.as_str()]);
    }
    return decision;
}

/// Every subtree of the layout,
/// in the order the arms use them.
fn build_layout(pool: &Pool) {
    for &(subtree, with_scratch) in SUBTREES {
        build_subtree(pool, subtree, with_scratch);
    }
}

//endregion Layout

//region Reading

/// The measured tree of one subtree,
/// from the same two requests the wiring step reads.
fn gather_tree(
    pool: &Pool,
    subtree: &str,
) -> MeasuredTree {
    let scope: String = format!("{}/{subtree}", pool.name);
    let listing: String = zfs(&["list", "-H", "-r", "-o", "name,mountpoint", scope.as_str()]);
    let markers: String = zfs(&[
        "get",
        "-H",
        "-r",
        "-o",
        "name,value",
        AUTO_SNAPSHOT_PROPERTY,
        scope.as_str(),
    ]);
    let mut datasets: Vec<MeasuredDataset> = Vec::new();
    for line in listing.lines() {
        let Some((name, mountpoint)) = line.split_once('\t') else {
            panic!("a dataset line is name and mount point: {line}");
        };
        let mut marker: Option<String> = None;
        for marker_line in markers.lines() {
            let Some((marker_name, value)) = marker_line.split_once('\t') else {
                panic!("a property line is name and value: {marker_line}");
            };
            if marker_name == name && value != "-" {
                marker = Some(String::from(value));
            }
        }
        datasets.push(MeasuredDataset {
            name: String::from(name),
            mountpoint: Some(String::from(mountpoint)).filter(|written| return written != "-"),
            auto_snapshot: marker,
        });
    }
    return MeasuredTree {
        datasets,
    };
}

/// The facts of one subtree with one configuration text.
fn facts(
    pool: &Pool,
    subtree: &str,
    decision: &CoverageDecision,
    text: &str,
    name: &str,
) -> CoverageFacts {
    let root: String = format!("{}/{subtree}", pool.fixture);
    let directory: String = format!("{root}/{name}");
    fs::create_dir_all(directory.as_str()).expect("the configuration folder");
    let path: String = format!("{directory}/sanoid.conf");
    fs::write(path.as_str(), text).expect("the configuration");
    return CoverageFacts {
        decision: decision.clone(),
        tree: gather_tree(pool, subtree),
        config_path: path,
        config: read_sanoid_conf(text),
        store_path: format!("{root}/home/.local/share/pnpm/store"),
    };
}

/// The marker a green arm prints.
fn claim(line: &str) {
    println!("SANOID-GUEST {line}");
}

/// Every field one answer found.
fn fields(answer: &CoverageAnswer) -> Vec<&str> {
    return answer
        .findings
        .iter()
        .map(|entry| return entry.field.as_str())
        .collect();
}

/// One finding by its field.
fn finding_of<'a>(
    answer: &'a CoverageAnswer,
    field: &str,
) -> &'a CoverageFinding {
    return answer
        .findings
        .iter()
        .find(|entry| return entry.field == field)
        .unwrap_or_else(|| panic!("no {field} finding in {:?}", fields(answer)));
}

/// Run one printed correction,
/// which the check promises is a command a person can run.
fn apply_correction(correction: &str) {
    let words: Vec<&str> = correction.split_whitespace().collect();
    assert_eq!(
        words.first(),
        Some(&"zfs"),
        "this arm applies a zfs set correction: {correction}"
    );
    run(words[0], &words[1..]);
}

//endregion Reading

//region Sanoid runs

/// Run one Sanoid release over one configuration in its own configuration,
/// cache and run folders.
fn run_sanoid(
    program: &str,
    defaults: &str,
    root: &str,
    name: &str,
    text: &str,
    phases: &[&str],
) {
    let directory: String = format!("{root}/{name}");
    fs::create_dir_all(directory.as_str()).expect("the sanoid folder");
    fs::write(format!("{directory}/sanoid.conf"), text).expect("the sanoid configuration");
    fs::copy(defaults, format!("{directory}/sanoid.defaults.conf")).expect("the sanoid defaults");
    let cache: String = format!("{directory}/cache");
    let run_directory: String = format!("{directory}/run");
    fs::create_dir_all(cache.as_str()).expect("the cache folder");
    fs::create_dir_all(run_directory.as_str()).expect("the run folder");
    let mut arguments: Vec<String> = vec![
        String::from("--configdir"),
        directory,
        String::from("--cache-dir"),
        cache,
        String::from("--run-dir"),
        run_directory,
        // The fixture folder outlives one run of the arm, so a cached snapshot list would describe a pool that no
        // longer exists and sanoid would take no snapshot at all.
        String::from("--force-update"),
        String::from("--verbose"),
    ];
    for phase in phases {
        arguments.push(String::from(*phase));
    }
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let mut all: Vec<&str> = vec![program];
    all.extend(borrowed);
    let said: String = run("perl", all.as_slice());
    for line in said.lines() {
        if line.starts_with("ERROR") || line.starts_with("WARN") {
            println!("SANOID-GUEST sanoid said {line}");
        }
    }
}

/// Every snapshot of one subtree.
fn snapshots(
    pool: &Pool,
    subtree: &str,
) -> Vec<String> {
    let scope: String = format!("{}/{subtree}", pool.name);
    let listing: String = zfs(&[
        "list",
        "-H",
        "-t",
        "snapshot",
        "-o",
        "name",
        "-r",
        scope.as_str(),
    ]);
    return listing.lines().map(String::from).collect();
}

/// Whether one dataset holds a snapshot Sanoid took.
fn has_autosnap(
    snapshots: &[String],
    dataset: &str,
) -> bool {
    let prefix: String = format!("{dataset}@");
    return snapshots.iter().any(|name| {
        return name
            .strip_prefix(prefix.as_str())
            .is_some_and(|rest| return rest.starts_with(AUTOSNAP));
    });
}

/// Whether one dataset holds an hourly snapshot Sanoid took,
/// which is the one period a zero hourly retention reaches.
fn has_hourly(
    snapshots: &[String],
    dataset: &str,
) -> bool {
    let prefix: String = format!("{dataset}@");
    return snapshots.iter().any(|name| {
        let Some(rest) = name.strip_prefix(prefix.as_str()) else {
            return false;
        };
        return rest.starts_with(AUTOSNAP) && rest.ends_with("_hourly");
    });
}

/// The decided text with the child dataset's section written before the
/// ancestor's,
/// which is the order Sanoid reads the exclusion out of effect.
fn misordered_text(decision: &CoverageDecision) -> String {
    let mut sections = sections_of(decision);
    let moved = sections.pop().expect("the child exclusion's section");
    let mut text: String = String::from(GENERATED_HEADER);
    text.push('\n');
    text.push_str(render_section(&moved).as_str());
    text.push('\n');
    text.push_str(render_sections(sections.as_slice()).as_str());
    return text;
}

//endregion Sanoid runs

//region The arms

/// The coverage comparison passes on the layout every marker is set on.
fn coverage_passes(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "decided", false);
    let text: String = generated_text(&decision);
    let answer: CoverageAnswer = check_coverage(&facts(
        pool,
        "decided",
        &decision,
        text.as_str(),
        "decided-config",
    ));
    assert!(
        matches!(answer.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        answer.outcome
    );
    assert!(answer.findings.is_empty(), "{:?}", fields(&answer));
    let tree: MeasuredTree = gather_tree(pool, "decided");
    assert!(
        tree.dataset(decision.recursive_root.as_str()).is_some(),
        "the ancestor is in the tree"
    );
    assert!(
        tree.dataset(pool.dataset("decided", "ROOT/cos/home/worktrees").as_str())
            .is_some(),
        "the covered child is in the tree"
    );
    for exclusion in &decision.exclusions {
        assert!(
            tree.dataset(exclusion.dataset.as_str()).is_some(),
            "{} is in the tree",
            exclusion.dataset
        );
    }
    assert_eq!(
        tree.datasets_under(decision.recursive_root.as_str()).len(),
        1,
        "only the covered child stands under the ancestor"
    );
    claim("coverage-passes");
}

/// Sanoid 2.3.0 snapshots the ancestor and its covered child,
/// and none of the nine exclusions.
fn measured_covers_home_only(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "decided", false);
    let root: String = format!("{}/decided", pool.fixture);
    run_sanoid(
        SANOID_MEASURED,
        DEFAULTS_MEASURED,
        root.as_str(),
        "measured-run",
        generated_text(&decision).as_str(),
        &["--take-snapshots"],
    );
    let taken: Vec<String> = snapshots(pool, "decided");
    assert!(
        has_autosnap(&taken, decision.recursive_root.as_str()),
        "the ancestor is covered: {taken:?}"
    );
    assert!(
        has_autosnap(
            &taken,
            pool.dataset("decided", "ROOT/cos/home/worktrees").as_str()
        ),
        "the child is covered: {taken:?}"
    );
    for exclusion in &decision.exclusions {
        assert!(
            !has_autosnap(&taken, exclusion.dataset.as_str()),
            "{} was snapshotted: {taken:?}",
            exclusion.dataset
        );
    }
    claim("2-3-0-covers-home-only");
}

/// Sanoid 2.3.0 honours a child's exclusion section after its ancestor's,
/// and snapshots the same child when the section stands before it.
fn measured_order_captures_the_exclusion(pool: &Pool) {
    let ordered: CoverageDecision = decision_for(pool, "order-ok", true);
    let ordered_root: String = format!("{}/order-ok", pool.fixture);
    run_sanoid(
        SANOID_MEASURED,
        DEFAULTS_MEASURED,
        ordered_root.as_str(),
        "ordered-run",
        generated_text(&ordered).as_str(),
        &["--take-snapshots"],
    );
    let scratch: String = pool.dataset("order-ok", SCRATCH);
    assert!(
        !has_autosnap(&snapshots(pool, "order-ok"), scratch.as_str()),
        "the exclusion written after its ancestor is honoured"
    );
    let misordered: CoverageDecision = decision_for(pool, "order-bad", true);
    let text: String = misordered_text(&misordered);
    let answer: CoverageAnswer = check_coverage(&facts(
        pool,
        "order-bad",
        &misordered,
        text.as_str(),
        "misordered-config",
    ));
    assert!(
        fields(&answer).contains(&FIELD_SECTION_ORDER),
        "the check names the order: {:?}",
        fields(&answer)
    );
    let misordered_root: String = format!("{}/order-bad", pool.fixture);
    run_sanoid(
        SANOID_MEASURED,
        DEFAULTS_MEASURED,
        misordered_root.as_str(),
        "misordered-run",
        text.as_str(),
        &["--take-snapshots"],
    );
    let captured: String = pool.dataset("order-bad", SCRATCH);
    assert!(
        has_autosnap(&snapshots(pool, "order-bad"), captured.as_str()),
        "the exclusion written before its ancestor is snapshotted"
    );
    claim("2-3-0-order-captures-the-exclusion");
}

/// Sanoid 2.3.0 with `recursive = zfs` takes one recursive snapshot no
/// exclusion can be left out of.
fn measured_zfs_recursion_captures_the_exclusion(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "zfs", true);
    let text: String = generated_text(&decision).replace("recursive = yes", "recursive = zfs");
    let answer: CoverageAnswer =
        check_coverage(&facts(pool, "zfs", &decision, text.as_str(), "zfs-config"));
    assert!(
        fields(&answer).contains(&FIELD_RECURSION),
        "the check names the recursion: {:?}",
        fields(&answer)
    );
    let root: String = format!("{}/zfs", pool.fixture);
    run_sanoid(
        SANOID_MEASURED,
        DEFAULTS_MEASURED,
        root.as_str(),
        "zfs-run",
        text.as_str(),
        &["--take-snapshots"],
    );
    let scratch: String = pool.dataset("zfs", SCRATCH);
    assert!(
        has_autosnap(&snapshots(pool, "zfs"), scratch.as_str()),
        "one recursive snapshot includes the exclusion"
    );
    claim("2-3-0-zfs-recursion-captures-the-exclusion");
}

/// The Ubuntu package's Sanoid 2.2.0 gives the same coverage result.
fn control_covers_home_only(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "control-ok", true);
    let root: String = format!("{}/control-ok", pool.fixture);
    run_sanoid(
        SANOID_CONTROL,
        DEFAULTS_CONTROL,
        root.as_str(),
        "ordered-run",
        generated_text(&decision).as_str(),
        &["--take-snapshots"],
    );
    let taken: Vec<String> = snapshots(pool, "control-ok");
    assert!(
        has_autosnap(&taken, decision.recursive_root.as_str()),
        "the control release covers the ancestor: {taken:?}"
    );
    let scratch: String = pool.dataset("control-ok", SCRATCH);
    assert!(
        !has_autosnap(&taken, scratch.as_str()),
        "the control release honours the exclusion: {taken:?}"
    );
    claim("2-2-0-covers-home-only");
}

/// The Ubuntu package's Sanoid 2.2.0 reproduces the order effect.
fn control_order_captures_the_exclusion(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "control-bad", true);
    let text: String = misordered_text(&decision);
    let root: String = format!("{}/control-bad", pool.fixture);
    run_sanoid(
        SANOID_CONTROL,
        DEFAULTS_CONTROL,
        root.as_str(),
        "misordered-run",
        text.as_str(),
        &["--take-snapshots"],
    );
    let scratch: String = pool.dataset("control-bad", SCRATCH);
    assert!(
        has_autosnap(&snapshots(pool, "control-bad"), scratch.as_str()),
        "the control release snapshots the misordered exclusion"
    );
    claim("2-2-0-order-captures-the-exclusion");
}

/// A zero hourly retention prunes every snapshot Sanoid named,
/// and keeps the pinned origin a living clone holds.
fn prune_keeps_the_pinned_origin(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "decided", false);
    let home: String = decision.recursive_root.clone();
    let worktrees: String = pool.dataset("decided", "ROOT/cos/home/worktrees");
    zfs(&[
        "snapshot",
        format!("{home}@{ORIGIN}{ORIGIN_NONCE}").as_str(),
    ]);
    zfs(&["snapshot", format!("{home}@manual-hold").as_str()]);
    zfs(&[
        "snapshot",
        format!("{worktrees}@{ORIGIN}{ORIGIN_NONCE}").as_str(),
    ]);
    let before: Vec<String> = snapshots(pool, "decided");
    assert!(
        has_hourly(&before, home.as_str()),
        "the prune has an hourlie to find: {before:?}"
    );
    let text: String = format!(
        "{}\n[template_home]\n  autosnap = yes\n  autoprune = yes\n  hourly = 0\n",
        generated_text(&decision)
    );
    let root: String = format!("{}/decided", pool.fixture);
    run_sanoid(
        SANOID_MEASURED,
        DEFAULTS_MEASURED,
        root.as_str(),
        "prune-run",
        text.as_str(),
        &["--prune-snapshots"],
    );
    let left: Vec<String> = snapshots(pool, "decided");
    assert!(
        !has_hourly(&left, home.as_str()),
        "the hourlies are pruned: {left:?}"
    );
    assert!(
        !has_hourly(&left, worktrees.as_str()),
        "the child's hourlies are pruned: {left:?}"
    );
    assert!(
        has_autosnap(&left, home.as_str()),
        "the daily and monthly snapshots outlive an hourly expiry: {left:?}"
    );
    assert!(
        left.contains(&format!("{home}@{ORIGIN}{ORIGIN_NONCE}")),
        "the pinned origin stays: {left:?}"
    );
    assert!(
        left.contains(&format!("{home}@manual-hold")),
        "a snapshot Sanoid did not name stays: {left:?}"
    );
    assert!(
        left.contains(&format!("{worktrees}@{ORIGIN}{ORIGIN_NONCE}")),
        "the child's pinned origin stays: {left:?}"
    );
    claim("prune-keeps-the-pinned-origin");
}

/// A lifted marker is a finding whose printed `zfs set` puts it back.
fn marker_finding_and_correction(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "decided", false);
    let text: String = generated_text(&decision);
    let npm: String = decision
        .exclusions
        .iter()
        .find(|exclusion| return exclusion.dataset.ends_with("/nosnap/npm"))
        .expect("the npm exclusion")
        .dataset
        .clone();
    zfs(&[
        "set",
        format!("{AUTO_SNAPSHOT_PROPERTY}=true").as_str(),
        npm.as_str(),
    ]);
    let broken: CoverageAnswer = check_coverage(&facts(
        pool,
        "decided",
        &decision,
        text.as_str(),
        "marker-config",
    ));
    let correction: String = finding_of(&broken, FIELD_EXCLUSION_MARKER)
        .correction
        .clone()
        .expect("the marker carries a correction");
    assert_eq!(
        correction,
        format!("zfs set {AUTO_SNAPSHOT_PROPERTY}={AUTO_SNAPSHOT_OFF} {npm}")
    );
    apply_correction(correction.as_str());
    let healed: CoverageAnswer = check_coverage(&facts(
        pool,
        "decided",
        &decision,
        text.as_str(),
        "marker-config",
    ));
    assert!(
        matches!(healed.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        healed.outcome
    );
    claim("marker-finding-and-correction-apply");
}

/// A moved exclusion section is a finding whose correction the decided order
/// applies.
fn order_finding_and_correction(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "order-bad", true);
    let text: String = misordered_text(&decision);
    let answer: CoverageAnswer = check_coverage(&facts(
        pool,
        "order-bad",
        &decision,
        text.as_str(),
        "order-config",
    ));
    let scratch: String = pool.dataset("order-bad", SCRATCH);
    let correction: String = finding_of(&answer, FIELD_SECTION_ORDER)
        .correction
        .clone()
        .expect("the order carries a correction");
    assert!(
        correction.contains(&format!(
            "move the [{scratch}] section below the [{}] section",
            decision.recursive_root
        )),
        "{correction}"
    );
    let healed: CoverageAnswer = check_coverage(&facts(
        pool,
        "order-bad",
        &decision,
        generated_text(&decision).as_str(),
        "order-config",
    ));
    assert!(
        matches!(healed.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        healed.outcome
    );
    claim("order-finding-and-correction-apply");
}

/// A `recursive = zfs` ancestor is a finding whose correction the decided text
/// applies.
fn recursion_finding_and_correction(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "zfs", true);
    let text: String = generated_text(&decision).replace("recursive = yes", "recursive = zfs");
    let answer: CoverageAnswer = check_coverage(&facts(
        pool,
        "zfs",
        &decision,
        text.as_str(),
        "recursion-config",
    ));
    let correction: String = finding_of(&answer, FIELD_RECURSION)
        .correction
        .clone()
        .expect("the recursion carries a correction");
    assert_eq!(
        correction,
        format!(
            "in the [{}] section of {}/zfs/recursion-config/sanoid.conf, change `recursive = zfs` to `recursive = yes`",
            decision.recursive_root, pool.fixture
        )
    );
    let healed: CoverageAnswer = check_coverage(&facts(
        pool,
        "zfs",
        &decision,
        text.replace("recursive = zfs", "recursive = yes").as_str(),
        "recursion-config",
    ));
    assert!(
        matches!(healed.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        healed.outcome
    );
    claim("recursion-finding-and-correction-apply");
}

/// A store under an excluded dataset is a finding no configuration line ends.
fn store_finding(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "decided", false);
    let text: String = generated_text(&decision);
    let mut facts_of_the_store: CoverageFacts =
        facts(pool, "decided", &decision, text.as_str(), "store-config");
    facts_of_the_store.store_path = format!("{}/decided/home/.npm/pnpm-store", pool.fixture);
    let answer: CoverageAnswer = check_coverage(&facts_of_the_store);
    let finding: &CoverageFinding = finding_of(&answer, FIELD_STORE_COVERAGE);
    assert_eq!(finding.correction, None);
    assert!(finding.detail.contains("nosnap/npm"), "{}", finding.detail);
    claim("store-finding-and-nothing-reaches-it");
}

/// The guard's report names what it cannot prune,
/// and a guard that descends is a finding.
fn guard_scope(pool: &Pool) {
    let decision: CoverageDecision = decision_for(pool, "decided", false);
    let guard: SpaceGuardFacts = SpaceGuardFacts {
        configured_dataset: decision.recursive_root.clone(),
        pruned_prefix: String::from(AUTOSNAP),
        recursive: false,
        decision: decision.clone(),
        tree: gather_tree(pool, "decided"),
    };
    let answer: ScopeAnswer = check_space_guard_scope(&guard);
    assert!(
        matches!(answer.outcome, CheckOutcome::Pass { .. }),
        "{:?}",
        answer.outcome
    );
    assert!(
        answer
            .lines
            .iter()
            .any(|line| return line.contains("ROOT/cos/home/worktrees")),
        "{:?}",
        answer.lines
    );
    let expanded: SpaceGuardFacts = SpaceGuardFacts {
        configured_dataset: decision.recursive_root.clone(),
        pruned_prefix: String::from(AUTOSNAP),
        recursive: true,
        decision,
        tree: gather_tree(pool, "decided"),
    };
    assert!(
        matches!(
            check_space_guard_scope(&expanded).outcome,
            CheckOutcome::Finding { .. }
        ),
        "a descending guard is a finding"
    );
    claim("guard-scope-names-what-it-cannot-prune");
}

//endregion The arms

#[test]
fn zfs_sanoid_coverage_guest() {
    let Ok(guest) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP {GUEST_VARIABLE} names no guest");
        return;
    };
    if guest.is_empty() {
        println!("SKIP {GUEST_VARIABLE} is empty");
        return;
    }
    let pool: Pool = Pool::create();
    println!("SANOID-GUEST pool {}", pool.name);
    println!(
        "SANOID-GUEST measured sanoid {}",
        version_of(SANOID_MEASURED).trim()
    );
    println!(
        "SANOID-GUEST control sanoid {}",
        version_of(SANOID_CONTROL).trim()
    );
    build_layout(&pool);
    claim("layout-built");
    coverage_passes(&pool);
    measured_covers_home_only(&pool);
    measured_order_captures_the_exclusion(&pool);
    measured_zfs_recursion_captures_the_exclusion(&pool);
    control_covers_home_only(&pool);
    control_order_captures_the_exclusion(&pool);
    prune_keeps_the_pinned_origin(&pool);
    marker_finding_and_correction(&pool);
    order_finding_and_correction(&pool);
    recursion_finding_and_correction(&pool);
    store_finding(&pool);
    guard_scope(&pool);
}

/// The version one Sanoid prints,
/// which the arm records so a moved package fails loudly.
fn version_of(program: &str) -> String {
    return run("perl", &[program, "--version"]);
}
