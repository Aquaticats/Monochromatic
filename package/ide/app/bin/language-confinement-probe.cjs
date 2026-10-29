// Escape and reachability probe for language-server confinement, run from inside a server's process tree.
// Adapted from the measurement in doc/planning/slint-ide-write-confinement.md ("In-tree probes").
// Runs as `node language-confinement-probe.cjs <who>` or via require(...).runProbe(who).
// Inputs come from IDE_PROBE_* variables; private state and the report directory default to
// XDG_CACHE_HOME, which the confined launch points into the server's private state.
// Output: one JSON document at <report directory>/probe-<who>-<pid>.json.
'use strict';
const fs = require('node:fs');
const net = require('node:net');
const path = require('node:path');
const { spawnSync, spawn } = require('node:child_process');

const attempt = (label, action) => {
  try {
    const value = action();
    return { label, ok: true, ...(value === undefined ? {} : { value }) };
  } catch (error) {
    return { label, ok: false, code: error.code ?? null, errno: error.errno ?? null, message: String(error.message).slice(0, 200) };
  }
};

const connectProbe = (label, options) => new Promise((resolve) => {
  const socket = net.connect(options);
  const finish = (result) => {
    socket.destroy();
    resolve({ label, ...result });
  };
  socket.setTimeout(1500, () => finish({ ok: false, code: 'TIMEOUT' }));
  socket.once('connect', () => finish({ ok: true }));
  socket.once('error', (error) => finish({ ok: false, code: error.code ?? null, message: String(error.message).slice(0, 200) }));
});

const run = (label, command, args, options = {}) => {
  const result = spawnSync(command, args, { encoding: 'utf8', timeout: 20000, ...options });
  return {
    label,
    ok: result.status === 0,
    status: result.status,
    signal: result.signal,
    error: result.error ? String(result.error.code ?? result.error.message) : null,
    stdout: String(result.stdout ?? '').slice(0, 400),
    stderr: String(result.stderr ?? '').slice(0, 400),
  };
};

