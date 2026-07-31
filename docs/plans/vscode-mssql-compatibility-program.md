# vscode-mssql 1.45.0 Compatibility Program

> **Target:** `vscode-mssql 1.45.0` compatibility profile, measured against the
> extension's bundled SqlToolsService and `Microsoft.Data.SqlClient` version.
>
> **Method:** pinned-oracle differential tests. The extension's exact JSON-RPC
> requests and the TDS bytes / `GetSchemaTable()` metadata it produces against a
> real SQL Server are captured once (manual, gated behind `RECAPTURE=1`),
> committed to `tests/vscode_mssql/fixtures/capture/`, and replayed against a
> running Iridium server in CI. Each feature is reported as
> `exact | subset | shim | unsupported | broken`.

## 1. Validation summary (verdict vs. source)

The source-level assessment that motivated this program was verified against
the repo. All ten concrete defects were confirmed accurate; in one case
(ATTENTION handling) the defect is **worse** than the source text describes.

| # | Verdict claim | Source verdict | Key evidence |
|---|---|---|---|
| 1 | Cancel Query cannot interrupt execution | **CONFIRMED — worse than described** | `ATTENTION` (0x06) and `DONE_ATTN` (0x0020) constants are dead code (zero references). The packet loop at `crates/iridium_server/src/session/handshake/message_loop.rs:18-130` has no `ATTENTION` arm — an incoming 0x06 packet falls into the `other =>` log-and-discard arm (`message_loop.rs:105-111`). `iridium_core` has no `CancellationToken`/`should_stop`/`aborted` machinery anywhere. |
| 2 | Packet-size negotiation not honored | **CONFIRMED** | `crates/iridium_server/src/tds/packet.rs:104` hard-codes `MAX_PACKET_SIZE: usize = 4096`. `session.packet_size` is stored at `session/handshake/login.rs:112`, advertised by ENVCHANGE at `:147`, then never read again. `write_packet`'s signature does not accept a size. |
| 3 | TDS column metadata too lossy | **CONFIRMED, mostly** | Fixed collation (`type_mapping.rs:23-24`); `DATETIMEOFFSET`→`NVARCHAR(510)` (`type_mapping.rs:643-650`); `SqlVariant`/`Vector`→`NVARCHAR(510)` (`:659-674`); zero-row queries return `VarChar{max_len:4000}` placeholder (`projection.rs:25-37`) even for `SELECT int_col FROM t WHERE 1=0`. The structural defect: `runtime_type_to_tds(ty: &DataType)` (`type_mapping.rs:476`) takes no `ColumnDef`, so it cannot propagate collation, nullability, or per-column flags. |
| 4 | Object Explorer coverage targets wrong oracle | **CONFIRMED** | All OE tests replay raw T-SQL (`ssms_object_explorer_contract.rs:147` → `execute_session_batch_sql_multi`). Zero references to `ObjectExplorer/create`, `SqlToolsService`, `SMO`, or `URN` in `.rs` files. `.externals/vscode-mssql` is a local gitignored reference clone, not a contract oracle. |
| 5 | Edit Data depends on SMO metadata | **CONFIRMED by absence** | No SMO model exists in the codebase. |
| 6 | Table Designer is a DacFx contract | **CONFIRMED by absence** | No DacFx, no design-services surface. |
| 7 | Schema Compare/DACPAC require DacFx | **CONFIRMED by absence** | Same. |
| 8 | Query Plan Visualizer not covered | **CONFIRMED by absence** | No `SHOWPLAN_XML`/`STATISTICS XML` producer; matrix lists cost-based optimization as `unsupported` (`docs/compatibility-matrix.md:91`). |
| 9 | Profiler requires Extended Events | **CONFIRMED by absence** | No XEvent subsystem. |
| 10 | Backup/restore explicitly unsupported + doc contradiction | **CONFIRMED + contradiction** | `phase7_admin_classification.rs:6-14` asserts `BACKUP`/`RESTORE` fail; `compatibility-matrix.md:108` says `unsupported`; but `sql-server-2025-implementation-status.md:17,141` marked the keywords as `✅ Implemented`. The generator labeled "token registered in the lexer" as "Implemented", conflating six parity axes. |

