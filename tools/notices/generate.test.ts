import { test } from 'bun:test';
import assert from 'node:assert/strict';
import { html } from './core.ts';
import { validateCollection } from './collect.ts';
import { verifyRustNotice } from './generate.ts';

test('unknown and synthesized license results need explicit review', () => {
  const p = { name: 'unreviewed', version: '1.0.0', license: 'MIT', manifest_path: '/unused/Cargo.toml', source: 'registry+https://github.com/rust-lang/crates.io-index' };
  const data = { crates: [{ package: p, license: 'MIT' }], licenses: [{ id: 'MIT', text: 'license', source_path: null, used_by: [{ crate: p }] }] };
  assert.throws(() => validateCollection(data, new Set()), /Unreviewed synthesized/);
  assert.throws(() => validateCollection({ ...data, licenses: [] }, new Set()), /Unresolved/);
});
test('HTML treats third-party text as text, not executable markup', () => {
  assert.ok(!html('<script>alert("x")</script>').includes('<script>'));
  assert.match(html('https://example.test/source'), /<a href="https:\/\/example.test\/source">/);
});

test('non-crates.io and unreviewed path dependencies cannot acquire misleading notices', () => {
  const p = { name: 'external', version: '1.0.0', license: 'MIT', manifest_path: '/external/Cargo.toml', source: 'registry+https://example.test/index' as string | null };
  const data = { crates: [{ package: p, license: 'MIT' }], licenses: [{ id: 'MIT', text: 'license', source_path: '/external/LICENSE', used_by: [{ crate: p }] }] };
  assert.throws(() => validateCollection(data, new Set()), /unsupported registry/);
  p.source = null;
  assert.throws(() => validateCollection(data, new Set()), /Unreviewed path dependency/);
});
test('first-party paths require MIT metadata', () => {
  const p = { name: 'astrology-engine', version: '0.1.0', license: 'MIT', manifest_path: '/project/Cargo.toml', source: null };
  const data = { crates: [{ package: p, license: 'MIT' }], licenses: [] };
  validateCollection(data, new Set(), new Set([p.manifest_path]));
  p.license = 'MPL-2.0';
  assert.throws(() => validateCollection(data, new Set(), new Set([p.manifest_path])), /Unreviewed path dependency/);
});

test('corrupt Rust attribution is rejected', () => { assert.throws(() => verifyRustNotice(new TextEncoder().encode('invalid')), /checksum/); });
