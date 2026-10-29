#!/usr/bin/env node
// Compare two watch scopes in disposable package copies, alternating runs of the ignored native measurement
// `watch_scope_reveal_staleness`: the shipped scope, which watches every expanded folder, and a copy patched
// to watch only the folders whose rows meet the tree's viewport, with the safety sweep covering the rest.
// Reports the kernel watch count with 60 expanded folders and how stale a folder that changed while out of
// view is when it is scrolled back in. The patch exists only in the disposable copy.
import { spawnSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, relative, sep } from 'node:path';
import { containerBounds } from './container-bounds.ts';
import { gpuArguments } from './gpu-arguments.ts';

// The bound of every container this script starts: the shared computed share of the host, or the caller's IDE_CONTAINER_* values.
const containerLimits = containerBounds();

const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
const disposable = ({ name, value }: { name: string; value: string | undefined }): string => {
  if (!value) throw new Error('Provide ' + name);
  const path = realpathSync(value);
  if (!path.startsWith(realpathSync(privateRoot) + sep) || !statSync(path).isDirectory()) throw new Error(name + ' must be a disposable directory below ' + privateRoot);
  return path;
};
const caches = { shipped: disposable({ name: 'the shipped-scope target cache', value: process.env.usage_cache }), viewport: disposable({ name: 'the viewport-scope target cache', value: process.env.usage_viewport_cache }) };
if (caches.shipped === caches.viewport) throw new Error('The two scopes need separate target caches');
const cargoHome = process.env.usage_cargo ? disposable({ name: 'the Cargo home copy', value: process.env.usage_cargo }) : 'ide-cargo';
const runs = Number(process.env.usage_runs ?? '3');
if (!Number.isInteger(runs) || runs < 1) throw new Error('runs must be a positive integer');

const origin = process.cwd();
const artifact = mkdtempSync(join(privateRoot, 'ide-watch-scope-'));
console.log('WATCH_SCOPE_ARTIFACT=' + artifact);
const copy = (name: string): string => {
  const target = join(artifact, name);
  cpSync(origin, target, { recursive: true, filter: path => !['target', '.git', 'node_modules'].includes(relative(origin, path).split(sep)[0] ?? '') });
  mkdirSync(join(target, 'target'), { recursive: true });
  return target;
};
const sources = { shipped: copy('shipped'), viewport: copy('viewport') };
const replaceOne = ({ path, before, after }: { path: string; before: string; after: string }): void => {
  const text = readFileSync(path, 'utf8');
  const start = text.indexOf(before);
  if (start < 0 || text.indexOf(before, start + before.length) >= 0) throw new Error('Patch anchor missing or ambiguous in ' + path + ': ' + before);
  writeFileSync(path, text.slice(0, start) + after + text.slice(start + before.length));
};
// The viewport scope: the tree sends no watch set when rows change; every tick sends the root and the
// expanded folders whose rows, children included, meet the viewport (48 px rows), and marks the other
// expanded folders as covered so the unwatched timer does not poll them; the safety sweep still lists them.
const watchFile = join(sources.viewport, 'src/native/navigation/watch.rs');
replaceOne({ path: watchFile, before: '    navigation.watcher.watch_only(&wanted, file.as_deref());\n    navigation.shown = shown;', after: '    let _ = (&wanted, &file);\n    navigation.shown = shown;' });
writeFileSync(watchFile, readFileSync(watchFile, 'utf8') + `
/// Viewport scope, measurement copy only.
pub(super) fn scope(window: &super::AppWindow, source: &Rc<RefCell<State>>, navigation: &mut Navigation) {
    let first = ((-window.get_tree_scroll_y()) / 48.0).floor().max(0.0) as usize;
    let count = (window.get_tree_viewport_height() / 48.0).ceil() as usize + 1;
    let last = first + count;
    let mut wanted = BTreeSet::new();
    wanted.insert(navigation.workspace.root().to_path_buf());
    let rows = &navigation.rows;
    for (index, row) in rows.iter().enumerate() {
        if !(row.entry.is_directory && row.expanded) {
            continue;
        }
        let mut end = index + 1;
        while end < rows.len() && rows[end].depth > row.depth {
            end += 1;
        }
        if index <= last && end > first {
            wanted.insert(row.entry.path.clone());
        }
    }
    let file = source.borrow().file_path.clone();
    navigation.watcher.watch_only(&wanted, file.as_deref());
    for path in navigation.shown.clone() {
        if !wanted.contains(&path) {
            navigation.watched.insert(path);
        }
    }
}
`);
replaceOne({ path: join(sources.viewport, 'src/native/navigation/tick.rs'), before: '    watch::update(source, &mut navigation);\n', after: '    watch::update(source, &mut navigation);\n    watch::scope(window, source, &mut navigation);\n' });

// What the ignored native measurement prints on one line.
interface WatchScopeMeasurement { stale_ms: number[]; kernel_watches: number; fresh_at_reveal: number; trials: number }
const results: unknown[] = [];
for (let run = 1; run <= runs; run++) {
  for (const scope of ['shipped', 'viewport'] as const) {
    const load = readFileSync('/proc/loadavg', 'utf8').split(' ').slice(0, 3).join(' ');
    const result = spawnSync('podman', [
      'run', '--rm', '--network=none', ...containerLimits.limitArguments,
      '--security-opt', 'label=disable',
      '--volume', sources[scope] + ':/work', '--volume', caches[scope] + ':/work/target', '--volume', cargoHome + ':/cargo',
      '--workdir', '/work', ...containerLimits.environmentArguments, '--env', 'SLINT_EMIT_DEBUG_INFO=1',
      ...gpuArguments, '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
      'localhost/monochromatic/ide',
      'cargo', 'nextest', 'run', '--offline', '--bin', 'monochromatic-ide',
      '--run-ignored', 'only', '--no-capture', '--filter-expr', 'test(watch_scope)',
    ], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
    const output = (result.stdout ?? '') + (result.stderr ?? '');
    writeFileSync(join(artifact, scope + '-run-' + run + '.log'), output);
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error('Run ' + run + ' of the ' + scope + ' scope failed; inspect ' + join(artifact, scope + '-run-' + run + '.log'));
    const line = output.split('\n').find(text => text.startsWith('{"case":"watch-scope"'));
    if (line === undefined) throw new Error('Run ' + run + ' of the ' + scope + ' scope printed no watch-scope record; inspect ' + join(artifact, scope + '-run-' + run + '.log'));
    const measured: WatchScopeMeasurement = JSON.parse(line);
    const stale = measured.stale_ms.filter(value => value > 0).sort((left, right) => left - right);
    results.push({ run, scope, load, ...measured });
    writeFileSync(join(artifact, 'results.json'), JSON.stringify(results, null, 2));
    console.log([run, scope, 'load=' + load, 'watches=' + measured.kernel_watches, 'fresh=' + measured.fresh_at_reveal + '/' + measured.trials,
      'stale_median=' + (stale.length ? stale[Math.floor(stale.length / 2)] : 0), 'stale_max=' + (stale.at(-1) ?? 0)].join(' '));
  }
}
console.log('Watch scope results: ' + join(artifact, 'results.json'));
