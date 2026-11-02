/**
 Source of the function the stand-in `podman start` runs to play cargo-mutants inside the container.

 It reads the created container's arguments the way a real container would see them: the host directory bind-mounted
 at the report path, `--iterate`, and the previous `mutants.out/previously_caught.txt`. It rotates `mutants.out` to
 `mutants.out.old`, leaves out the mutants `--iterate` names, appends each scored mutant to its list file as cargo-mutants
 does, and ends with the attempt's own exit status, 3 for a timeout and 2 for a missed mutant, or 0.
 The environment drives it: `FAKE_MUTANTS` (comma-separated names in scoring order), `FAKE_MISSED`, `FAKE_TIMEOUT` and
 `FAKE_UNVIABLE` (names that get those outcomes, the rest are caught), `FAKE_KILL_AFTER` (end the process with status 137
 after that many scored mutants) and `FAKE_KILL_START` (the start number the kill applies to, default 1).
 `FAKE_OUTCOMES=1` stands for five caught mutants.
 Every scored mutant adds a line `<start number>:<name>` to `scored.log` in the fixture directory.
 The source expects `fs`, `path`, `directory` and `campaignImageId` in scope.
 */
export const fakeMutantsSimulationSource: string = String.raw`
function listOf(name) {
  return (process.env[name] || '').split(',').filter(Boolean);
}
function completeLines(file) {
  return fs.existsSync(file) ? fs.readFileSync(file, 'utf8').split('\n').slice(0, -1) : [];
}
function simulateCargoMutants() {
  const mutants = process.env.FAKE_MUTANTS ? listOf('FAKE_MUTANTS') : (process.env.FAKE_OUTCOMES === '1' ? ['m1', 'm2', 'm3', 'm4', 'm5'] : []);
  if (mutants.length === 0) return;
  const created = JSON.parse(fs.readFileSync(path.join(directory, 'created.json'), 'utf8'));
  const host = created[created.indexOf('--volume') + 1].split(':')[0];
  const command = created.slice(created.indexOf(campaignImageId) + 1);
  const startsFile = path.join(directory, 'starts.count');
  const start = (fs.existsSync(startsFile) ? Number(fs.readFileSync(startsFile, 'utf8')) : 0) + 1;
  fs.writeFileSync(startsFile, String(start));
  const out = path.join(host, 'mutants.out');
  const skipped = command.includes('--iterate') ? completeLines(path.join(out, 'previously_caught.txt')) : [];
  if (fs.existsSync(out)) {
    fs.rmSync(path.join(host, 'mutants.out.old'), { recursive: true, force: true });
    fs.renameSync(out, path.join(host, 'mutants.out.old'));
  }
  fs.mkdirSync(path.join(out, 'log'), { recursive: true });
  fs.mkdirSync(path.join(out, 'diff'));
  fs.writeFileSync(path.join(out, 'mutants.json'), JSON.stringify(mutants));
  if (command.includes('--iterate')) fs.writeFileSync(path.join(out, 'previously_caught.txt'), skipped.map((name) => name + '\n').join(''));
  const killAfter = process.env.FAKE_KILL_AFTER !== undefined && start === Number(process.env.FAKE_KILL_START || '1') ? Number(process.env.FAKE_KILL_AFTER) : -1;
  const counts = { caught: 0, missed: 0, timeout: 0, unviable: 0 };
  const entries = [];
  for (const name of mutants) {
    if (skipped.includes(name)) continue;
    if (entries.length === killAfter) process.exit(137);
    const outcome = listOf('FAKE_MISSED').includes(name) ? 'missed' : listOf('FAKE_TIMEOUT').includes(name) ? 'timeout' : listOf('FAKE_UNVIABLE').includes(name) ? 'unviable' : 'caught';
    counts[outcome]++;
    entries.push({ scenario: { Mutant: { name } }, summary: outcome });
    fs.writeFileSync(path.join(out, 'log', name.replace(/[^a-z0-9]/gi, '_') + '.log'), name + '\n');
    fs.writeFileSync(path.join(out, 'outcomes.json'), JSON.stringify({ outcomes: entries, total_mutants: entries.length, ...counts, success: 0 }));
    fs.appendFileSync(path.join(out, outcome + '.txt'), name + '\n');
    fs.appendFileSync(path.join(directory, 'scored.log'), start + ':' + name + '\n');
  }
  if (entries.length === 0) fs.writeFileSync(path.join(out, 'outcomes.json'), JSON.stringify({ outcomes: [], total_mutants: 0, caught: 0, missed: 0, timeout: 0, unviable: 0, success: 0 }));
  process.exit(counts.timeout > 0 ? 3 : counts.missed > 0 ? 2 : 0);
}
`;