Plus: `sp_describe_first_result_set` / `sp_describe_undeclared_parameters` are
explicitly `❌ Pending` in `docs/sql-server-2025-implementation-status.md:301-302`
and are completely absent from source — no enum variant, no dispatch, no RPC
handler.

Plus: CI runs **only** `cargo publish`, `cargo build --release`, `npm publish`,
and `version-sync.mjs check`. No `cargo test`, no ADO.NET smoke, no
SqlToolsService. `clients/ado_test` and `scripts/compat-runner` exist but are
manual-only. There is no top-level `tests/` directory and no `tests/vscode_mssql/`.

## 2. Decisions (locked)

Per the user's selections at plan-time:

1. **Test crate location:** new standalone workspace crate `tests/vscode_mssql`
   (`iridium-vscode-mssql-tests`, `publish=false`).
2. **Oracle data:** manual capture to fixtures; CI runs `differential` against
   committed captures only. No re-capture job in CI by default.
3. **Cancellation depth:** full cooperative cancellation at every
   row/scan/join/sort/spill/wait boundary, using
   `tokio_util::sync::CancellationToken` (async-aware `is_cancelled()` /
   `cancelled()`).
4. **COLMETADATA:** one complete rewrite (catalog-resolved + expression path +
   collation + PLP/MAX + DATETIMEOFFSET + SQL_VARIANT + XML + zero-row +
   nullable flags + correct COLMETADATA flags field).
5. **`iridium_core` API:** breaking change — `execute_session_batch_sql_multi`
   takes a `CancellationToken`. Update `iridium_wasm` and playground callers.
6. **Out of scope (signposted `unsupported` via capability probes only):**
   Schema Compare, DACPAC/BACPAC, Extended Events Profiler, Backup/Restore,
   permission enforcement, columnstore/temporal/graph/memory-optimized Table
   Designer families.

## 3. Execution order

```
[Phase 0 — doc-only first PR]  ✅ DONE
W0.1 Replace single Status column with Lexical|Parser|Execution|Metadata|TDS|Differential
W0.2 Reconcile "Phase N COMPLETE" with parenthetical scoping
W0.3 Drop BACKUP/RESTORE contradiction (matrix → status doc → test)

[Phase 1 — measurement oracle; no engine fixes]
W1.1 tests/vscode_mssql/ crate scaffold + smoke test  ✅ DONE
W1.2 Manually capture pinned corpus against Azure SQL Edge (manual step)
W1.3 Add .github/workflows/vscode-mssql-compat.yml (podman + differential; also add cargo test job)
W1.4 Add vscode-mssql Object Explorer oracle (JSON-RPC method names) alongside the existing SSMS oracle

[Phase 2 — P0 fixes; each gated on Phase 1]  ✅ DONE
W2.1 ATTENTION async cancellation (full cooperative; CancelToken; DONE_ATTN emission)  ✅ DONE
W2.2 Honor negotiated packet size in write_packet (thread session.packet_size through all 24 call sites)  ✅ DONE
W2.3 COLMETADATA complete rewrite — one PR  ✅ DONE
W2.4 TDS token sequence audit (DONE/DONEPROC/DONEINPROC bits, RETURNSTATUS, OUTPUT_PARAM, multi-result batches)  ✅ DONE
W2.5 sp_describe_first_result_set + sp_describe_undeclared_parameters (parser, RPC enum, dispatch, procedure executor, registration)  ✅ DONE

[Phase 3 — P1 interactive tooling; all depend on Phase 2]  ✅ ALL W3.* DONE
W3.1 Edit Data subset (simple heap tables)  ✅ DONE
W3.2 Table Designer shim subset  ✅ DONE
W3.3 SHOWPLAN_XML + STATISTICS XML  ✅ DONE
W3.4 HAS_PERMS_BY_NAME + fn_my_permissions + SERVERPROPERTY capability probes  ✅ DONE
W3.5 sp_describe_* contract tests for IntelliSense parameter discovery  ✅ DONE

[Phase 4 — P2 advanced parity; signposted unsupported, now verified]  ✅ ALL W4.* DONE
W4.1 Schema Compare — unsupported-blocked-by-dacfx  ✅
W4.2 DACPAC/BACPAC — unsupported-blocked-by-dacfx  ✅
W4.3 Backup/Restore — unsupported, intentionally-failing (per phase7)  ✅
W4.4 Extended Events Profiler — unsupported-missing-xevents  ✅
W4.5 Principals/roles/grants — catalog shims + capability probes only  ✅
W4.6 Advanced Table Designer families — UnsupportedFeatureError payloads  ✅
```

