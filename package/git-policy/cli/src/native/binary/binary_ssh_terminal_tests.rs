//! What:
//!  A command a person types at a real terminal, run through the wrapper over `ssh://`,
//!  asks no one for the SSH key's passphrase:
//!  the key is protected by one,
//!  no agent holds it,
//!  and OpenSSH would otherwise read the question from the terminal itself.
//! Why:
//!  The owner decided no command the wrapper runs or forwards asks a person for a credential,
//!  an SSH key's passphrase counted as one
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  the entry beginning "owner answered H1 and M4").
//!  `SSH_ASKPASS_REQUIRE=never` alone met that without a terminal and not at one:
//!  the integration pass of 2026-11-19 measured the question through the wrapper at a pseudo-terminal
//!  (`doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`).
//!  The control reads the terminal before it reads the answer as absent:
//!  a pseudo-terminal that shows nothing proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(ptyRun('ssh ...').output).toContain('Enter passphrase for key');
//! // expect(ptyRun('git ls-remote ssh://...').output).not.toMatch(/passphrase/i);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, bounded_for, fixture, pseudo_terminal_slave_name, wrapper_first_path};
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// How long a command at the terminal may run before the bound ends it;
///  a question that is asked waits for ever, so the bound is what makes a red run fail instead of hanging.
const TERMINAL_BOUND_SECONDS: &str = "30";

/// The passphrase the client key is protected by.
///  Nothing answers it, so a run that asks waits until the bound.
const KEY_PASSPHRASE: &str = "correct horse battery staple";

/// A pseudo-terminal the control keeps open:
/// the master end,
/// which the control reads and nothing writes into,
/// and the slave end,
/// which is a child's standard input and its controlling terminal.
struct Terminal {
    /// Read for everything the terminal carried; kept open for the child's whole run.
    master: std::fs::File,
    /// The end the child's standard input is made from.
    slave: std::fs::File,
}

/// Open a new pseudo-terminal.
fn terminal() -> Terminal {
    use std::os::fd::FromRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    // SAFETY: `posix_openpt` returns a descriptor nothing else owns, which `File` takes over at once; the two calls after it
    //  only read or change that descriptor. `O_NOCTTY` keeps the terminal from becoming this test process's own.
    let (master, descriptor): (std::fs::File, libc::c_int) = unsafe {
        let descriptor: libc::c_int = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(descriptor >= 0, "a pseudo-terminal is opened");
        let master: std::fs::File = std::fs::File::from_raw_fd(descriptor);
        assert_eq!(libc::grantpt(descriptor), 0, "the slave is granted");
        assert_eq!(libc::unlockpt(descriptor), 0, "the slave is unlocked");
        (master, descriptor)
    };
    let name: std::ffi::CString = pseudo_terminal_slave_name(descriptor);
    let slave: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open(name.to_str().expect("a UTF-8 terminal name"))
        .expect("the slave opens");
    return Terminal { master, slave };
}

/// What one command at a terminal showed.
struct AtTerminal {
    /// Everything the terminal carried,
    /// which is where a question OpenSSH asks a person is written.
    transcript: String,
    /// The status the program chose,
    /// and nothing when a signal ended it,
    /// which is what the bound does to a command that asked and waited.
    code: Option<i32>,
}

/// Everything the terminal carried so far,
/// waiting no longer than `wait_milliseconds` for the first byte.
fn read_terminal(master: &std::fs::File, collected: &mut Vec<u8>, wait_milliseconds: libc::c_int) {
    use std::os::fd::AsRawFd;
    let mut descriptor = libc::pollfd { fd: master.as_raw_fd(), events: libc::POLLIN, revents: 0 };
    // SAFETY: `descriptor` is one `pollfd` this function owns, and `poll` only reads and writes it.
    let ready: libc::c_int = unsafe { libc::poll(&mut descriptor, 1, wait_milliseconds) };
    if ready <= 0 {
        return;
    }
    // One read per look: the master is a blocking descriptor, so a second read waits for a byte that may never come.
    let mut buffer: [u8; 4096] = [0; 4096];
    // SAFETY: the buffer is this function's, and the read writes at most its length into it.
    let got: isize = unsafe { libc::read(master.as_raw_fd(), buffer.as_mut_ptr().cast(), buffer.len()) };
    if got <= 0 {
        return;
    }
    collected.extend_from_slice(buffer.get(..usize::try_from(got).unwrap_or(0)).unwrap_or_default());
}