async function runProbe(who) {
  const env = process.env;
  const project = env.IDE_PROBE_PROJECT;
  const state = env.IDE_PROBE_STATE ?? env.XDG_CACHE_HOME;
  const outside = env.IDE_PROBE_OUTSIDE;
  const tmp = env.IDE_PROBE_TMP;
  const reportDir = env.IDE_PROBE_REPORT_DIR ?? env.XDG_CACHE_HOME;
  const hostPid = Number(env.IDE_PROBE_HOST_PID ?? 0);
  const tag = `${who}_${process.pid}`;
  const results = [];
  const push = (entry) => results.push(entry);
  const victim = (name) => path.join(project, name);

  // region plain writes: project must fail, private state must succeed.
  push(attempt('project.create', () => fs.writeFileSync(path.join(project, `PROBE_${tag}_create`), 'x')));
  push(attempt('project.append', () => fs.appendFileSync(victim('victim-append.txt'), 'x')));
  push(attempt('project.truncate', () => fs.truncateSync(victim('victim-truncate.txt'), 0)));
  push(attempt('project.rename', () => fs.renameSync(victim('victim-rename.txt'), victim('victim-rename.moved'))));
  push(attempt('project.unlink', () => fs.unlinkSync(victim('victim-unlink.txt'))));
  push(attempt('project.mkdir', () => fs.mkdirSync(path.join(project, `PROBE_${tag}_dir`))));
  push(attempt('project.chmod', () => fs.chmodSync(victim('victim-meta.txt'), 0o600)));
  push(attempt('project.utimes', () => fs.utimesSync(victim('victim-meta.txt'), 1, 1)));
  push(run('project.setxattr', 'python3', ['-c', 'import os,sys; os.setxattr(sys.argv[1], "user.ide_probe", b"1")', victim('victim-meta.txt')]));
  push(attempt('state.create', () => fs.writeFileSync(path.join(state, `PROBE_${tag}_create`), 'x')));
  if (outside) push(attempt('outside.create', () => fs.writeFileSync(path.join(outside, `PROBE_${tag}_create`), 'x')));
  if (tmp) {
    push(attempt('tmp.create', () => fs.writeFileSync(path.join(tmp, `PROBE_${tag}_create`), 'x')));
    push(attempt('vartmp.create', () => {
      const file = `/var/tmp/PROBE_${tag}`;
      fs.writeFileSync(file, 'x');
      fs.unlinkSync(file);
    }));
  }
  push(attempt('devnull.write', () => fs.writeFileSync('/dev/null', 'x')));
  push(attempt('devshm.create', () => {
    const file = `/dev/shm/PROBE_${tag}`;
    fs.writeFileSync(file, 'x');
    fs.unlinkSync(file);
  }));
  // endregion

  // region link and rename escapes.
  // Symlink in writable state pointing into the project, then write through it.
  push(attempt('escape.symlink-into-project.write', () => {
    const link = path.join(state, `PROBE_${tag}_symlink`);
    fs.symlinkSync(victim('victim-link.txt'), link);
    fs.appendFileSync(link, 'x');
  }));
  // Symlink created inside the project pointing out.
  push(attempt('escape.symlink-out-of-project.create', () => fs.symlinkSync('/etc/hostname', path.join(project, `PROBE_${tag}_symlink`))));
  // Hard link from writable state to a project inode, then write through it.
  const hardlink = attempt('escape.hardlink-into-state.create', () => fs.linkSync(victim('victim-link.txt'), path.join(state, `PROBE_${tag}_hardlink`)));
  push(hardlink);
  // Writing through the link only means something when the link exists.
  if (hardlink.ok) push(attempt('escape.hardlink-into-state.write', () => fs.appendFileSync(path.join(state, `PROBE_${tag}_hardlink`), 'x')));
  // Hard link inside the project.
  push(attempt('escape.hardlink-in-project.create', () => fs.linkSync(victim('victim-link.txt'), path.join(project, `PROBE_${tag}_hardlink`))));
  // Rename a state file over a project file.
  push(attempt('escape.rename-over-project-file', () => {
    const source = path.join(state, `PROBE_${tag}_rename_source`);
    fs.writeFileSync(source, 'replaced');
    fs.renameSync(source, victim('victim-link.txt'));
  }));
  // Rename a project file out into state (removal from the project).
  push(attempt('escape.rename-out-of-project', () => fs.renameSync(victim('victim-steal.txt'), path.join(state, `PROBE_${tag}_stolen`))));
  // endregion

  // region path aliases.
  push(attempt('alias.proc-self-root', () => fs.appendFileSync(path.join('/proc/self/root', victim('victim-append.txt')), 'x')));
  push(attempt('alias.proc-self-cwd', () => fs.appendFileSync(`/proc/self/cwd/victim-append.txt`, 'x')));
  if (hostPid) push(attempt('alias.proc-hostpid-root', () => fs.appendFileSync(path.join(`/proc/${hostPid}/root`, victim('victim-append.txt')), 'x')));
  // /home is a symlink to var/home on this host; the other spelling of the same path.
  const swapped = project.startsWith('/var/home/') ? project.slice(4) : project.startsWith('/home/') ? `/var${project}` : null;
  if (swapped) push(attempt('alias.home-symlink-spelling', () => fs.appendFileSync(path.join(swapped, 'victim-append.txt'), 'x')));
  // An already open read-only descriptor reopened for writing through /proc/self/fd.
  push(attempt('alias.proc-self-fd-reopen', () => {
    const fd = fs.openSync(victim('victim-append.txt'), 'r');
    try {
      fs.appendFileSync(`/proc/self/fd/${fd}`, 'x');
    } finally {
      fs.closeSync(fd);
    }
  }));
  // endregion

  // region undoing the confinement from inside.
  const remountScript = 'mount --options remount,bind,rw "$1" 2>&1; echo x >> "$1/victim-append.txt"';
  push(run('undo.nested-userns-remount-rw', 'unshare', ['--user', '--map-root-user', '--mount', '--', 'sh', '-c', remountScript, 'sh', project]));
  push(run('undo.direct-remount-rw', 'mount', ['--options', 'remount,bind,rw', project]));
  // endregion

  // region delegation through host services.
  if (env.IDE_PROBE_DELEGATE === '1') {
    push(run('delegate.systemd-run-user', 'systemd-run', ['--user', '--wait', '--collect', '--quiet', '--',
      '/usr/bin/touch', path.join(project, `ESCAPED_VIA_SYSTEMD_${who}`)]));
  }
  // The host's runtime directory and display, passed explicitly because the confined environment is cleared.
  const runtime = env.IDE_PROBE_RUNTIME_DIR ?? env.XDG_RUNTIME_DIR ?? `/run/user/${process.getuid()}`;
  push(await connectProbe('reach.unix.dbus-session', { path: `${runtime}/bus` }));
  push(await connectProbe('reach.unix.wayland', { path: `${runtime}/${env.IDE_PROBE_WAYLAND ?? env.WAYLAND_DISPLAY ?? 'wayland-0'}` }));
  push(await connectProbe('reach.unix.system-dbus', { path: '/run/dbus/system_bus_socket' }));
  push(await connectProbe('reach.unix.abstract-x11', { path: '\0/tmp/.X11-unix/X0' }));
  if (env.IDE_PROBE_TCP_PORT) push(await connectProbe('reach.tcp.loopback-listener', { host: '127.0.0.1', port: Number(env.IDE_PROBE_TCP_PORT) }));
  if (hostPid) {
    push(attempt('reach.signal0-host-pid', () => process.kill(hostPid, 0)));
    push(attempt('reach.read-host-pid-environ', () => fs.readFileSync(`/proc/${hostPid}/environ`).length));
  }
  push(attempt('reach.read-home-ssh-dir', () => fs.readdirSync(path.join(env.HOME ?? '/', '.ssh')).length));
  // endregion

  // region double fork: a reparented grandchild writes after its parents are gone.
  const delayed = `sleep 1; echo x >> "$IDE_PROBE_PROJECT/DOUBLE_FORK_${who}" 2>"$IDE_PROBE_REPORT_DIR/double-fork-${tag}.err"; echo "rc=$?" >> "$IDE_PROBE_REPORT_DIR/double-fork-${tag}.log"`;
  const forked = spawn('setsid', ['--fork', 'sh', '-c', delayed], { detached: true, stdio: 'ignore', env: { ...env, IDE_PROBE_REPORT_DIR: reportDir ?? '/tmp' } });
  forked.unref();
  push({ label: 'doublefork.spawned', ok: true });
  // endregion

  const namespaces = {};
  for (const name of ['user', 'mnt', 'pid', 'net', 'ipc', 'uts', 'cgroup']) {
    namespaces[name] = attempt(name, () => fs.readlinkSync(`/proc/self/ns/${name}`)).value ?? null;
  }
  const status = fs.readFileSync('/proc/self/status', 'utf8');
  const field = (name) => (status.match(new RegExp(`^${name}:\\s*(.*)$`, 'm')) ?? [])[1] ?? null;
  // Read-write mount points, from the fifth and sixth mountinfo fields.
  const writableMounts = attempt('mounts', () => fs.readFileSync('/proc/self/mountinfo', 'utf8').split('\n')
    .map(line => line.split(' ')).filter(fields => fields.length > 5 && fields[5].split(',').includes('rw')).map(fields => fields[4])).value ?? null;
  const document = {
    who,
    environment: Object.keys(env).sort(),
    writableMounts,
    pid: process.pid,
    ppid: process.ppid,
    cwd: attempt('cwd', () => process.cwd()),
    uid: process.getuid(),
    namespaces,
    noNewPrivs: field('NoNewPrivs'),
    seccomp: field('Seccomp'),
    capEff: field('CapEff'),
    results,
  };
  const text = `${JSON.stringify(document, null, 2)}\n`;
  if (reportDir) {
    try {
      fs.writeFileSync(path.join(reportDir, `probe-${who}-${process.pid}.json`), text);
    } catch (error) {
      process.stderr.write(`probe: cannot write report: ${error.message}\n`);
    }
  }
  return document;
}

module.exports = { runProbe };

if (require.main === module) {
  runProbe(process.argv[2] ?? 'direct').then((document) => {
    process.stdout.write(`${JSON.stringify(document)}\n`);
  });
}
