//! Uncommitted signing-configuration audit probe using the production shadow creator and native commit runner.

/// The ordinary reference snapshot must accept Git-valid byte names.
#[test]
fn audit_non_utf8_tag_does_not_prevent_native_shadow_creation() {
    // Locate the native-control repository built by the separate generator.
    let root = std::path::PathBuf::from(std::env::var_os("SHADOW_AUDIT_ROOT").expect("fixture root"));
    // Own the manifest while borrowing its fields for capture.
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("nonutf-ref.json")).expect("manifest")).expect("JSON");
    // Own the repository path for the context and production creator.
    let repo = std::path::PathBuf::from(field(&manifest, "repo"));
    // Keep plain Git's native PATH limited to the disposable test.
    let mut context = crate::test_support::git_context();
    // Resolve the plain executable through the child overlay.
    context.real_git = std::path::PathBuf::from("git");
    // Add native PATH to the context rather than changing the agent environment.
    context.overlay.push(("PATH".into(), "/usr/bin:/bin".into()));
    // Git input files stay within this fixture.
    context.common_dir = Some(repo.join(".git"));
    // Production capture sees a normal main branch with an unrelated byte-named tag.
    let layout = crate::capture_invocation::capture_invocation_layout(&context, &repo, false).expect("layout");
    // Capture the base after the successful native commit.
    let capture = crate::capture_invocation::capture_invocation_base(&context, &repo, layout).expect("capture");
    // Keep the partial shadow for inspection after the deliberately red assertion.
    let scratch = std::path::PathBuf::from(field(&manifest, "root"));
    // Run the exact production creator, including its strict reference snapshot decoder.
    let made = crate::shadow_create::create_shadow_repository_at(&context, &repo, &capture, &scratch.join("shadow"), &scratch);
    // Native control's commit succeeded and its subject demonstrates which control ran.
    let native = git_at(&repo, &["log", "-1", "--format=%s"]);
    // Quote the production diagnostic rather than guessing which step failed.
    println!("non-UTF-8 tag: native subject={:?}, shadow creation={made:?}", String::from_utf8_lossy(&native.stdout));
    // A valid tag unrelated to the target must not prevent creating this commit's scratch repository.
    assert!(made.is_ok(), "valid non-UTF-8 reference should not block a commit");
}

/// Removing a partially built shadow must not follow its links into the real repository.
#[test]
fn audit_partial_shadow_cleanup_does_not_follow_shared_links() {
    // Every path belongs to a generated fixture in the current worktree.
    let root = std::path::PathBuf::from(std::env::var_os("SHADOW_AUDIT_ROOT").expect("fixture root"));
    // Use a separate scratch directory, not the configuration probes' shadows.
    let shadow = root.join("partial-shadow-cleanup");
    // The ASCII-control repository has a normal shared info directory.
    let common = root.join("ascii-control").join(".git");
    // Leave a known payload in the real shared directory to detect traversal through its shadow link.
    let sentinel = common.join("info").join("audit-sentinel");
    // Exclusive fixture state distinguishes a preserved payload from an empty directory.
    std::fs::write(&sentinel, b"must survive shadow cleanup\n").expect("sentinel");
    // The interrupted-creation seam is after skeleton/link creation, before config/ref completion.
    crate::shadow_skeleton::create_shadow_skeleton(&shadow).expect("partial skeleton");
    // Production links provide the actual recursive-deletion hazard, not a simulated mapping.
    crate::shadow_links::link_shadow_entries(&shadow, &common, &common).expect("shared links");
    // Prove the shadow really reads the real payload before deleting the shadow.
    assert_eq!(std::fs::read(shadow.join("info").join("audit-sentinel")).expect("linked sentinel"), b"must survive shadow cleanup\n");
    // Production removal deletes only the private tree and the link itself.
    crate::shadow_git::remove_shadow_repository(&shadow).expect("cleanup");
    // The direct shared path must retain the exact bytes.
    assert_eq!(std::fs::read(&sentinel).expect("real sentinel survived"), b"must survive shadow cleanup\n");
    // There is no completed shadow to confuse with the interrupted one.
    assert!(!shadow.exists());
    // This output names both the exercised path and its expected observation.
    println!("partial shadow: shared sentinel intact=true, shadow removed=true");
}

