// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

import { execFileSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { generate } from '../../tools/notices/generate.ts';
import { workerBuildVersion } from '../../tools/ci/versions.ts';

const directory = dirname(fileURLToPath(import.meta.url));
const flags = process.argv.slice(2);
if (flags.length > 1 || flags.some(flag => flag !== '--verification')) throw new Error('Usage: bun build.ts [--verification]');
const version = execFileSync('worker-build', ['--version'], { encoding: 'utf8' }).trim().split(/\s+/).at(-1);
if (version !== workerBuildVersion) throw new Error(`Expected worker-build ${workerBuildVersion}, got ${version}`);
execFileSync('worker-build', ['--release', ...(flags.length ? ['--', '--features', 'verification'] : [])], { cwd: directory, stdio: 'inherit' });

await generate(resolve(directory, '../..'), 'worker', ['examples/cloudflare-worker/build/', 'examples/cloudflare-worker/build/worker/']);
