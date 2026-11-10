# Phase 0 Execution Backlog

**Status:** Proposed for review  
**Baseline:** TechSERS `main` as inspected during the Phase 0 audit; inherited Aspen implementation  
**Scope:** Risk remediation and evidence collection before agent-platform feature work  
**Change policy:** No production deployment, schema migration, destructive operation, or paid infrastructure commitment is authorized by this backlog.

## Operating rules

1. Preserve Git correctness and acknowledged-write durability over throughput.
2. Treat repository documentation and historical measurements as leads, not proof of current behaviour.
3. Record the exact commit SHA, environment, configuration, commands, raw outputs, and test artifacts for every result.
4. Separate static inspection, automated regression tests, local load tests, sustained tests, and deployed verification.
5. Do not increase concurrency or publish capacity claims until resource bounds and failure behaviour are demonstrated.
6. Any production or cloud test requiring spend, external infrastructure, or a change to the live service requires explicit approval of its target, duration, and budget first.
7. Keep this backlog documentation-only until a specific implementation task is reviewed and authorized.

## Priority and status definitions

- **P0:** blocks safe progression on the affected reliability/security surface.
- **P1:** required before a production-readiness claim or the dependent platform phase.
- **P2:** later enhancement.
- **Proposed:** scope and acceptance criteria await review.
- **In progress / Blocked / Ready for review / Done:** update only with linked evidence.

## Execution order

| Order | ID | Priority | Work item | Depends on | Current status |
|---:|---|---|---|---|---|
| 1 | TS-001 | P0 | Bound sampled-index local spool disk usage | None | Proposed |
| 2 | TS-005 | P0 | Establish safe push-concurrency envelope and close the 128-in-flight evidence gap | TS-001 resource model | Proposed |
| 3 | TS-002 | P0 | Complete continuous chaos acceptance | TS-001, TS-005, approved test host/budget | Proposed |
| 4 | TS-003 | P0 | Verify deployed quotas and emergency kill switch | Deployment target and explicit approval | Proposed |
| 5 | TS-004 | P0 | Verify public-endpoint security against deployment topology | TS-003, deployment target and explicit approval | Proposed |
| 6 | TS-006 | P1 | Define and test backup/restore RPO and RTO | Storage/metadata topology documented | Proposed |
| 7 | TS-007 | P1 | Decide distributed rate-limit semantics | Multi-instance support contract | Proposed |
| 8 | TS-008 | P1 | Define and enforce repository storage quotas | Tenant/storage accounting model | Proposed |
| 9 | TS-009 | P1 | Reconcile README, plan and security review | TS-001/TS-005 findings incorporated | Proposed |
| 10 | TS-010 | P1 | Establish TechSERS identity and upstream governance | License/provenance review | Proposed |
| 11 | TS-011 | P1 | Design agent coordination architecture | Git foundation gates | Deferred |
| 12 | TS-012 | P2 | Design trusted CI policy ingestion | Identity/provenance contracts | Deferred |

---

# TS-001 — Bound sampled-index local spool disk usage

