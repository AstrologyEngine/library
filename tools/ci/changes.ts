// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

import { execFileSync } from 'node:child_process';
import { appendFileSync, readFileSync } from 'node:fs';

export const jobs = ['native', 'tooling', 'worker', 'configurable-chart'] as const;
export type Job = typeof jobs[number];
export type Selection = Record<Job, boolean>;
export const allJobs = (): Selection => Object.fromEntries(jobs.map(job => [job, true])) as Selection;
export const selectJobs = (paths: string[]): Selection => {
  const selected = Object.fromEntries(jobs.map(job => [job, false])) as Selection;
  for (const path of paths) {
    // Legal/build inputs take precedence over documentation exemptions.
    if (/^(LICENSE|THIRD_PARTY_NOTICES)(\.|$)/.test(path) ||
        /^(src\/|\.github\/|\.cargo\/|tools\/(ci|notices|verification)\/)/.test(path)) return allJobs();
    const prose = /^(docs\/.*\.(md|rst|txt)|[^/]+\.md)$/i.test(path) || /(^|\/)(README|AGENTS)\.md$/.test(path);
    if (prose) continue;
    const affected: Job[] | undefined = path.startsWith('examples/cloudflare-worker/') ? ['worker']
      : path.startsWith('examples/configurable-chart/') ? ['configurable-chart']
      : /^(tools\/dataset-builder\/|tools\/jpl\/|tools\/regenerate\.py$)/.test(path) ? ['native', 'tooling']
      : undefined;
    if (!affected) return allJobs();
    for (const job of affected) selected[job] = true;
  }
  return selected;
};
export const changedPaths = (base: string, head: string, run = (args: string[]): string =>
  execFileSync('git', args, { encoding: 'utf8' })): string[] => {
  if (![base, head].every(ref => /^[a-f0-9]{40}$/.test(ref))) throw new Error('Expected base/head commit IDs');
  const ancestor = run(['merge-base', base, head]).trim();
  // Disable rename collapsing so both old and new paths affect selection.
  return run(['diff', '--name-only', '--no-renames', '-z', ancestor, head, '--']).split('\0').filter(Boolean);
};
export const determineSelection = (event: string, collect: () => { paths: string[] }): Selection => {
  if (event !== 'pull_request') return allJobs();
  try {
    const { paths } = collect();
    return selectJobs(paths);
  } catch (error) {
    console.warn(`Change detection unavailable; running all checks: ${error}`);
    return allJobs();
  }
};
if (import.meta.main) {
  const selected = determineSelection(process.env.GITHUB_EVENT_NAME ?? '', () => {
    const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH!, 'utf8'));
    const base: string = event.pull_request.base.sha;
    const head: string = event.pull_request.head.sha;
    return { paths: changedPaths(base, head) };
  });
  const output = jobs.map(job => `${job}=${selected[job]}`).join('\n') + '\n';
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, output);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY,
    `### Selected checks\n${jobs.map(job => `- ${job}: ${selected[job] ? 'run' : 'unaffected'}`).join('\n')}\n`);
  console.log(output);
}