/// Borrow the parent probe's bounded real-Git invocation and manifest reader.
use super::{field, git_at};
/// Owned paths name disposable fixture locations beyond the manifest borrow.
use std::path::PathBuf;

/// What: create the production shadow and build an empty native commit using a real-gitdir signing include.
/// Why: detect a successful unsigned commit despite the real repository requiring a signature.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shadow = createShadow(capture(repo)); runNativeCommit(shadow); assert(readPrepared(shadow).signed);
/// ```
#[test]
fn audit_exact_gitdir_include_keeps_required_signing_enabled() {
    // The fixture generator has already proved native Git signed through this include.
    let root = PathBuf::from(std::env::var_os("SHADOW_AUDIT_ROOT").expect("fixture root"));
    // Decode the fixture locations without using anyone's real configuration.
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("signing-config.json")).expect("manifest")).expect("JSON");
    // Owned paths outlive all requests borrowing them.
    let repo = PathBuf::from(field(&manifest, "repo"));
    // The generator's signed native commit is a required positive control.
    let native = git_at(&repo, &["cat-file", "commit", "HEAD"]);
    // Inspect only signature presence, never print the disposable private key.
    let native_signed = String::from_utf8_lossy(&native.stdout).contains("\ngpgsig ");
    // The control is mandatory, not an informational line that could silently be false.
    assert!(native.status.success() && native_signed, "native positive control must sign");
    // Context owns the overlay; production still selects every command and environment adjustment.
    let mut context = crate::test_support::git_context();
    // Native PATH lets the plain executable name refer to Git without using an absolute bypass.
    context.real_git = PathBuf::from("git");
    // Resolve plain Git only inside the disposable fixture.
    context.overlay.push(("PATH".into(), "/usr/bin:/bin".into()));
    // Scratch input files stay inside the fixture.
    context.common_dir = Some(repo.join(".git"));
    // The ordinary production capture records repository layout and the required signing configuration.
    let layout = crate::capture_invocation::capture_invocation_layout(&context, &repo, false).expect("layout");
    // The base is the native signed control, exactly the state at the invocation seam.
    let capture = crate::capture_invocation::capture_invocation_base(&context, &repo, layout).expect("capture");
    // Shadow and native index have private names under the fixture root.
    let scratch = PathBuf::from(field(&manifest, "root"));
    // The production creator includes the real config then snapshots the real references.
    let shadow = scratch.join("shadow");
    // All filesystem steps and reference writes are the unmodified production implementation.
    crate::shadow_create::create_shadow_repository_at(&context, &repo, &capture, &shadow, &scratch).expect("shadow");
    // Create a private index that reads only the fixture's current tree.
    let index = scratch.join("private-index");
    // A request borrows the fixture's base and writes no real index.
    let mut request = crate::transaction_git::GitRequest::new(&repo, &["read-tree", "HEAD"]);
    // `Some` enables the optional private-index selection instead of changing the real fixture index.
    request.index = Some(index.clone());
    // Read the same tree the native signed control committed.
    crate::transaction_git::run_git_checked(&context, &request).expect("private index");
    // Owned command arguments survive until native Git finishes.
    let args = vec![String::from("--allow-empty"), String::from("--message=audit shadow signing")];
    // The production request borrows the capture and private paths, with no caller signing override.
    let preparation = crate::commit_preparation::PreparationRequest { capture: &capture, shadow: &shadow, commit_index: &index, global_args: &[], commit_args: &args, object_directory: None };
    // A shortened explicit ceiling prevents any signer or native commit from holding the probe indefinitely.
    let code = crate::commit_preparation::run_native_commit_within(&context, &repo, &preparation, std::time::Duration::from_secs(10)).expect("native commit returns");
    // The production reader reports the object Git placed at the private HEAD.
    let prepared = crate::commit_preparation::read_prepared_commit(&context, &shadow).expect("prepared commit");
    // Quote the observable disagreement without dumping config values or signing material.
    println!("exact gitdir include: native signed={native_signed}, shadow exit={code}, shadow signed={}", prepared.signed);
    // A clean unsigned exit violates the signing request rather than refusing it explicitly.
    assert_eq!(code, 0);
    // This is intentionally red against the audited implementation.
    assert!(prepared.signed, "the shadow must not silently turn off the person's configured signing");
}
