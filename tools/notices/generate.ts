// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { collect } from './collect.ts';
import { html, profiles, read, sha256, write, type Profile } from './core.ts';
import rustLibrary from './rust-library.json';

export const verifyRustNotice = (bytes: Uint8Array): void => {
  if (sha256(bytes) !== rustLibrary.sha256) throw new Error('Rust library attribution checksum mismatch');
};
export const generate = async (root: string, profile: Profile, destinations: string[]): Promise<void> => {
  for (const destination of destinations) {
    if (destination.split(/[\\/]/).includes('..')) throw new Error('Invalid attribution destination');
  }
  const version = read(root, 'rust-toolchain.toml').match(/channel = "([^"]+)"/)?.[1];
  if (version !== rustLibrary.version) throw new Error('Review Rust library attribution for the new toolchain');
  const cache = join(root, 'notices/RUST_LIBRARY_NOTICES.html');
  let bytes: Uint8Array;
  if (existsSync(cache)) bytes = readFileSync(cache);
  else {
    const sysroot = execFileSync('rustc', ['--print', 'sysroot'], {cwd: root, encoding: 'utf8'}).trim();
    const installed = join(sysroot, 'share/doc/rust/COPYRIGHT-library.html');
    if (existsSync(installed)) bytes = readFileSync(installed);
    else {
      const response = await fetch(rustLibrary.url, {signal: AbortSignal.timeout(60000)});
      if (!response.ok) throw new Error(`Rust attribution download failed: HTTP ${response.status}`);
      bytes = new Uint8Array(await response.arrayBuffer());
    }
  }
  verifyRustNotice(bytes);
  // Fetch only this artifact's locked Cargo graph; collection itself is offline.
  const config = profiles[profile];
  execFileSync('cargo', ['fetch', '--locked', '--manifest-path', config.manifest,
    ...(config.target ? ['--target', config.target] : [])], {cwd: root, stdio: 'inherit'});
  const result = collect(root, profile);
  const standard = new TextDecoder().decode(bytes);
  write(root, 'notices/RUST_LIBRARY_NOTICES.html', standard);
  write(root, `notices/${profile}.txt`, result.text);
  for (const destination of destinations) {
    const out = resolve(root, destination);
    if (!['examples/cloudflare-worker/build/', 'examples/configurable-chart/engine/', 'examples/configurable-chart/public/', 'target/'].some(part => destination.startsWith(part))) {
      throw new Error('Attribution destination must be a build output directory');
    }
    write(out, 'LICENSE', read(root, 'LICENSE'));
    write(out, 'THIRD_PARTY_NOTICES.txt', result.text);
    write(out, 'RUST_LIBRARY_NOTICES.html', standard);
    write(out, 'THIRD_PARTY_NOTICES.html', html(result.text).replace('</pre>',
      '</pre><p><a href="RUST_LIBRARY_NOTICES.html">Rust standard-library attribution</a></p>'));
  }
};
if (import.meta.main) {
  const [profile] = process.argv.slice(2);
  if (!profile || !(profile in profiles)) throw new Error('Usage: bun tools/notices/generate.ts runtime|builder|worker|browser');
  const root = resolve(import.meta.dir, '../..');
  await generate(root, profile as Profile, [`target/distribution/${profile}/`]);
}
