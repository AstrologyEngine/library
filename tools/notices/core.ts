// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';

export const generatorVersion = '0.9.2';
import { workerBuildVersion, wasmBindgenVersion } from '../ci/versions.ts';
export const profiles = {
  runtime: { manifest: 'Cargo.toml', target: undefined },
  builder: { manifest: 'tools/dataset-builder/Cargo.toml', target: undefined },
  worker: { manifest: 'examples/cloudflare-worker/Cargo.toml', target: 'wasm32-unknown-unknown' },
  browser: { manifest: 'examples/configurable-chart/wasm/Cargo.toml', target: 'wasm32-unknown-unknown' },
} as const;
export type Profile = keyof typeof profiles;
export const profileNames = Object.keys(profiles) as Profile[];
export const sha256 = (value: string | Uint8Array): string => createHash('sha256').update(value).digest('hex');
export const read = (root: string, path: string): string => readFileSync(join(root, path), 'utf8');
export const write = (root: string, path: string, value: string): void => {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), value);
};
export const filesBelow = (root: string, directory: string): string[] =>
  readdirSync(join(root, directory), { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? filesBelow(root, path) : entry.isFile() ? [path] : [];
  }).sort();

export type Supplement = {
  name: string; version: string; profiles: Profile[]; source: string;
  files: { path: string; sha256: string }[];
};
export const supplements = (root: string): Supplement[] => {
  const entries: Supplement[] = JSON.parse(read(root, 'tools/notices/supplements.json'));
  const expected: Record<string, string> = {
    'worker-build': workerBuildVersion, 'wasm-bindgen-cli-support': wasmBindgenVersion,
    'rust-standard-library': read(root, 'rust-toolchain.toml').match(/channel = "([^"]+)"/)?.[1] ?? '',
    vite: JSON.parse(read(root, 'examples/configurable-chart/package.json')).devDependencies.vite,
  };
  for (const entry of entries) {
    if (expected[entry.name] && expected[entry.name] !== entry.version) throw new Error(`Update generator attribution: ${entry.name}`);
  }
  for (const entry of entries) for (const file of entry.files) {
    if (sha256(read(root, file.path)) !== file.sha256) throw new Error(`License material changed: ${file.path}`);
  }
  return entries;
};

export const html = (text: string): string => {
  const escaped = text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
  const linked = escaped.replace(/https:\/\/[^\s<>]+/g, url => `<a href="${url}">${url}</a>`);
  return `<!doctype html>\n<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Third-party notices</title><style>body{font:1rem/1.5 system-ui;max-width:75rem;margin:2rem auto;padding:0 1rem}pre{white-space:pre-wrap;overflow-wrap:anywhere}</style><h1>Third-party notices</h1><pre>${linked}</pre></html>\n`;
};

export type PackageNotice = { name: string; version: string; license: string; source: string };
