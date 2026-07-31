# Fixture index

This directory is populated by the W1.2 manual captures against two pinned
oracles:

- `RealSqlServer/` — captures taken against the pinned Azure SQL Edge
  container (`mcr.microsoft.com/azure-sql-edge:latest`, port 11433 — see
  `AGENTS.md`).
- `vscode-mssql-1.45.0/` — captures of the JSON-RPC requests the vscode-mssql
  1.45.0 extension sends to its bundled SqlToolsService.

Each fixture lives at `fixtures/capture/<oracle>/<area>/<sub_area>/<case>.json`
and on completion contains `{"request": ..., "expected": ...}`.

The pinned-status defaults (i.e. the expected `Status` for each sub-area at the
time the first manual capture was taken) are:

| Area / Sub-area | Expected status today | Source |
|---|---|---|
| `sqltoolsservice/connection` | `subset` | `iridium_server` login + TLS path exists; integrated auth missing |
| `sqltoolsservice/object_explorer` | `subset` | T-SQL probe replay works; vscode-mssql JSON-RPC oracle not yet wired |
| `sqltoolsservice/query_execution` | `subset` | Basic batch runs |
| `sqltoolsservice/query_cancel` | `broken` | W2.1 not yet lander; ATTENTION is silently log-and-drop |
| `sqltoolsservice/edit_data` | `subset` | Simple tables |
| `sqltoolsservice/table_designer` | `shim` | W3.2 not yet landed |
| `sqltoolsservice/schema_compare` | `unsupported` | DacFx blocked; intentionally not attempted |
| `sqltoolsservice/profiler` | `unsupported` | XEvents missing; intentionally not attempted |
| `sqltoolsservice/disaster_recovery` | `unsupported` | Backup/restore intentionally rejected per `phase7_admin_classification.rs` |
| `sqlclient/result_metadata` | `shim` | W2.3 not yet landed; COLMETADATA collapses many types |
| `sqlclient/parameters` | `unsupported` | W2.5 not yet landed; `sp_describe_*` rejected |
| `sqlclient/multiple_results` | `subset` | Multi-result batches work; token sequencing may diverge |
| `sqlclient/cancellation` | `broken` | See `query_cancel` |
| `sqlclient/bulk_copy` | `shim` | BULK_INSERT exists but converts rows to individual INSERT statements |

As each P0/P1/P2 item in `docs/plans/vscode-mssql-compatibility-program.md`
lands, the corresponding `expected_status` here is bumped to the new baseline
and the capture is re-pinned.

A machine-readable version of this index will be generated as `index.json`
by the W1.2 capture tooling.