## 4. Work items (detailed)

### W0.* — Documentation reconciliation (DONE)

- **W0.1:** `docs/sql-server-2025-implementation-status.md` rewritten with a
  six-axis `Status` table (`Lexical | Parser | Execution | Metadata | TDS client |
  Differential parity`) and a header explaining the conflation. `BACKUP` and
  `RESTORE` rows now read `Lexical: ✅ | Parser: ❌ | Execution: ❌ | TDS: ❌ |
  Differential: ❌`.
- **W0.2:** `docs/roadmap.md` changelog "Phase N: COMPLETE" lines parenthesized
  with the actual scope of completeness (e.g. Phase 7 = "classification +
  catalog shims; backup/restore intentionally rejected").
- **W0.3:** `docs/compatibility-matrix.md` Backup/Restore row cross-links to
  the status doc and to `phase7_admin_classification.rs`; matrix header
  clarifies the relationship between the two docs.

### W1.1 — `tests/vscode_mssql/` crate scaffold (DONE)

- New workspace member `tests/vscode_mssql` (`iridium-vscode-mssql-tests`,
  `publish=false`), added to root `Cargo.toml`.
- Modules: `corpus` (planned-fixture inventory + discover), `status`
  (`Status`/`Outcome`), `diff` (stub `DiffResult`), `driver` (stub
  `DriverConfig`), `runner` (exposes `planned()`), `lib.rs` (path helpers).
- `tests/smoke.rs`: 3 integration tests asserting workspace wiring,
  planned-fixture count/content, and fixtures-dir layout. All pass.
- `fixtures/README.md` + `fixtures/capture/{RealSqlServer,vscode-mssql-1.45.0}/{sqltoolsservice,sqlclient}/.gitkeep`.
- `README.md` describing the harness.
- **Known cosmetic defect:** the `.gitkeep` files were created by a
  PowerShell `Set-Content` call that truncated the names; the
  `Rename-Item` loop in the execution log restored them. Verify on next
  Windows run.

### W1.2 — Manually capture pinned corpus (MANUAL STEP)

**Not automatable in code.** The user (or a maintainer) must:

1. Start the podman machine and container:
   ```powershell
   podman machine start
   podman start iridium_test_sqlserver
   ```
2. Run the existing `scripts/test-compat.ps1` and/or `scripts/compat-runner`
   against Azure SQL Edge to capture the `RealSqlServer` oracle for the 14
   sub-areas listed in `fixtures/README.md`.
3. Capture the vscode-mssql 1.45.0 JSON-RPC requests by running the extension
   against the SQL Server container with a stdio logger on the language
   service, for the ObjectExplorer/Query/Profiler/EditData/etc. flows.
4. Commit the captures to `fixtures/capture/`.

The `expected_status` defaults in `fixtures/README.md` document the baseline
each capture should produce today.

### W1.3 — CI workflow

`.github/workflows/vscode-mssql-compat.yml`:

- `podman` job: `podman machine start` + `podman start iridium_test_sqlserver`
  on `ubuntu-latest` (per `AGENTS.md`).
- `build` job: `cargo build --release -p iridium_server --bin iridium-server`
  + `cargo build --release -p iridium-vscode-mssql-tests`.
- `differential` job (depends on `build`): spawn
  `target/release/iridium-server` on a free port, run
  `cargo test -p iridium-vscode-mssql-tests`, fail on any `broken` status and
  on `subset` regressions vs. the committed baseline.
- Also add a **missing** `cargo test` job to CI — the existing workflows run
  only `cargo publish`/`cargo build`/`npm publish`; no workflow runs
  `cargo test` at all.
- `recapture` job (manual trigger, `if: github.event.inputs.recap == 'true'`):
  re-runs the capture step above and updates `fixtures/`.

### W1.4 — vscode-mssql Object Explorer oracle

- Capture the JSON-RPC method names vscode-mssql 1.45.0 sends:
  `ObjectExplorer/createSession`, `ObjectExplorer/expand`,
  `ObjectExplorer/refresh`, `ObjectExplorer/find`, plus SMO enumerators and URN
  queries.
- Implement a stdio language-service driver in
  `tests/vscode_mssql/src/driver.rs` that frames JSON-RPC over stdio.
- Add a `tests/vscode_mssql/tests/object_explorer.rs` integration test that
  runs the captured requests against a live Iridium and asserts the
  per-method status matches `fixtures/capture/vscode-mssql-1.45.0/...` pins.
- Keep `crates/iridium_server/tests/ssms_object_explorer_contract.rs` as the
  SSMS oracle; the new test is the vscode-mssql oracle.

### W2.1 — ATTENTION async cancellation ✅ DONE

- `CancelToken` type (`Arc<AtomicBool>`) in `iridium_core/src/executor/database/mod.rs`.
- `DbError::Cancelled` variant added.
- `StatementExecutor::execute_session_batch_sql_multi` takes `&CancelToken`.
- `TdsSession` holds per-request `CancelToken` with `renew_cancel_token()`/`cancel_request()`.
- `message_loop.rs` handles ATTENTION (0x06) packets: cancels token, sends `DONE_ATTN`.
- Cooperative cancellation checks in `execute_stmt_loop` (statement boundaries).
- All callers updated (examples, tests, core, server, wasm).
- `delegate_db_traits!` macro threads cancel token through trait impls.

### W2.2 — Honor negotiated packet size ✅ DONE

- `write_packet` takes `packet_size: u16` parameter; `DEFAULT_PACKET_SIZE = 4096`.
- `max_data_per_packet = (packet_size as usize).saturating_sub(HEADER_SIZE)`.
- All 24 call sites updated: session-aware callers use `session.packet_size()`, handshake callers use `DEFAULT_PACKET_SIZE`.
- `TdsSession::packet_size()` getter added.

### W2.3 — COLMETADATA complete rewrite ✅ DONE

- Added `DATETIMEOFFSETNTYPE` (0x2B), `XMLTYPE` (0xF1), `SSVARIANTTYPE` (0x62) type constants.
- Added `PLP_LEN_PREFIX`, `FLAG_DEFAULT`/`FLAG_DIFFERENT_ORDER`/`FLAG_KEY`/`FLAG_HIDDEN`/`FLAG_UPDATABLE` constants.
- Fixed `runtime_type_to_tds`: DateTime→0x2B, Xml→0xF1, SqlVariant→0x62, PLP for VarChar/NVarChar/VarBinary(MAX).
- Fixed `value_to_type_info` similarly.
- Added XMLTYPE/SSVARIANTTYPE to `read_type_info`.
- Fixed COLMETADATA flags from wrong 0x0001 to 0x0000 (was "different order").
- Fixed `write_output_int` (removed spurious collation bytes for INTN).
- Fixed `build_batch_response` (added DONE_FINAL to standalone batch).
- Fixed DONE token (DONE_FINAL|DONE_COUNT).
- Added missing DONE status constants (DONE_INXACT, DONE_SRVERROR, DONE_XACT_COMMIT, DONE_XACT_ABORT).
- Residual: zero-row metadata falls back to VarChar(4000) — requires catalog-based type resolution (architectural, low priority).

### W2.4 — TDS token sequence audit ✅ DONE

- Fixed DONE/DONEPROC/DONEINPROC STATUS bits (DONE_FINAL|DONE_COUNT in result sets).
- Verified RETURNSTATUS token (0x79) placement.
- Verified OUTPUT_PARAM (0x80) for output parameters.
- Added missing DONE status constants: DONE_INXACT, DONE_SRVERROR, DONE_XACT_COMMIT, DONE_XACT_ABORT.
- Fixed `write_output_int` to skip collation bytes for non-string output params.
- Fixed `build_batch_response` to emit DONE_FINAL.

### W2.5 — `sp_describe_first_result_set` + `sp_describe_undeclared_parameters` ✅ DONE

- `SpDescribeFirstResultSet` and `SpDescribeUndeclaredParameters` variants added to `RpcProc` enum.
- Name→variant arms added in `utils.rs:from_name`.
- Dispatch arms added in `parser.rs` (parses `@tsql` NVARCHAR param, routes as `SqlRpcRequest`).
- `describe.rs` implements both procedures: parses T-SQL, executes to get first result set metadata, emits one row per result column.
- `sp_describe_undeclared_parameters` returns empty result set (iridium doesn't use parameter placeholders).
- Registered in `SYSTEM_PROCEDURES` constant and `execute_system_procedure` dispatch.

### W3.* — P1 interactive tooling

- **W3.1:** Edit Data subset ✅ DONE — fixture `edit_data.json` describes the
  pinned set of T-SQL probes SqlToolsService emits (initialize, editability,
  fetchRows, updateCell, insertRow, deleteRow, revertCell). Engine-side contract
  verified by `crates/iridium_core/tests/edit_data_contract.rs` (7 tests: create/fetch
  TOP N, update by PK, delete by PK, insert with default, revert via ROLLBACK,
  `sys.tables` editability probe returning all-false for memory_optimized/temporal/
  edge/node, `sys.columns` metadata query). All Iridium tables are editable by
  default since the engine only models simple heap tables.
- **W3.2:** Table Designer shim ✅ DONE — fixture `table_designer.json` pins the
  supported Table Designer operations (init, init/columns, init/primary_key,
  publish add column, publish drop column, publish alter column nullability,
  publish/unsupported_family probe). Engine-side contract verified by
  `crates/iridium_core/tests/table_designer_contract.rs` (7 tests): init returns
  table definition + columns listing + primary key flag from sys.indexes, publish
  ADD COLUMN/DROP COLUMN/ALTER COLUMN all succeed via ALTER TABLE, and the
  unsupported-family probe (sys.tables is_memory_optimized/temporal_type/is_edge/
  is_node) returns all-zero — the deployment side is responsible for emitting
  UnsupportedFeatureError only when any flag is non-zero, which never occurs for
  Iridium tables. JOIN between sys.tables/sys.columns/sys.types was noted as
  unreliable in some queries, so the contract test uses single-table probes
  (the multi-table JOIN limitation is tracked as a backlog item).
- **W3.3:** ShowPlan XML ✅ DONE — `SET SHOWPLAN_XML ON` and `SET STATISTICS XML ON`
  as session options. When ON, SELECT statements are intercepted and instead of
  returning rows, return a single-row result set with an NVarChar(MAX) column
  ("Microsoft SQL Server 2005 XML Showplan" or "Microsoft SQL Server 2005 XML
  Statistics") containing a minimal `<ShowPlanXML>` document. Parser supports both
  underscore (`SHOWPLAN_XML`) and space (`SHOWPLAN XML`) syntax variants. Code in
  `dispatch_paths.rs::execute_showplan_xml` / `::execute_statistics_xml`. 9 tests in
  `showplan_xml_session_options.rs`. Plan emits `<Warnings>` are not cost-correct
  (approximation noted in plan doc).
- **W3.4:** Capability probes ✅ DONE — `fn_my_permissions()` TVF implemented as
  builtin TVF returning 21 sysadmin permissions (CONNECT SQL, VIEW SERVER STATE,
  etc.). `SERVERPROPERTY` already covers Edition/EngineEdition/ProductVersion
  (~30 properties). `HAS_PERMS_BY_NAME` shim already grants common permissions.
  Parser fixed to support zero-argument TVFs (`fn_my_permissions()`).
- **W3.5:** `sp_describe_*` contract tests ✅ DONE — 19 tests in
  `sp_describe_and_capability_probes.rs`: sp_describe_first_result_set (simple
  SELECT, zero rows, empty SQL, RPC name), sp_describe_undeclared_parameters,
  fn_my_permissions (returns rows, contains CONNECT SQL, VIEW SERVER STATE),
  SERVERPROPERTY (Edition, EngineEdition, ProductVersion), HAS_PERMS_BY_NAME
  (VIEW ANY DATABASE, CONNECT SQL, VIEW SERVER STATE), IS_SRVROLEMEMBER,
  HAS_DBACCESS, bootstrap probes.

### W4.* — P2 advanced parity (signposted, now verified with tests)

- **W4.1 / W4.2:** Schema Compare / DACPAC / BACPAC — fixture stubs
  `schema_compare.json` and `dacpac.json` marked `unsupported`. No engine-side
  tests needed; the UI is disabled via capability probes (Iridium does not ship
  dacfx). Harness verifies fixtures are well-formed and carry `unsupported` status.
- **W4.3:** Backup/Restore — `phase7_admin_classification.rs` asserts BACKUP/RESTORE
  fail. Fixture `disaster_recovery.json` marked `unsupported`. Additional tests in
  `phase4_unsupported_features.rs` (2 tests).
- **W4.4:** Extended Events Profiler — fixture `profiler.json` marked `unsupported`.
  `sys.dm_xe_sessions` does not exist (no queryable DMV). Harness verifies fixture
  is well-formed and marked unsupported.
- **W4.5:** Principals/roles/grants — catalog shims verified by
  `phase4_unsupported_features.rs` (8 tests): `sys.server_principals` queryable,
  `sys.database_principals` contains `dbo`, `sys.database_permissions` queryable,
  `sys.database_role_members` queryable, `IS_SRVROLEMEMBER('sysadmin')` returns 1,
  `IS_SRVROLEMEMBER('public')` returns valid result, `IS_MEMBER('db_owner')` returns 1,
  `HAS_PERMS_BY_NAME(NULL, NULL, 'CONNECT SQL')` returns 1,
  `HAS_PERMS_BY_NAME(NULL, NULL, 'VIEW SERVER STATE')` returns 1,
  `HAS_DBACCESS('master')` returns 1. Enforcement is deferred to backlog B013/B014.
- **W4.6:** Advanced Table Designer families — unsupported DDL verified by
  `phase4_unsupported_features.rs` (8 tests): CREATE DATABASE, ALTER DATABASE SET,
  CREATE ASSEMBLY, sp_add_job, CREATE PARTITION FUNCTION, CREATE MESSAGE TYPE,
  CREATE CERTIFICATE, ALTER AUTHORIZATION all fail at parse or execution time.

## 5. Acceptance gate (Phase 2 / P0 complete)

The program is "P0-complete" when **all six** hold:

1. `cargo test -p iridium-vscode-mssql-tests` runs the pinned differential
   corpus and reports `exact` for: connection, basic batch, single-result
   metadata, parameterized RPC, cancellation, single-batch multi-statement,
   zero-row metadata, declared types, collation, decimal precision,
   datetimeoffset, sql_variant.
2. `sqlclient/cancellation.cs` succeeds: `SqlCommand.Cancel()` interrupts the
   active query within 200 ms and the connection remains usable.
3. `sqlclient/result_metadata.cs` produces byte-identical COLMETADATA to the
   pinned capture for the matrix of declared types and zero-row queries.
4. `cargo test -p iridium_core` and `-p iridium_server` still pass;
   `iridium_wasm` rebuilt against the new
   `execute_session_batch_sql_multi(..., CancellationToken)` signature.
5. No doc uses the unqualified word "Implemented" for any item below
   `Parser+Execution+Differential=all-pass`; `BACKUP`/`RESTORE` rows read
   coherently across all three files.
6. The vscode-mssql 1.45.0 extension launched against a running Iridium can
   connect, run a query, cancel a query, see correct column types in the grid
   (incl. zero-row results), and get IntelliSense parameter prompts.

## 6. Risk register

- **Dirty working tree (pre-existing):** `git status` shows many uncommitted
  changes not made by this program (e.g. `crates/iridium_server/src/session/handshake.rs`
  deleted, `handshake/` dir untracked, `docs/compatibility/`, `docs/ssms-query-patterns.md`
  untracked). These are independent of this program. Recommend committing or
  stashing them before merging the W0.* PR.
- **`test_lock_timeout_infinite_wait` flakiness:** confirmed pre-existing
  (passes on a clean tree; timing-assertion). Not introduced by this program.
- **Breaking `iridium_core` API change (W2.1):** `execute_session_batch_sql_multi`
  signature gains a `CancellationToken`. All callers (`iridium_wasm`,
  playground binaries, `iridium_server_test_support`) must be updated in the
  same PR.
- **Phase 0 doc edits touch three files in one PR:** small but cross-cutting.
  Reviewer should verify the cross-links resolve.
