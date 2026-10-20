import { test } from 'node:test';
import assert from 'node:assert/strict';
import { acceptanceState } from './result.mjs';

test('a passing port cannot turn an upstream skip or todo into an acceptance pass', () => {
  assert.equal(acceptanceState('run', 'passed'), 'passed');
  assert.equal(acceptanceState('skip', 'passed'), 'skipped');
  assert.equal(acceptanceState('todo', 'passed'), 'skipped');
});

test('upstream skip/todo never conceals a failed, missing or unverified port', () => {
  for (const mode of ['run', 'skip', 'todo']) {
    for (const state of ['failed', 'unverified', 'unmapped', 'skipped']) {
      assert.equal(acceptanceState(mode, state), state);
    }
  }
});
