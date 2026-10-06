import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, rmSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { root } from '../scripts/build-site.mjs';

test('documented workflows and boundary combinations match the executable', () => {
  const binary = path.join(root, 'target/debug', process.platform === 'win32' ? 'onesource.exe' : 'onesource');
  const dir = mkdtempSync(path.join(tmpdir(), 'onesource-docs-'));
  const run = (...args) => execFileSync(binary, args, { cwd: dir, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  try {
    mkdirSync(path.join(dir, 'src'));
    writeFileSync(path.join(dir, 'src/main.rs'), 'fn main() {}');
    writeFileSync(path.join(dir, 'Cargo.toml'), '[package]\nname="example"');
    writeFileSync(path.join(dir, '.env'), 'SECRET=example');
    writeFileSync(path.join(dir, '.gitignore'), 'ignored.txt\n');
    writeFileSync(path.join(dir, 'ignored.txt'), 'ignored');
    writeFileSync(path.join(dir, 'exact.txt'), 'x'.repeat(1024));
    writeFileSync(path.join(dir, 'over.txt'), 'x'.repeat(1025));
    writeFileSync(path.join(dir, 'binary.dat'), Buffer.from([97, 0]));
    writeFileSync(path.join(dir, 'legacy.txt'), Buffer.from([255, 97]));
    const output = path.join(dir, 'review.onesource');
    const preview = run('--no-config', '-o', output, '-m', '1', '--dry-run', '--copy');
    assert.match(preview, /\[EXPECT\] exact.txt/);
    assert.match(preview, /\[SKIP\] over.txt \(larger than 1 KiB: 1025 bytes\)/);
    assert.match(preview, /\[SKIP\] binary.dat \(binary file\)/);
    assert.match(preview, /legacy.txt .*lossy UTF-8/);
    assert.match(preview, /NO COPY WAS MADE WHILE DRY RUN/);
    assert.equal(existsSync(output), false);
    run('--no-config', '-i', '*.rs', '-o', output);
    const bundle = readFileSync(output, 'utf8');
    assert.match(bundle, /<file path="src\/main.rs">/);
    assert.ok(!bundle.includes('<file path="Cargo.toml">'));
    run('profile', 'create', 'review', '-i', '*.rs', '--no-tree');
    run('-p', 'review', '--include=', '--no-tree=false', '--save', '--dry-run');
    const profile = JSON.parse(readFileSync(path.join(dir, '.onesourcerc'), 'utf8')).profiles.review;
    assert.equal(profile.include, '');
    assert.equal(profile.no_tree, false);
    assert.equal(profile.dry_run, undefined);
    const configBefore = readFileSync(path.join(dir, '.onesourcerc'), 'utf8');
    const diagnosis = run('explain', 'Cargo.toml', '--no-config', '-i', '*.rs', '--ti', '*.toml', '--save', '--replace', '--copy');
    assert.match(diagnosis, /not included by include/);
    assert.match(diagnosis, /Tree:\n    Result: included/);
    assert.equal(readFileSync(path.join(dir, '.onesourcerc'), 'utf8'), configBefore);
    assert.match(run('explain', 'ignored.txt', '--no-config'), /blocked by ignore filters/);
    assert.match(run('explain', '.env', '--no-config'), /blocked by blacklist/);
    assert.match(run('explain', '.env', '--no-config', '--no-blacklist'), /Result: included/);
    run('profile', 'update', 'review', '--replace', '-m', '1');
    assert.deepEqual(JSON.parse(run('profile', 'show', 'review', '--json')), { max_size: 1 });
    run('profile', 'desc', 'Legacy description', '-p', 'review');
    assert.match(run('profile', 'ls'), /Legacy description/);
    run('profile', 'rename', 'review', 'backend');
    run('profile', 'rm', 'backend');
    assert.throws(() => run('--no-config', '--replace', '--dry-run'), /--replace can only be used with --save/);
    assert.throws(() => run('--no-config', '--copy=false'), /unexpected value 'false'/);
    assert.throws(() => run('--no-config', '-i', '[', '--dry-run'), /Invalid glob pattern/);
    run('--no-config', '-i', '*.rs', '--save', '--dry-run');
    assert.equal(JSON.parse(readFileSync(path.join(dir, '.onesourcerc'), 'utf8')).profiles.default.include, '*.rs');
    if (process.platform !== 'win32') {
      symlinkSync(path.join(dir, 'src'), path.join(dir, 'linked-src'));
      const links = run('--no-config', '--dry-run');
      assert.ok(!links.includes('[EXPECT] linked-src/main.rs'));
    }
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

const decode = text => text.replace(/&quot;/g, '"').replace(/&#x27;|&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
const tutorials = JSON.parse(readFileSync(path.join(root, 'site/docs.en.json'), 'utf8'));
const fixtures = {
  'getting-started': { 'Cargo.toml': '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n', 'README.md': '# Demo\n', 'src/main.rs': 'fn main() {}\n' },
  filtering: { 'README.md': '# Demo\n', 'src/main.rs': 'fn main() {}\n', 'src/util.rs': 'pub fn value() -> u8 { 1 }\n', 'src/legacy/old.rs': 'fn old() {}\n', 'tests/smoke.rs': '#[test]\nfn smoke() {}\n', '.gitignore': 'notes.txt\n', 'notes.txt': 'Local notes\n', '.env': 'EXAMPLE_KEY=demo-only\n' },
  tree: { 'README.md': '# Demo\n', 'src/main.rs': 'fn main() {}\n', 'src/legacy/old.rs': 'fn old() {}\n', 'tests/smoke.rs': '#[test]\nfn smoke() {}\n', '.gitignore': 'notes.txt\n', 'notes.txt': 'Local notes\n' },
  profiles: { 'README.md': '# Demo\n', 'src/main.rs': 'fn main() {}\n', 'tests/smoke.rs': '#[test]\nfn smoke() {}\n' },
  explain: { 'README.md': '# Demo\n', 'src/main.rs': 'fn main() {}\n', '.gitignore': 'notes.txt\n', 'notes.txt': 'Local notes\n', '.env': 'EXAMPLE_KEY=demo-only\n' },
};
fixtures.output = fixtures['getting-started'];
function commandLines(slug) {
  return tutorials[slug].sections.flatMap(section => [...section.html.matchAll(/<pre data-kind="command"><code>([\s\S]*?)<\/code><\/pre>/g)].flatMap(match => decode(match[1]).split('\n')));
}
function argsOf(line) {
  // The published tutorial grammar uses double-quoted arguments without shell expansions.
  return [...line.matchAll(/"([^"]*)"|([^\s"]+)/g)].map(match => match[1] ?? match[2]).reduce((args, token, index, tokens) => {
    if (index && tokens[index - 1].endsWith('=') && args.length) args[args.length - 1] += token;
    else args.push(token);
    return args;
  }, []);
}
for (const slug of Object.keys(fixtures)) test(`independent ${slug} walkthrough follows its published command sequence`, () => {
  const parent = mkdtempSync(path.join(tmpdir(), 'onesource-tutorial-'));
  const demo = path.join(parent, 'demo');
  const binary = path.join(root, 'target/debug', process.platform === 'win32' ? 'onesource.exe' : 'onesource');
  let cwd = demo;
  const executions = [];
  try {
    mkdirSync(demo);
    for (const [name, content] of Object.entries(fixtures[slug])) {
      mkdirSync(path.dirname(path.join(demo, name)), { recursive: true });
      writeFileSync(path.join(demo, name), content);
    }
    for (const line of commandLines(slug)) {
      const [command, ...args] = argsOf(line);
      if (command === 'cd') { cwd = path.resolve(cwd, args[0]); continue; }
      if (command === 'mkdir') { mkdirSync(path.resolve(cwd, args[0])); continue; }
      assert.equal(command, 'onesource', line);
      assert.notEqual(args[0], 'update', 'Executable replacement is outside tutorial integration checks');
      if (args.includes('/path/to/project')) continue; // Explicitly documented user-supplied root.
      if (args.includes('--copy') && !args.includes('--dry-run') && args[0] !== 'explain') continue; // Desktop clipboard tested separately.
      const stdout = execFileSync(binary, args, { cwd, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
      executions.push({ line, stdout });
      if (slug === 'profiles' && line === 'onesource profile show review --json') {
        const result = JSON.parse(stdout);
        if (result.max_size !== undefined) assert.equal(result.max_size, 100);
      }
    }
    const find = line => {
      const result = executions.find(result => result.line === line);
      assert.ok(result, `Missing execution: ${line}`);
      return result.stdout;
    };
    const paths = stdout => [...stdout.matchAll(/^\[EXPECT\] (.+?) \(/gm)].map(m => m[1].replaceAll('\\', '/'));
    const blockPaths = name => [...readFileSync(path.join(demo, name), 'utf8').matchAll(/<file path="([^"]+)">/g)].map(m => m[1].replaceAll('\\', '/'));
    if (slug === 'getting-started') {
      assert.deepEqual(paths(find('onesource --no-config --dry-run')), ['Cargo.toml', 'README.md', 'src/main.rs']);
      assert.deepEqual(blockPaths('review.onesource'), ['src/main.rs']);
    }
    if (slug === 'filtering') {
      assert.deepEqual(paths(find('onesource --no-config -i "*.rs" --dry-run')), ['src/legacy/old.rs', 'src/main.rs', 'src/util.rs', 'tests/smoke.rs']);
      assert.equal(paths(find('onesource --no-config -i "src/**" --dry-run')).length, 3);
      assert.equal(paths(find('onesource --no-config -i "src/**,README.md" --dry-run')).length, 4);
      assert.deepEqual(blockPaths('review.onesource'), ['README.md', 'src/main.rs', 'src/util.rs']);
      assert.deepEqual(paths(find('onesource --no-config -i "src/[mu]*.rs" --dry-run')), ['src/main.rs', 'src/util.rs']);
      assert.equal(paths(find('onesource --no-config --include="" --dry-run')).length, 5);
      assert.equal(paths(find('onesource --no-config -i "notes.txt" --dry-run')).length, 0);
      assert.deepEqual(paths(find('onesource --no-config -i "notes.txt" --no-ignore --dry-run')), ['notes.txt']);
    }
    if (slug === 'tree') {
      const narrow = find('onesource --no-config -i "src/main.rs" --dry-run');
      assert.match(narrow, /demo\/\n└── src\/\n    └── main.rs/);
      const broad = find('onesource --no-config -i "src/main.rs" --tree-include="" --dry-run');
      assert.match(broad, /README.md/);
      assert.deepEqual(paths(broad), ['src/main.rs']);
      const separate = find('onesource --no-config --no-ignore --tree-no-ignore=false --dry-run');
      assert.ok(!separate.split('[EXPECT]')[0].includes('notes.txt'));
      assert.ok(paths(separate).includes('notes.txt'));
      assert.ok(!/^demo\/$/m.test(find('onesource --no-config -i "src/main.rs" --no-tree --dry-run')));
      assert.deepEqual(blockPaths('map.onesource'), ['src/main.rs']);
    }
    if (slug === 'profiles') {
      assert.match(find('onesource -p review --dry-run'), /Files Processed: 1/);
      assert.match(find('onesource -p review -i "src/**,tests/**" --dry-run'), /Files Processed: 2/);
      assert.match(find('onesource -p review -i "src/**,README.md" --no-tree=false --save --dry-run'), /Files Processed: 2/);
      assert.deepEqual(JSON.parse(executions.filter(e => e.line === 'onesource profile show review --json').at(-1).stdout), { include: 'README.md' });
      const config = JSON.parse(readFileSync(path.join(demo, '.onesourcerc'), 'utf8'));
      assert.deepEqual(config.profiles.default, { description: 'Backend source', include: 'src/**' });
      assert.equal(config.profiles.backend, undefined);
      assert.equal(config.profiles.review, undefined);
    }
    if (slug === 'explain') {
      assert.match(find('onesource explain notes.txt --no-config -i "notes.txt"'), /blocked by ignore filters/);
      assert.match(find('onesource explain notes.txt --no-config -i "notes.txt" --no-ignore'), /Content:\n    Result: included/);
      assert.match(find('onesource explain README.md --no-config -i "*.rs" --ti "*.md"'), /Rule: include = \*\.rs/);
      assert.deepEqual(paths(find('onesource --no-config -i "*.rs,README.md" --ti "*.md" --dry-run')), ['README.md', 'src/main.rs']);
    }
    if (slug === 'output') {
      assert.match(find('onesource demo --no-config -o demo/review.onesource --dry-run'), /Files Processed: 3/);
      assert.deepEqual(blockPaths('review.onesource'), ['Cargo.toml', 'README.md', 'src/main.rs']);
      assert.deepEqual(blockPaths('bundles/source.onesource'), ['src/main.rs']);
      assert.match(find('onesource --no-config --dry-run'), /Files Processed: 3/);
      assert.deepEqual(JSON.parse(readFileSync(path.join(demo, '.onesourcerc'), 'utf8')).profiles.default, { include: 'src/**' });
    }
  } finally { rmSync(parent, { recursive: true, force: true }); }
});

test('configuration migration example preserves values and restores unnamed-run behavior', () => {
  const dir = mkdtempSync(path.join(tmpdir(), 'onesource-migration-'));
  const binary = path.join(root, 'target/debug', process.platform === 'win32' ? 'onesource.exe' : 'onesource');
  const migration = tutorials.troubleshooting.sections.find(s => s.id === 'migration');
  const configs = [...migration.html.matchAll(/<pre data-kind="config"><code>([\s\S]*?)<\/code><\/pre>/g)].map(m => decode(m[1]));
  const run = args => execFileSync(binary, args, { cwd: dir, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  try {
    mkdirSync(path.join(dir, 'src'));
    writeFileSync(path.join(dir, 'src/main.rs'), 'fn main() {}');
    writeFileSync(path.join(dir, '.onesourcerc'), configs[0]);
    assert.throws(() => run(['--dry-run']), /Incompatible configuration format/);
    assert.match(run(['--no-config', '--dry-run']), /Files Processed: 1/);
    writeFileSync(path.join(dir, '.onesourcerc'), configs[1]);
    assert.deepEqual(JSON.parse(run(['profile', 'show', 'default', '--json'])), JSON.parse(configs[0]));
    const preview = run(['--dry-run']);
    assert.match(preview, /using profile: default/);
    assert.match(preview, /Files Processed: 1/);
    assert.ok(!preview.includes('└──'));
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
