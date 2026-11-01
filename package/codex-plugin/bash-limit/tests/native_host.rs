//! Opt-in verification against an installed Codex host with a local deterministic provider.

/// What:
/// `mod` includes sibling source modules,
/// and `#[path]` chooses their locations.
/// Why:
/// Keep host transport and response fixture details separate from the test assertions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as rpc from './host_support/rpc';
/// ```
#[path = "host_support/rpc.rs"]
mod rpc;
/// Include the bounded local Responses fixture.
#[path = "host_support/server.rs"]
mod server;
/// Include shared native binary and disposable-control helpers.
mod support;
/// Import filesystem,
/// I/O,
/// process,
/// networking,
/// and thread primitives.
use std::{fs, io::{self, BufReader}, net::TcpListener, process::{Command, Stdio}, thread};
/// Import owned JSON values for protocol fields,
/// avoiding panic-generating macros.
use serde_json::Value;
/// Import the checked parser for fixed protocol templates.
use rpc::parse;

/// What:
/// `#[test]` registers a nextest case;
/// `#[ignore]` makes host availability opt-in.
/// `Result<()>` permits setup and transport failures to fail the test with their causes.
/// Why:
/// Verify actual hook discovery,
/// trust,
/// command rewriting,
/// and host-reported latency.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test.skip('installed native host', () => { ... });
/// ```
#[test]
#[ignore = "requires installed Codex and local loopback sockets; run the test:host task"]
fn installed_host_rewrites_commands() -> io::Result<()> {
    // Prevent a host-launched process from recursively starting another host fixture.
    assert!(std::env::var_os("CODEX_BASH_LIMIT_HOST_FIXTURE").is_none());
    // What: `?` propagates setup failures; paths own their data while references borrow it.
    // Why: Everything written by the host belongs to this independent fixture.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parent = fixture();
    // ```
    let parent = support::fixture()?;
    // Borrow UTF-8 path text; reject lossy fixture paths explicitly.
    let path = parent.to_str().ok_or_else(|| { return io::Error::other("fixture path encoding"); })?;
    // Positive preflight also confirms this test uses the same release artifact as unit fixtures.
    let preflight = support::invoke(&["--hook", path], &support::event("true")?)?;
    assert!(preflight.status.success(), "{preflight:?}");
    let home = parent.join("host-home");
    // Borrow the path so later configuration and host startup can reuse it.
    fs::create_dir(&home)?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    // format! interpolates a numeric port into fixed TOML text; no user text crosses this boundary.
    let config = format!("model = 'fixture'\nmodel_provider = 'local-fixture'\napproval_policy = 'never'\nsandbox_mode = 'workspace-write'\n[model_providers.local-fixture]\nname = 'Local deterministic fixture'\nbase_url = 'http://127.0.0.1:{port}/v1'\nwire_api = 'responses'\n[features]\nhooks = true\n");
    fs::write(home.join("config.toml"), config)?;
    // Shell-quote both filenames at the command boundary; the installed host reads command only.
    let command = format!("'{}' --hook '{}'", env!("CARGO_BIN_EXE_codex-bash-limit").replace('\'', "'\\''"), path.replace('\'', "'\\''"));
    // Decode fixed JSON syntax, then assign the command as data rather than JSON source.
    let mut hooks = parse(r#"{"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{
        "type":"command","command":""
    }]}]}}"#)?;
    hooks["hooks"]["PreToolUse"][0]["hooks"][0]["command"] = Value::from(command);
    fs::write(home.join("hooks.json"), hooks.to_string())?;
    let stderr = fs::File::create(parent.join("host-stderr.log"))?;
    // Some gives an environment override; Command owns the launched process until cleanup.
    let mut child = Command::new("codex").args(["--no-daemon", "app-server", "--listen", "stdio://"])
        .env("CODEX_BASH_LIMIT_HOST_FIXTURE", "1")
        .env("CODEX_HOME", &home).current_dir(&parent)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(stderr).spawn()?;
    // take moves each pipe out of the child handle, leaving an absent Option behind.
    let mut pipe = child.stdin.take().ok_or_else(|| { return io::Error::other("host stdin missing"); })?;
    let stdout = child.stdout.take().ok_or_else(|| { return io::Error::other("host stdout missing"); })?;
    let mut reader = BufReader::new(stdout);
    // Lend exclusive access to both pipes for each sequential RPC request.
    rpc::call(&mut pipe, &mut reader, 1, "initialize", parse(r#"{"clientInfo":{"name":"bash-limit-fixture","version":"1"},"capabilities":{"experimentalApi":true}}"#)?)?;
    rpc::send(&mut pipe, parse(r#"{"method":"initialized"}"#)?)?;
    let mut listing = parse(r#"{"cwds":[""]}"#)?;
    listing["cwds"][0] = Value::from(path);
    let listed = rpc::call(&mut pipe, &mut reader, 2, "hooks/list", listing)?;
    let hook = &listed["data"][0]["hooks"][0];
    assert!(hook["key"].is_string(), "{listed}");
    let key = hook["key"].as_str().ok_or_else(|| { return io::Error::other("hook key missing"); })?;
    // Map owns a variable set of JSON object keys; clone preserves the discovered hash exactly.
    let mut states = serde_json::Map::new();
    let mut state = parse(r#"{"trusted_hash":""}"#)?;
    state["trusted_hash"] = hook["currentHash"].clone();
    states.insert(key.to_owned(), state);
    let mut trust = parse(r#"{"keyPath":"hooks.state","value":null,"mergeStrategy":"upsert"}"#)?;
    trust["value"] = Value::Object(states);
    rpc::call(&mut pipe, &mut reader, 3, "config/value/write", trust)?;
    let mut thread_params = parse(r#"{"cwd":"","ephemeral":true,"baseInstructions":"Execute deterministic fixture tool calls."}"#)?;
    thread_params["cwd"] = Value::from(path);
    let started = rpc::call(&mut pipe, &mut reader, 4, "thread/start", thread_params)?;
    let thread_id = &started["thread"]["id"];
    assert!(thread_id.is_string(), "{started}");
    // move transfers listener ownership into a worker thread; it serves a bounded local sequence.
    let provider = thread::spawn(move || { return server::serve(listener, 40); });
    let mut turn = parse(r#"{"threadId":"","input":[{"type":"text","text":"Run the native hook fixture."}]}"#)?;
    turn["threadId"] = thread_id.clone();
    rpc::call(&mut pipe, &mut reader, 5, "turn/start", turn)?;
    // Vec owns a growing list; a borrowed slice would not permit collecting new durations.
    let mut durations = Vec::new();
    loop {
        let message = rpc::receive(&mut reader)?;
        if message["method"] == "hook/completed" {
            let run = &message["params"]["run"];
            assert_eq!(run["status"], "completed", "{run}");
            durations.push(run["durationMs"].as_u64().ok_or_else(|| { return io::Error::other("hook duration missing"); })?);
        }
        if message["method"] == "turn/completed" {
            assert_eq!(message["params"]["turn"]["status"], "completed", "{message}");
            break;
        }
    }
    // Drop closes the owned stdin pipe, asking the app-server to finish normally.
    drop(pipe);
    assert!(child.wait()?.success());
    // join returns the thread result; convert a worker panic and propagate its I/O failures.
    provider.join().map_err(|_panic| { return io::Error::other("local provider thread panicked"); })??;
    assert_eq!(durations.len(), 40);
    durations.sort_unstable();
    // Print measured evidence for the opt-in task's retained output.
    println!("native host hook milliseconds: {durations:?}");
    // Read the rewritten command's observable membership write, independent of hook events.
    let member = fs::read_to_string(parent.join("codex/cgroup.procs"))?;
    assert!(member.parse::<u32>().is_ok(), "{member}");
    fs::remove_dir_all(parent)?;
    // Ok denotes a successfully verified host interaction.
    return Ok(());
}
