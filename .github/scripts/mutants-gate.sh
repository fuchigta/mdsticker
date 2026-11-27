#!/usr/bin/env bash
# Mutation testing gate: sums mutants.out/outcomes.json over all shards.
#
# Environment (set by .github/workflows/mutation.yml):
#   EXPECTED        number of mutants in the diff, counted by the plan job
#   MAX_MISSED      upper bound on survivors (missed); fail when exceeded
#   PLAN_RESULT     result of the plan job (fail unless success)
#   MUTANTS_RESULT  result of the shard jobs (skipped when EXPECTED is 0)
#   OUTCOMES_DIR    directory holding mutants-out-<shard>/ (default: cwd)
set -euo pipefail
: "${MAX_MISSED:?}"
dir="${OUTCOMES_DIR:-.}"

if [[ "${PLAN_RESULT:-}" != "success" ]]; then
  echo "::error::plan job ended with '${PLAN_RESULT:-unknown}'; cannot judge"
  exit 1
fi
: "${EXPECTED:?}"

if [[ "$EXPECTED" == "0" ]]; then
  echo "No mutants in the diff; nothing to test, treated as success"
  exit 0
fi

if [[ "${MUTANTS_RESULT:-}" != "success" ]]; then
  echo "::error::shard jobs ended with '${MUTANTS_RESULT:-unknown}' (unmutated tests or build failed?)"
  exit 1
fi

shopt -s nullglob
files=("$dir"/mutants-out-*/outcomes.json)
if (( ${#files[@]} == 0 )); then
  echo "::error::no outcomes.json found; no evidence that any mutant was tested"
  exit 1
fi

sum() { jq -s "map(.$1) | add" "${files[@]}"; }
tested=$(sum total_mutants)
caught=$(sum caught)
missed=$(sum missed)
timeout=$(sum timeout)
unviable=$(sum unviable)

echo "Tested mutants: ${tested} (planned ${EXPECTED}, ${#files[@]} shard(s))"
echo "  caught ${caught} / missed ${missed} / timeout ${timeout} / unviable ${unviable}"
viable=$(( caught + missed + timeout ))
if (( viable > 0 )); then
  # A timeout means the mutant made the tests hang, i.e. it was detected.
  echo "  score ((caught + timeout) / (caught + missed + timeout)): $(( (caught + timeout) * 1000 / viable / 10 )).$(( (caught + timeout) * 1000 / viable % 10 ))%"
fi

if (( tested != EXPECTED )); then
  echo "::error::tested ${tested} mutants but the plan expected ${EXPECTED} (missing shard or empty run?)"
  exit 1
fi

if (( missed > 0 )); then
  echo "Missed mutants:"
  cat "$dir"/mutants-out-*/missed.txt | sed 's/^/  /'
fi

if (( missed > MAX_MISSED )); then
  echo "::error::${missed} missed mutants exceed MAX_MISSED=${MAX_MISSED}. Add tests, or, for equivalent mutants, explain in the PR and revisit the limit"
  exit 1
fi
echo "${missed} missed mutants is within MAX_MISSED=${MAX_MISSED}"