**Priority:** P0  
**Type:** Resource safety / data-plane reliability  
**Known reference:** Upstream Aspen issue [#386](https://github.com/reason-machines/aspen/issues/386)  
**Evidence lead:** The 2026-11-07 entry in `docs/PLAN.md` describing sampled `.idx` local lookup copies and chain maps.

## Problem statement

The sampled-index optimization stores local lookup-section copies and chain maps under the configured spool directory. The execution-plan entry estimates approximately 40 bytes per indexed object across these files, with an example of approximately 464 MB for an 11.6-million-object import pack.

The same entry states that the sampled-index cache budget accounts for in-memory sample cost but not the full local-copy/chain-map disk footprint. Since pushes also use the spool, sufficiently large or numerous sampled packs may exhaust the shared filesystem and cause pushes or other spool-dependent operations to fail.

This is a documented risk to validate against the exact TechSERS commit before implementation. The numerical estimates are workload-specific, not universal sizing guarantees.

## Goals

- Establish an enforceable upper bound for sampled-index auxiliary disk usage.
- Ensure the bound is coordinated with the total spool budget used by push ingest and other temporary files.
- Ensure concurrent sample creation cannot oversubscribe the disk budget.
- Ensure cleanup, eviction, cancellation, crash recovery and failed writes release or reconcile reservations.
- Preserve correct lookup behaviour when a local copy cannot be admitted or written.

## Non-goals

- Replacing the pack/index format.
- Increasing push concurrency.
- Claiming that a configured byte limit is a hard filesystem quota unless the implementation enforces it against all relevant spool consumers.
- Silently degrading Git correctness to preserve an optimization.

## Investigation tasks

- [ ] Identify the current sampled-index cache implementation, including `Idx::cost`, sample lifecycle and eviction.
- [ ] Identify local lookup-copy and chain-map creation, ownership, cleanup and file-descriptor lifecycle.
- [ ] Inventory every consumer of the same spool directory: push ingest, sampled index files, chain maps, compaction/stitch temporary files and any other scratch data.
- [ ] Determine whether the spool path can be shared across processes or instances and whether the current accounting is process-local.
- [ ] Trace all failure paths: partial write, disk full, permission failure, process cancellation, panic, sample eviction and process restart.
- [ ] Confirm whether the cache can retain files after its in-memory entry is evicted or the process crashes.
- [ ] Record actual filesystem capacity and available-space behaviour in supported deployment modes.

## Implementation requirements

- [ ] Define explicit configuration for the sampled auxiliary-disk budget, with a documented default and validation.
- [ ] Reserve the expected disk cost before writing; reservations must be atomic with respect to concurrent creators.
- [ ] Account for actual bytes written, not only estimates. Handle underestimates without exceeding the configured limit.
- [ ] Define the relationship between sampled auxiliary storage and the total spool budget. Prevent independent sub-budgets from collectively exceeding the shared limit.
- [ ] When admission is denied, preserve correctness by falling back to the existing remote/ranged lookup path or another verified safe path; do not return incomplete or incorrect lookup results.
- [ ] Make reservation release idempotent and cover successful deletion, failed creation, partial files, eviction and cancellation.
- [ ] Add startup reconciliation for stale files if crash leftovers are possible. Reconciliation must not delete files still owned by a live process.
- [ ] Bound file descriptors as well as bytes; document the per-sample descriptor cost and enforce a practical cap.
- [ ] Expose metrics for current/reserved/peak sampled bytes, admission refusals, cleanup failures, stale-file recovery and fallback lookups.
- [ ] Emit actionable structured diagnostics when the budget is reached, without logging secrets or unbounded per-object details.

## Required tests

- [ ] Unit tests for budget arithmetic, overflow-safe size calculation and configuration validation.
- [ ] Concurrent reservation test proving N simultaneous creators cannot exceed the budget.
- [ ] Exact-budget, one-byte-over-budget and underestimated-size cases.
- [ ] Partial-write and simulated ENOSPC tests proving reservations and files are cleaned up.
- [ ] Cancellation/panic tests proving reservations are released.
- [ ] Eviction tests proving disk usage and descriptors return to baseline.
- [ ] Restart/reconciliation tests for stale partial and complete files.
- [ ] Multi-process test if the spool is supported as shared across processes; otherwise document and enforce the single-process ownership contract.
- [ ] Regression test proving lookups return identical results with local copies enabled, denied, missing or unreadable.
- [ ] Regression test proving push ingest remains correct when sampled-copy admission is denied.
- [ ] Resource test with multiple large sampled packs, measuring actual disk bytes and open descriptors.

## Acceptance criteria

TS-001 is **Done** only when all are true:

1. A documented, configurable and enforceable sampled auxiliary-disk limit exists.
2. Concurrent writers cannot exceed the reservation limit; actual writes are reconciled against reservations.
3. The total shared spool budget is not accidentally exceeded by independently accounted consumers.
4. Disk-full, cancellation, eviction and restart paths leave no unaccounted permanent growth.
5. Local lookup and push correctness are unchanged when the optimization is unavailable.
6. Metrics expose budget use, denials, cleanup failures and fallback behaviour.
7. Required CI tests pass on the implementation commit.
8. A reproducible stress result includes commit SHA, configuration, filesystem capacity, workload, peak bytes, descriptor count, and retained test artifacts.

**Evidence to attach:** test names and CI run; stress harness and raw result; before/after spool accounting; relevant metrics; reviewed code paths.

**Exit decision:** If the shared spool cannot be globally accounted for, document the limitation and block higher push concurrency until an enforceable bound exists.

---

# TS-005 — Establish safe push-concurrency envelope

**Priority:** P0  
**Type:** Admission control / capacity / operational safety  
**Evidence lead:** The execution-plan entry for sampled local lookups explicitly notes a remaining production evidence gap at 128 pushes in flight. The security review documents per-caller and per-instance admission limits, including bounded queues.

## Problem statement

The repository documents bounded per-caller and per-instance admission for fetch, push and stitch work. However, the Phase 0 audit identified an unresolved production evidence gap at 128 pushes in flight.

The number 128 must not be treated as a known safe capacity, configured limit, or confirmed failure threshold. This work item first establishes exactly what the historical statement means, then validates the configured admission behaviour and resource envelope under controlled tests.

## Goals

- Determine whether 128 means offered concurrent requests, accepted active pushes, queued requests, or a historical test target.
- Verify that per-caller and per-instance limits behave as documented.
- Measure memory, CPU, disk, file descriptors, database pool pressure, queue depth and latency under increasing offered concurrency.
- Establish a conservative, evidence-backed operating envelope and saturation behaviour.
- Verify that overload is bounded and rejected/queued predictably rather than causing unbounded resource growth.

## Non-goals

- Raising default concurrency limits as part of this investigation.
- Treating a closed-loop benchmark's achieved throughput as server maximum capacity.
- Running a public production load test without explicit approval.
- Claiming high availability or multi-instance global admission from per-instance controls.

## Investigation tasks

- [ ] Trace the configured per-caller and per-instance push limits from configuration parsing to admission and release.
- [ ] Verify the documented defaults and overrides against current code and deployment manifests.
- [ ] Trace the bounded queue capacity, wait timeout, rejection response, and `Retry-After` behaviour for HTTP and SSH.
- [ ] Determine whether all push work holds a permit for the full lifetime of body read, ingest, metadata publication and cleanup.
- [ ] Identify work that can continue after client disconnect, timeout, cancellation or server shutdown.
- [ ] Trace blocking tasks, database pool admission, object-store requests, sampled-index file creation and spool consumers.
- [ ] Locate the original source of the 128-in-flight gap and document its exact test/production context.
- [ ] Define offered concurrency separately from admitted active work, queued work and completed throughput.

## Test plan

**Step 1 — deterministic admission tests (CI):**
- [ ] Test per-caller limit at limit, limit + 1 and after permit release.
- [ ] Test per-instance active limit and bounded queue at capacity and capacity + 1.
- [ ] Test queue timeout and correct HTTP/SSH refusal semantics.
- [ ] Test cancellation, disconnect, panic and task failure always release permits.
- [ ] Test multiple callers so one caller cannot consume all per-caller allowance or bypass the instance-wide cap.
- [ ] Test that retries do not duplicate a push or bypass admission.
- [ ] Test that saturation metrics and `Retry-After` values are consistent with actual retryability.

**Step 2 — local controlled load test (no production traffic):**
- [ ] Use a fixed repository fixture and repeatable push workload.
- [ ] Sweep offered concurrency in staged increments, including 1, 4, 8, 16, 32, 64 and 128, subject to safe local resource limits.
- [ ] Run both one-caller and many-caller cases.
- [ ] Record offered, admitted, queued, rejected, completed and failed counts separately.
- [ ] Measure p50/p95/p99 latency, CPU, RSS, spool bytes, sampled auxiliary bytes, open descriptors, DB pool wait, object-store operations and queue age.
- [ ] Include slow-body, client-disconnect, transient database/object-store failure and restart scenarios.
- [ ] Verify the server returns to baseline resource use after each run.

**Step 3 — deployment-like verification (approval-gated):**
- [ ] Reproduce only in an isolated staging environment with no production repositories or credentials.
- [ ] Confirm deployment CPU/memory/disk limits and monitoring are active before starting.
- [ ] Set a written stop condition for resource use, error rate and latency.
- [ ] Obtain explicit approval for target, window and any cloud spend before provisioning or running.
- [ ] Do not test a public production endpoint in this backlog without a separate approval.

## Acceptance criteria

TS-005 is **Done** only when all are true:

1. The meaning and provenance of the 128-in-flight gap are documented.
2. Per-caller and per-instance admission controls match their documented configuration and behaviour.
3. Active work and queued work are both bounded, including cancellation and failure paths.
4. Saturation has deterministic, tested response semantics and observable metrics.
5. Load tests distinguish offered concurrency from accepted concurrency and throughput.
6. Resource measurements show no unbounded memory, disk, descriptor, queue or database-pool growth.
7. A conservative operating envelope is documented with hardware, configuration, workload and confidence limitations.
8. No default concurrency increase is made without a separate reviewed change and supporting evidence.
9. All regression tests pass on the implementation commit.

**Evidence to attach:** exact source paths; CI run; load-test harness; raw result files; resource graphs/metrics; stop-condition record; review of failure and cleanup paths.

**Exit decision:** Until TS-001 is complete and TS-005 demonstrates bounded spool use under load, do not raise push concurrency or describe the system as validated for 128 concurrent pushes.

---

# Remaining Phase 0 work items

These are intentionally scoped at the backlog level. They must receive detailed implementation plans before code changes.

## TS-002 — Complete continuous chaos acceptance

- [ ] Confirm the current chaos harness and the exact launch-gate scenario.
- [ ] Reconcile the historical 11.59-hour segmented run with the continuous 24-hour acceptance requirement.
- [ ] Ensure the build includes the fix for the historical storage-growth issue.
- [ ] Arrange a supervisor whose lifecycle is independent of an interactive coding session.
- [ ] Confirm host disk, memory and runtime capacity from a measured preflight.
- [ ] Obtain explicit host and budget approval.
- [ ] Run one continuous deployment with per-epoch checks and final queue-drain/dead-job/alert-counter checks.
- [ ] Preserve logs, commit SHA, configuration, environment description and result summary.
- [ ] Do not call the gate passed if the run is interrupted or final checks are absent.

## TS-003 — Verify deployed quotas and emergency kill switch

- [ ] Identify every quota and kill-switch control and its default.
- [ ] Trace authorization, failure behaviour and audit/metrics.
- [ ] Test limits and refusal semantics in isolated staging.
- [ ] Test emergency disable and recovery paths.
- [ ] Verify controls against the actual deployment topology.
- [ ] Obtain explicit approval before changing any live service.

## TS-004 — Verify public-endpoint security

- [ ] Inventory exposed listeners, proxy hops, trusted proxy ranges and network policies.
- [ ] Verify TLS termination, client-address propagation and rate-limit keys.
- [ ] Verify metrics are not publicly exposed.
- [ ] Verify authentication, authorization, quota and kill-switch behaviour externally.
- [ ] Reconcile deployed configuration with `docs/SECURITY.md`.
- [ ] Preserve a dated, reproducible security-verification report.

## TS-006 — Backup and restore

- [ ] Document object-store and metadata backup consistency requirements.
- [ ] Define RPO/RTO with maintainers before claiming recovery objectives.
- [ ] Test restore into an isolated environment.
- [ ] Verify refs, pack integrity, metadata relationships and Git client fetch/fsck.
- [ ] Test partial restore and interrupted restore recovery.
- [ ] Record actual recovery duration and data-loss window.

## TS-007 — Distributed rate limits

- [ ] Document current per-instance semantics and aggregate multi-instance exposure.
- [ ] Decide whether shared/distributed limits are required for the supported topology.
- [ ] If deferred, document the aggregate limit as per-instance limit multiplied by instance count.
- [ ] If implemented, define fail-closed/fail-open behaviour during limiter-store outage and test it.

## TS-008 — Repository storage quotas

- [ ] Define accounting units: logical repository size, stored pack bytes, temporary spool bytes and shared/deduplicated objects.
- [ ] Define quota enforcement points for push, mirror, compaction and administrative operations.
- [ ] Define behaviour for quota changes, concurrent pushes and garbage collection.
- [ ] Test that quota races cannot overshoot beyond the documented allowance.
- [ ] Expose usage and near-limit metrics.

## TS-009 — Documentation reconciliation

- [ ] Reconcile README launch-gate status with `docs/PLAN.md`.
- [ ] Reconcile the README's #118 status with `docs/SECURITY.md` and actual code.
- [ ] Update all claims only after verifying source and evidence.
- [ ] Label historical measurements with date, commit, topology and limitations.
- [ ] Ensure production-readiness language matches the remaining gates.

## TS-010 — TechSERS identity and upstream governance

- [ ] Confirm license, copyright notices, attribution and fork obligations.
- [ ] Document the upstream repository and synchronization model.
- [ ] Define how upstream changes are imported and conflicts resolved.
- [ ] Define ownership and review requirements for TechSERS-specific code.
- [ ] Distinguish inherited Aspen components from TechSERS additions in documentation.
- [ ] Record architecture decisions in reviewed ADRs.

## TS-011 — Agent coordination architecture (deferred)

Do not implement until Git reliability and identity contracts are reviewed. Design only after the preceding gates establish the foundation.

- [ ] Define human, service and agent identity boundaries.
- [ ] Define task-scoped permissions and revocation.
- [ ] Define isolated workspaces and cleanup guarantees.
- [ ] Define task/branch/commit/validation provenance.
- [ ] Define conflict detection and merge coordination contracts.
- [ ] Prove Git remains independently usable if the coordination service is unavailable.

## TS-012 — Trusted CI policy ingestion (deferred)

- [ ] Define authenticated CI provider identities and webhook verification.
- [ ] Bind check results to immutable commit SHA and repository identity.
- [ ] Define replay protection, idempotency and stale-result handling.
- [ ] Define required-check policy and fail-closed behaviour.
- [ ] Require explicit approval for any protected deployment action.

---

## Review checklist for this backlog

A reviewer should confirm:

- [ ] TS-001 and TS-005 describe the observed risks accurately without assuming unverified behaviour.
- [ ] The 128-in-flight figure is treated as an evidence gap, not a proven capacity threshold.
- [ ] Resource accounting includes shared spool consumers, concurrent reservations and crash cleanup.
- [ ] Test plans are deterministic where possible and distinguish CI from load/staging evidence.
- [ ] Production and paid infrastructure work remains approval-gated.
- [ ] Acceptance criteria are measurable and do not equate passing CI with production readiness.
- [ ] No agent, autonomous merge or deployment feature is pulled ahead of the Git foundation gates.

## Status reporting format

For each work item, report:

- **State:** Proposed / In progress / Blocked / Ready for review / Done
- **Commit:** exact SHA
- **Scope:** files and components changed
- **Tests:** exact commands and pass/fail results
- **Runtime evidence:** environment, workload, measurements and artifacts
- **Residual risks:** explicitly listed
- **Decision:** accepted, rejected or blocked, with reviewer and date

A work item is not Done solely because code was merged or CI passed.
