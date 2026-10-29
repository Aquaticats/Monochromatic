//! Real child processes prove cancellation/reaping and configuration isolation without shell scripts.

/// Test the production command construction and owned-child lifetime boundary.
use super::{Stream, command, consume, execute, stop};
/// One retained cancellation flag wakes collection even when a pipe is silent.
use crate::search_cancel::SearchCancellation;
/// Tests create only disposable files and owned process pipes.
use std::{fs, process::Stdio, time::Duration};
/// Tokio supplies actual asynchronous child process handles and timer deadlines.
use tokio::process::Command;

/// Construct a silent child that would otherwise outlive the regression test.
fn sleeper() -> Command {
    let mut command = Command::new("sleep");
    command
        .arg("60")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    return command;
}

/// Cancellation must kill and reap an already spawned child without waiting for stdout or stderr bytes.
#[tokio::test]
async fn cancellation_interrupts_silent_pipes_and_reaps_the_child() {
    let fixture = tempfile::tempdir().expect("disposable search root");
    let cancellation = SearchCancellation::new();
    let signal = cancellation.subscribe();
    let child = sleeper().spawn().expect("silent child");
    let pid = child.id().expect("live child pid");
    let proc_path = format!("/proc/{pid}");
    assert!(
        std::path::Path::new(&proc_path).exists(),
        "positive process-lifetime control"
    );
    let result = tokio::time::timeout(Duration::from_secs(3), async {
        let (output, ()) = tokio::join!(
            consume(
                child,
                fixture.path(),
                "silent",
                Stream::Paths,
                &cancellation,
                signal
            ),
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                cancellation.cancel();
            }
        );
        return output;
    })
    .await
    .expect("cancelled search must not wait for child EOF")
    .expect("cancelled stream cleanup");
    assert!(result.is_none());
    assert!(
        !std::path::Path::new(&proc_path).exists(),
        "owned child was not reaped"
    );
}

/// EOF must not turn the final process wait into an uncancellable join.
#[tokio::test]
async fn cancellation_after_stdout_eof_still_reaps_the_running_child() {
    let fixture = tempfile::tempdir().expect("disposable search root");
    let cancellation = SearchCancellation::new();
    let signal = cancellation.subscribe();
    let mut closed_output = Command::new("true")
        .stdout(Stdio::piped())
        .spawn()
        .expect("EOF pipe owner");
    closed_output.wait().await.expect("close the pipe writer");
    let mut child = sleeper().spawn().expect("running child after stdout EOF");
    let pid = child.id().expect("running pid");
    // An already closed writer exercises EOF without unsafe descriptor mutation or a shell script.
    child.stdout = closed_output.stdout.take();
    let output = tokio::time::timeout(Duration::from_secs(3), async {
        let (result, ()) = tokio::join!(
            consume(
                child,
                fixture.path(),
                "eof",
                Stream::Paths,
                &cancellation,
                signal
            ),
            async {
                tokio::time::sleep(Duration::from_millis(20)).await;
                assert!(
                    std::path::Path::new(&format!("/proc/{pid}")).exists(),
                    "child exited before the cancellation control"
                );
                cancellation.cancel();
            }
        );
        return result;
    })
    .await
    .expect("EOF must retain cancellation during process wait")
    .expect("process cleanup");
    assert!(output.is_none());
    assert!(
        !std::path::Path::new(&format!("/proc/{pid}")).exists(),
        "EOF child was not reaped"
    );
}

/// A signal cancelled before startup prevents even an invalid executable from being spawned.
#[tokio::test]
async fn pre_cancelled_search_skips_process_start_and_live_spawn_errors_are_visible() {
    let fixture = tempfile::tempdir().expect("disposable root");
    let cancelled = SearchCancellation::default();
    cancelled.cancel();
    assert!(
        execute(
            Command::new("ide-nonexistent-search-executable"),
            fixture.path(),
            "query",
            Stream::Paths,
            &cancelled
        )
        .await
        .expect("pre-cancelled outcome")
        .is_none()
    );
    assert!(
        execute(
            Command::new("ide-nonexistent-search-executable"),
            fixture.path(),
            "query",
            Stream::Paths,
            &SearchCancellation::new()
        )
        .await
        .is_err()
    );
}

/// A completed process needs no further signal,
///  and its already reaped status remains available.
#[tokio::test]
async fn stopping_an_already_completed_child_keeps_its_status() {
    let mut child = Command::new("true").spawn().expect("immediate child");
    let finished = child.wait().await.expect("reap completed child");
    assert_eq!(stop(&mut child).await.expect("completed cleanup"), finished);
}

/// Inherited preprocessing configuration is a real positive control,
///  then the production flags disable it.
#[tokio::test]
async fn production_command_ignores_external_ripgrep_configuration() {
    let fixture = tempfile::tempdir().expect("disposable parent");
    let root = fixture.path().join("project");
    fs::create_dir(&root).expect("search root");
    fs::write(root.join("source.txt"), "needle\n").expect("source fixture");
    let config = fixture.path().join("ripgrep.conf");
    fs::write(&config, "--pre\n/usr/bin/false\n").expect("external config fixture");
    let mut affected = Command::new("rg");
    affected
        .args(["--json", "--", "needle"])
        .arg(&root)
        .env("RIPGREP_CONFIG_PATH", &config);
    affected
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let control = execute(
        affected,
        &root,
        "needle",
        Stream::Contents,
        &SearchCancellation::new(),
    )
    .await;
    assert!(
        control.is_err(),
        "positive control did not honor preprocessing configuration"
    );
    let mut protected = command(&root, "needle", Stream::Contents);
    // Reintroduce the environment value to verify --no-config independently of env_remove.
    protected.env("RIPGREP_CONFIG_PATH", &config);
    let result = execute(
        protected,
        &root,
        "needle",
        Stream::Contents,
        &SearchCancellation::new(),
    )
    .await
    .expect("configuration-disabled search")
    .expect("uncancelled result");
    assert_eq!(result.len(), 1);
}