/// What:
///  Run one command in a session of its own whose controlling terminal is `at`,
///  read everything the terminal carries until the command ends or the bound ends it,
///  and answer with the transcript and the status.
/// Why:
///   OpenSSH reads the question from `/dev/tty` and not from standard input or standard error,
///  so a control that read only its output could not see whether it asked;
///  the transcript is where both the question and an answer's absence appear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { transcript, code } = await runAtTerminal(command, at); // reads the pty master until exit
/// ```
fn run_at(
    fixture: &Fixture,
    program: &Path,
    arguments: &[OsString],
    variables: &[(&str, &str)],
    at: &Terminal,
    stop_when: Option<&str>,
) -> AtTerminal {
    use std::os::unix::process::CommandExt as _;
    let stderr_path: PathBuf = fixture.root.join("terminal-stderr.txt");
    let stderr_file: std::fs::File = std::fs::File::create(stderr_path.as_path()).expect("the standard error file");
    let mut command: Command = bounded_for(fixture, program, wrapper_first_path(fixture).as_os_str(), TERMINAL_BOUND_SECONDS);
    command
        .args(arguments)
        .envs(variables.iter().copied())
        // The question is on the terminal; standard output is only where the program's own words are.
        .stdin(Stdio::from(at.slave.try_clone().expect("duplicate the terminal")))
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));
    // SAFETY: between fork and exec only async-signal-safe calls run: `setsid`, and one `ioctl` on descriptor 0, which is the
    //  terminal this command was given.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            libc::ioctl(0, libc::TIOCSCTTY, 0);
            return Ok(());
        });
    }
    let mut child: Child = command.spawn().expect("the command at the terminal starts");
    let bound: u64 = TERMINAL_BOUND_SECONDS.parse().unwrap_or(30);
    let deadline: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(bound);
    let mut collected: Vec<u8> = Vec::new();
    let mut ended: Option<std::process::ExitStatus> = None;
    while std::time::Instant::now() < deadline {
        read_terminal(&at.master, &mut collected, 200);
        let carried: String = String::from_utf8_lossy(collected.as_slice()).into_owned();
        if stop_when.is_some_and(|marker| { return carried.contains(marker); }) {
            // The control saw what it came for; the rest of its run is a wait for an answer nobody gives.
            // SAFETY: `child` is this call's own child, and the group is the one `setsid` made in it.
            unsafe {
                libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
            }
            let _collected = child.wait().expect("the control at the terminal is collected");
            break;
        }
        if let Some(status) = child.try_wait().expect("the command's state is read") {
            ended = Some(status);
            break;
        }
    }
    // Everything the terminal carried before the end, and everything the command wrote before it.
    read_terminal(&at.master, &mut collected, 200);
    let status: std::process::ExitStatus = match ended {
        Some(status) => status,
        None => {
            // A command past its bound is the red case: the question was asked and waited for. The bound process leads the
            //  session's group, so its whole group ends together.
            // SAFETY: `child` is this call's own child, and the group is the one `setsid` made in it.
            unsafe {
                libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
            }
            child.wait().expect("the command is collected")
        }
    };
    let mut written: String = String::new();
    if let Ok(mut reported) = std::fs::File::open(stderr_path.as_path()) {
        let _read: std::io::Result<usize> = reported.read_to_string(&mut written);
    }
    let mut transcript: String = String::from_utf8_lossy(collected.as_slice()).replace('\r', "");
    if !written.trim().is_empty() {
        transcript.push_str(&format!("\nstderr:\n{written}"));
    }
    return AtTerminal { transcript, code: status.code() };
}

