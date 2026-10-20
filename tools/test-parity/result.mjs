// Preserve upstream skip/todo provenance even when a Rust port executes its assertions.
export function acceptanceState(upstreamMode, executionState) {
  return upstreamMode !== 'run' && executionState === 'passed' ? 'skipped' : executionState;
}
