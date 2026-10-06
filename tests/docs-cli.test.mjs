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