/// The first executable file with a program's name in the two places a distribution puts the SSH programs or in this test
///  process's own `PATH`,
///  because the fixture's `PATH` carries neither.
fn program(name: &str) -> Option<PathBuf> {
    let mut directories: Vec<PathBuf> = vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/sbin"), PathBuf::from("/bin")];
    if let Some(path) = std::env::var_os("PATH") {
        directories.extend(std::env::split_paths(&path));
    }
    return directories.into_iter().map(|directory| { return directory.join(name); }).find(|candidate| { return candidate.is_file(); });
}

/// The port a throwaway `sshd` listens on:
///  taken by binding it and giving it up at once,
///  which is how the review's probe found one.
fn free_port() -> u16 {
    let listener: std::net::TcpListener = std::net::TcpListener::bind("127.0.0.1:0").expect("a local port");
    let port: u16 = listener.local_addr().expect("the address it bound").port();
    drop(listener);
    return port;
}

/// A throwaway `sshd` on `127.0.0.1`, stopped with the fixture.
struct Sshd {
    child: Child,
}

/// What:
///  Start a throwaway `sshd` for this account with a throwaway host key,
///  and answer with it and the port it listens on as soon as it accepts a connection.
/// Why:
///   The question is asked over a real SSH connection,
/// and the account's own keys, agents and `known_hosts` stay out of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sshd = await startSshd(fixture); // its own host key, its own authorized_keys, 127.0.0.1 only
/// ```
fn start_sshd(sshd: &Path, fixture: &Fixture, port: u16, ssh_directory: &Path) -> Sshd {
    let config: PathBuf = ssh_directory.join("sshd_config");
    let config_text: String = format!(
        "Port {port}\nListenAddress 127.0.0.1\nHostKey {host_key}\nPidFile {pid_file}\nAuthorizedKeysFile {authorized_keys}\n\
         PasswordAuthentication no\nKbdInteractiveAuthentication no\nPubkeyAuthentication yes\nUsePAM no\nStrictModes no\nPerSourcePenalties no\nLogLevel ERROR\n",
        host_key = ssh_directory.join("host_key").display(),
        pid_file = ssh_directory.join("sshd.pid").display(),
        authorized_keys = ssh_directory.join("authorized_keys").display(),
    );
    std::fs::write(config.as_path(), config_text.as_bytes()).expect("the sshd configuration");
    let mut command: Command = Command::new(sshd);
    command.args(["-f", config.to_str().expect("a UTF-8 configuration path"), "-D", "-e"]);
    command.current_dir(fixture.root.as_path()).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let started: Sshd = Sshd { child: command.spawn().expect("the throwaway sshd starts") };
    let deadline: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return started;
        }
        if std::time::Instant::now() >= deadline {
            // Dropping it ends it, so a server that never listened leaves no process behind either.
            drop(started);
            panic!("the throwaway sshd never accepted a connection on port {port}");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

impl Drop for Sshd {
    fn drop(&mut self) {
        let _stopped = self.child.kill();
        let _collected = self.child.wait();
    }
}

/// What:
///  Make the two keys a control of the wrapper's SSH behaviour uses:
///  a throwaway host key,
///  and a client key protected by `phrase` (none for an empty one) which the throwaway `sshd` accepts.
/// Why:
///   The account's own keys,
///   its agent and its `known_hosts` stay out of every control,
///  and each control needs the same three files:
///  the host key,
///  the client key and the `authorized_keys` that names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makeKeys(sshDir, ''); // ssh-keygen -N '' for client_key and host_key, then authorized_keys = client_key.pub
/// ```
fn make_ssh_keys(keygen: &Path, ssh_directory: &Path, phrase: &str) {
    for (name, protection) in [("client_key", phrase), ("host_key", "")] {
        let key: PathBuf = ssh_directory.join(name);
        let made: std::process::Output = Command::new(keygen)
            .args(["-q", "-t", "ed25519", "-N", protection, "-f", key.to_str().expect("a UTF-8 path")])
            .output()
            .expect("ssh-keygen starts");
        assert!(made.status.success(), "ssh-keygen failed: {}", String::from_utf8_lossy(made.stderr.as_slice()));
    }
    let authorized: String = std::fs::read_to_string(ssh_directory.join("client_key.pub")).expect("the public key");
    std::fs::write(ssh_directory.join("authorized_keys"), authorized.as_bytes()).expect("authorized_keys");
}

/// What:
///  The options a person's own `GIT_SSH_COMMAND` would carry for a key of the fixture's own,
///  and nothing of the wrapper's rule.
/// Why:
///   `-F none` and `GlobalKnownHostsFile=none` keep the account's own files out:
///  OpenSSH takes the home directory from the passwd entry and not from `HOME`,
///  so a `~/.ssh/config` naming an `IdentityFile`,
///  a `ProxyCommand` or an `Include` would change what either arm sees.
///  `strict` is `no` for the controls that are about the key and `ask` for the control that is about the host key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sshArgs(sshDir, identity, port, 'no') // ['-F', 'none', '-i', identity, '-p', port, '-o', 'StrictHostKeyChecking=no', ...]
/// ```
fn ssh_arguments(ssh_directory: &Path, identity: &Path, port: u16, strict: &str) -> Vec<OsString> {
    let known_hosts: PathBuf = ssh_directory.join("known_hosts");
    return vec![
        OsString::from("-F"),
        OsString::from("none"),
        OsString::from("-i"),
        identity.to_path_buf().into_os_string(),
        OsString::from("-p"),
        OsString::from(port.to_string()),
        OsString::from("-o"),
        OsString::from("IdentitiesOnly=yes"),
        OsString::from("-o"),
        OsString::from("IdentityAgent=none"),
        OsString::from("-o"),
        OsString::from("GlobalKnownHostsFile=none"),
        OsString::from("-o"),
        OsString::from(format!("UserKnownHostsFile={}", known_hosts.display())),
        OsString::from("-o"),
        OsString::from(format!("StrictHostKeyChecking={strict}")),
    ];
}

/// The same options as one `GIT_SSH_COMMAND` value,
/// which is how a person's own command would carry them.
fn ssh_command(ssh: &Path, arguments: &[OsString]) -> String {
    let parts: Vec<String> = arguments.iter().map(|part| { return part.to_string_lossy().into_owned(); }).collect();
    return format!("{} {}", ssh.display(), parts.join(" "));
}

/// A Git command of the fixture's own,
/// run through the wrapper with the fixture's environment.
fn git_through_wrapper(fixture: &Fixture, arguments: &[&str]) -> std::process::Output {
    return Command::new(fixture.root.join("bin/git"))
        .args(arguments)
        .env_clear()
        .env("PATH", wrapper_first_path(fixture))
        .env("HOME", fixture.root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdin(Stdio::null())
        .current_dir(fixture.root.as_path())
        .output()
        .expect("git starts");
}

/// What:
///  A typed command over `ssh://` at a real terminal asks no passphrase,
///  and the control shows that the same terminal does show one when nothing forbids it.
/// Why:
///   The owner's decision counts a key's passphrase as a credential,
///  and `SSH_ASKPASS_REQUIRE=never` alone met it only without a terminal:
///  OpenSSH's own `read_passphrase` then reads `/dev/tty` (`readpass.c:150-167` and `:185` at `V_10_6_P2`),
///  which the integration pass measured through the wrapper.
///  The key here is protected by a passphrase and no agent holds it,
///  so a run that asks waits for an answer that never comes and is the red case this control fails on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(control.transcript).toContain('Enter passphrase for key');
/// expect(wrapped.transcript.toLowerCase()).not.toContain('passphrase'); expect(wrapped.code).toBe(128)
/// ```
#[cfg(unix)]
#[test]
fn a_typed_command_over_ssh_at_a_terminal_asks_no_passphrase() {
    let fixture: Fixture = fixture("ssh-terminal-passphrase");
    let (Some(ssh), Some(keygen), Some(sshd)) = (program("ssh"), program("ssh-keygen"), program("sshd")) else {
        eprintln!("skipped: ssh, ssh-keygen or sshd is not installed, so no real SSH connection was exercised");
        remove_settled(&fixture);
        return;
    };
    let ssh_directory: PathBuf = fixture.root.join("ssh");
    std::fs::create_dir_all(ssh_directory.as_path()).expect("the ssh directory");
    make_ssh_keys(keygen.as_path(), ssh_directory.as_path(), KEY_PASSPHRASE);
    let port: u16 = free_port();
    let server: Sshd = start_sshd(sshd.as_path(), &fixture, port, ssh_directory.as_path());

    // The repository the command reaches, made by the wrapper's own `git init`.
    let served: PathBuf = fixture.root.join("served.git");
    let served_text: String = served.to_string_lossy().into_owned();
    let made: std::process::Output = git_through_wrapper(&fixture, &["init", "--bare", "--quiet", served_text.as_str()]);
    assert!(made.status.success(), "git init failed: {}", String::from_utf8_lossy(made.stderr.as_slice()));
    let url: String = format!("ssh://127.0.0.1:{port}{served_text}");
    let identity: PathBuf = ssh_directory.join("client_key");
    let options: Vec<OsString> = ssh_arguments(ssh_directory.as_path(), identity.as_path(), port, "no");

    // The control: `ssh` alone at the same kind of terminal, with the wrapper's variables absent, asks the question.
    //  Without it, a transcript with no question could mean the terminal cannot show one.
    let control_at: Terminal = terminal();
    let mut control_arguments: Vec<OsString> = options.clone();
    control_arguments.push(OsString::from("127.0.0.1"));
    control_arguments.push(OsString::from("true"));
    let control: AtTerminal = run_at(&fixture, ssh.as_path(), control_arguments.as_slice(), &[], &control_at, Some("Enter passphrase for key"));
    assert!(
        control.transcript.contains("Enter passphrase for key"),
        "the terminal shows OpenSSH's question when nothing forbids it: {:?}",
        control.transcript
    );

    // The claim: the same command through the wrapper at a terminal asks nothing and fails at once.
    let at: Terminal = terminal();
    let ssh_command: String = ssh_command(ssh.as_path(), options.as_slice());
    let wrapped_arguments: Vec<OsString> = vec![OsString::from("ls-remote"), OsString::from(url.as_str())];
    let wrapped: AtTerminal = run_at(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        wrapped_arguments.as_slice(),
        &[("GIT_SSH_COMMAND", ssh_command.as_str())],
        &at,
        None,
    );
    assert!(
        !wrapped.transcript.to_lowercase().contains("passphrase"),
        "the wrapper's command asked for the key's passphrase at a terminal: {:?}",
        wrapped.transcript
    );
    assert_eq!(
        wrapped.code,
        Some(128),
        "the command failed at once instead of waiting for an answer: {:?}",
        wrapped.transcript
    );
    assert!(wrapped.transcript.contains("Permission denied (publickey"), "the connection reached the key and was refused it: {:?}", wrapped.transcript);
    drop(server);
    remove_settled(&fixture);
}

/// What:
///  A key without a passphrase connects through the wrapper with no agent running,
///  once at a real terminal and once with no terminal at all.
/// Why:
///   The project's answer of 2026-11-20 is "We must support situations in which ssh-agent isn't there":
///  a key without a passphrase keeps working with no agent,
///  and this control proves it.
///  The no-prompt rule replaces the account's `SSH_ASKPASS` for these children and forbids every question,
///  so a key that needs nothing from a person must still reach the server.
///  No `SSH_AUTH_SOCK` reaches either child:
///  the environment is built from named values only,
///  and `-o IdentityAgent=none` keeps OpenSSH from finding an agent by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(ptyRun(wrapper, ['ls-remote', url]).code).toBe(0); expect(run(wrapper, ['ls-remote', url]).stdout).toContain('refs/tags/probe')
/// ```
#[cfg(unix)]
#[test]
fn a_key_without_a_passphrase_connects_through_the_wrapper_with_no_agent() {
    let fixture: Fixture = fixture("ssh-terminal-no-agent");
    let (Some(ssh), Some(keygen), Some(sshd)) = (program("ssh"), program("ssh-keygen"), program("sshd")) else {
        eprintln!("skipped: ssh, ssh-keygen or sshd is not installed, so no real SSH connection was exercised");
        remove_settled(&fixture);
        return;
    };
    let ssh_directory: PathBuf = fixture.root.join("ssh");
    std::fs::create_dir_all(ssh_directory.as_path()).expect("the ssh directory");
    // An empty passphrase leaves the key protected by nothing, so no agent has to hold it.
    make_ssh_keys(keygen.as_path(), ssh_directory.as_path(), "");
    let port: u16 = free_port();
    let server: Sshd = start_sshd(sshd.as_path(), &fixture, port, ssh_directory.as_path());

    // The repository the command lists, with one ref to name, made by the wrapper's own Git.
    let served: PathBuf = fixture.root.join("served.git");
    let served_text: String = served.to_string_lossy().into_owned();
    let made: std::process::Output = git_through_wrapper(&fixture, &["init", "--bare", "--quiet", served_text.as_str()]);
    assert!(made.status.success(), "git init failed: {}", String::from_utf8_lossy(made.stderr.as_slice()));
    let written: std::process::Output = git_through_wrapper(&fixture, &["-C", served_text.as_str(), "hash-object", "-w", "--stdin"]);
    assert!(written.status.success(), "hash-object failed: {}", String::from_utf8_lossy(written.stderr.as_slice()));
    let object: String = String::from_utf8_lossy(written.stdout.as_slice()).trim().to_owned();
    // A tag may name any object, while a branch must name a commit, and the listing only needs a ref to print.
    let pointed: std::process::Output =
        git_through_wrapper(&fixture, &["-C", served_text.as_str(), "update-ref", "refs/tags/probe", object.as_str()]);
    assert!(pointed.status.success(), "update-ref failed: {}", String::from_utf8_lossy(pointed.stderr.as_slice()));
    let url: String = format!("ssh://127.0.0.1:{port}{served_text}");
    let identity: PathBuf = ssh_directory.join("client_key");
    let options: Vec<OsString> = ssh_arguments(ssh_directory.as_path(), identity.as_path(), port, "no");
    let command_line: String = ssh_command(ssh.as_path(), options.as_slice());
    let wrapped_arguments: Vec<OsString> = vec![OsString::from("ls-remote"), OsString::from(url.as_str())];

    // Arm one: a real terminal, which is where a passphrase question would be asked.
    let at: Terminal = terminal();
    let at_terminal: AtTerminal = run_at(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        wrapped_arguments.as_slice(),
        &[("GIT_SSH_COMMAND", command_line.as_str())],
        &at,
        None,
    );
    assert_eq!(
        at_terminal.code,
        Some(0),
        "the key without a passphrase connected with no agent at a terminal: {:?}",
        at_terminal.transcript
    );
    assert!(!at_terminal.transcript.contains("Permission denied"), "the key was accepted: {:?}", at_terminal.transcript);
    assert!(!at_terminal.transcript.to_lowercase().contains("passphrase"), "no passphrase was asked for: {:?}", at_terminal.transcript);

    // Arm two: no terminal at all, where a command that asked would find nothing to read.
    let mut command: Command = bounded_for(&fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(&fixture).as_os_str(), TERMINAL_BOUND_SECONDS);
    command
        .args(wrapped_arguments.as_slice())
        .env("GIT_SSH_COMMAND", command_line.as_str())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    assert!(
        !command.get_envs().any(|(name, _value)| { return name.to_string_lossy() == "SSH_AUTH_SOCK"; }),
        "the child starts with no agent socket at all"
    );
    let without_terminal: std::process::Output = command.output().expect("the command without a terminal starts");
    assert_eq!(
        without_terminal.status.code(),
        Some(0),
        "the key without a passphrase connected with no agent and no terminal: {}",
        String::from_utf8_lossy(without_terminal.stderr.as_slice())
    );
    assert!(
        String::from_utf8_lossy(without_terminal.stdout.as_slice()).contains("refs/tags/probe"),
        "the connection listed the ref: {}",
        String::from_utf8_lossy(without_terminal.stdout.as_slice())
    );
    drop(server);
    remove_settled(&fixture);
}

/// What:
///  An unknown host key is refused at once through the wrapper at a terminal,
///  and the same kind of terminal shows the question when nothing forbids it.
/// Why:
///   `SSH_ASKPASS_REQUIRE=force` sends OpenSSH's yes/no question for an unknown host key to a program that answers nothing,
///  and an empty answer is not "yes",
///  so the connection is refused before any login
///  (`confirm`,
///  `sshconnect.c` at `V_10_6_P2` lines 603 to 627).
///  The project's answer of 2026-11-20 keeps that refusal and puts the way out in the words the wrapper owns,
///  which plain `ssh` at this same terminal still shows a question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(control.transcript).toContain('Are you sure you want to continue connecting'); expect(wrapped.transcript).toContain('Host key verification failed.')
/// ```
#[cfg(unix)]
#[test]
fn an_unknown_host_key_is_refused_at_a_terminal_and_never_asked() {
    let fixture: Fixture = fixture("ssh-terminal-host-key");
    let (Some(ssh), Some(keygen), Some(sshd)) = (program("ssh"), program("ssh-keygen"), program("sshd")) else {
        eprintln!("skipped: ssh, ssh-keygen or sshd is not installed, so no real SSH connection was exercised");
        remove_settled(&fixture);
        return;
    };
    let ssh_directory: PathBuf = fixture.root.join("ssh");
    std::fs::create_dir_all(ssh_directory.as_path()).expect("the ssh directory");
    make_ssh_keys(keygen.as_path(), ssh_directory.as_path(), "");
    let port: u16 = free_port();
    let server: Sshd = start_sshd(sshd.as_path(), &fixture, port, ssh_directory.as_path());
    // No `known_hosts` entry is ever written, so the server's key is unknown to both arms.
    let identity: PathBuf = ssh_directory.join("client_key");
    let options: Vec<OsString> = ssh_arguments(ssh_directory.as_path(), identity.as_path(), port, "ask");
    let command_line: String = ssh_command(ssh.as_path(), options.as_slice());

    // The control: `ssh` alone at the same kind of terminal, with the wrapper's variables absent, asks the question.
    let control_at: Terminal = terminal();
    let mut control_arguments: Vec<OsString> = options.clone();
    control_arguments.push(OsString::from("127.0.0.1"));
    control_arguments.push(OsString::from("true"));
    let control: AtTerminal = run_at(
        &fixture,
        ssh.as_path(),
        control_arguments.as_slice(),
        &[],
        &control_at,
        Some("Are you sure you want to continue connecting"),
    );
    assert!(
        control.transcript.contains("Are you sure you want to continue connecting"),
        "the terminal shows OpenSSH's host-key question when nothing forbids it: {:?}",
        control.transcript
    );

    // The claim: the same terminal, through the wrapper, is refused before any login and never asked.
    let at: Terminal = terminal();
    let url: String = format!("ssh://127.0.0.1:{port}/served.git");
    let wrapped_arguments: Vec<OsString> = vec![OsString::from("ls-remote"), OsString::from(url.as_str())];
    let wrapped: AtTerminal = run_at(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        wrapped_arguments.as_slice(),
        &[("GIT_SSH_COMMAND", command_line.as_str())],
        &at,
        None,
    );
    assert!(!wrapped.transcript.contains("Are you sure"), "the host key was never asked about: {:?}", wrapped.transcript);
    assert!(
        wrapped.transcript.contains("Host key verification failed"),
        "OpenSSH's refusal names the host key: {:?}",
        wrapped.transcript
    );
    assert_eq!(wrapped.code, Some(128), "the connection failed at once instead of waiting: {:?}", wrapped.transcript);
    drop(server);
    remove_settled(&fixture);
}
