# SQL Server 2025 Implementation Status

This document tracks the implementation status of SQL Server 2025 features in Iridium SQL.

## How to read these tables

Earlier revisions of this document used a single `Status` column whose values
were `✅ Implemented` / `❌ Pending`. That collapsed six distinct parity axes
into one label, which produced contradictions with `docs/compatibility-matrix.md`
and with the executable contract tests.

For example, `BACKUP` and `RESTORE` are registered as token variants in
`crates/iridium_core/src/parser/token/keyword.rs:325-326`, so the lexer
recognizes them and rejects them as identifiers. There is *no* parser
production for the statements, *no* executor handler, and the contract test
`crates/iridium_core/tests/phase7_admin_classification.rs` explicitly asserts
both must fail at execution. The previous table labeled these keywords
`✅ Implemented`, contradicting both `compatibility-matrix.md` (which says
`unsupported`) and the test (which asserts rejection).

To stop that conflation, each row now records status along the six axes that
matter for compatibility:

| Axis | Meaning | Source of truth |
| :--- | :--- | :--- |
| `Lexical` | Token is recognized by the lexer (reserved as a keyword). | `crates/iridium_core/src/parser/token/keyword.rs` |
| `Parser` | A grammar production consumes the keyword into a real AST node. | `crates/iridium_core/src/parser/parse/**` |
| `Execution` | An executor handler turns that AST node into engine behavior. | `crates/iridium_core/src/executor/**` |
| `Metadata` | Catalog / `sys.*` / `INFORMATION_SCHEMA` rows describe the construct. | `crates/iridium_core/src/executor/metadata/**` |
| `TDS client` | The TDS layer exposes the construct to clients (SSMS, ADS, vscode-mssql, …). | `crates/iridium_server/src/tds/**` |
| `Differential parity` | The behavior matches SQL Server under the differential harness. | `scripts/compat-runner/**`, `tests/vscode_mssql/**` |

Refer to `docs/compatibility-matrix.md` for the user-facing status (`exact`,
`compatible subset`, `shim`, `unsupported`). The per-axis rows below tell you
*which axis* is the bottleneck when the matrix says `subset` or `unsupported`.

Cells use:

- `✅` — supported for that axis
- `❌` — not supported
- `—` — not applicable (e.g. `BACKUP` has no `Metadata` axis)
- `📝` — partial / needs verification / pinned in backlog

## Reserved Keywords (Section 1)

| Keyword | Lexical | Parser | Execution | Metadata | TDS client | Differential | Notes |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| ADD | ✅ | ✅ | ✅ | — | ✅ | 📝 | `ALTER TABLE … ADD` column |
| ALL | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ALTER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| AND | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ANY | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| AS | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ASC | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| AUTHORIZATION | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| BACKUP | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexically reserved only. `phase7_admin_classification.rs` asserts execution fails. See `compatibility-matrix.md` row "Backup / restore". |
| BEGIN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| BETWEEN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| BREAK | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| BROWSE | ✅ | ❌ | ❌ | — | ❌ | ❌ | Reserved only; T-SQL `BROWSE FOR` is not implemented. |
| BULK | ✅ | ✅ | 📝 | — | 📝 | 📝 | `BULK INSERT` is a shim that converts rows into individual `INSERT` statements; no native bulk load path. See `compatibility-matrix.md` "Flat-file import". |
| BY | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CASCADE | ✅ | ✅ | ✅ | 📝 | ✅ | 📝 | |
| CASE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CHECK | ✅ | ✅ | ✅ | 📝 | ✅ | 📝 | `CHECK` constraints parsed and enforced for new rows; metadata visibility is partial. |
| CHECKPOINT | ✅ | ✅ | ✅ | — | ✅ | 📝 | WAL/checkpoint engine has its own checkpoint semantics; not 1:1 with SQL Server's `CHECKPOINT`. |
| CLOSE | ✅ | ✅ | ✅ | — | ✅ | 📝 | Cursors. |
| CLUSTERED | ✅ | ✅ | 📝 | ✅ | 📝 | 📝 | Index cluster flag parsed and cataloged; storage engine is not B-tree clustered. |
| COALESCE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| COLLATE | ✅ | 📝 | 📝 | 📝 | 📝 | ❌ | Parser accepts the clause; the actual collation is *not* enforced and *not* propagated to the wire (COLMETADATA uses a fixed `Latin1_General_CI_AS`). See `compatibility-matrix.md` "Full collation and type-fidelity behavior". |
| COLUMN | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| COMMIT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| COMPUTE | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; T-SQL `COMPUTE BY` is not implemented. |
| CONSTRAINT | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| CONTAINS | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; full-text not implemented. |
| CONTAINSTABLE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only; full-text not implemented. |
| CONTINUE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CONVERT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CREATE | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| CROSS | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CURRENT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CURRENT_DATE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CURRENT_TIME | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CURRENT_TIMESTAMP | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CURRENT_USER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| CURSOR | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DATABASE | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| DBCC | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| DEALLOCATE | ✅ | ✅ | ✅ | — | ✅ | 📝 | Cursors. |
| DECLARE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DEFAULT | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| DELETE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DENY | ✅ | ❌ | ❌ | 📝 | ❌ | ❌ | Lexer-only; permission enforcement is not implemented. Catalog shim rows exist for `sys.database_permissions`. See `compatibility-matrix.md` "Principals, roles, grants, deny, revoke". |
| DESC | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DISK | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; `DISK` device commands are not implemented. |
| DISTINCT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DISTRIBUTED | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DOUBLE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| DROP | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| DUMP | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| ELSE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| END | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ERRLVL | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| ESCAPE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| EXCEPT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| EXEC | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| EXECUTE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| EXISTS | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| EXIT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only (control-flow `EXIT` not implemented; recognized for compatibility lexical reservation). |
| EXTERNAL | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; external assemblies / SQLCLR not implemented. |
| FETCH | ✅ | ✅ | ✅ | — | ✅ | 📝 | Cursors. |
| FILE | ✅ | ❌ | ❌ | 📝 | ❌ | ❌ | Lexer-only. `sys.database_files` partial metadata exists. |
| FILLFACTOR | ✅ | ✅ | 📝 | ✅ | 📝 | 📝 | Index option parsed and cataloged; not honored by the BTree storage layer. |
| FOR | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| FOREIGN | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| FREETEXT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; full-text. |
| FREETEXTTABLE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only; full-text. |
| FROM | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| FULL | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| FUNCTION | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | Subset of T-SQL function features. |
| GOTO | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| GRANT | ✅ | ❌ | ❌ | 📝 | ❌ | ❌ | Lexer-only; see `DENY` row. |
| GROUP | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| HAVING | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| HOLDLOCK | ✅ | ✅ | 📝 | — | 📝 | 📝 | Hint accepted; isolation mapping is partial. |
| IDENTITY | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| IDENTITYCOL | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| IDENTITY_INSERT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| IF | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| IN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| INDEX | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| INNER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| INSERT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| INTERSECT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| INTO | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| IS | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| JOIN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| KEY | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| KILL | ✅ | ✅ | 📝 | — | ✅ | 📝 | Subset: `KILL <spid>` is recognized; `KILL STATS`/`KILL UOW` are stubs. |
| LEFT | ✅ | ✅ | ✅ | — | ✅ | 📝 | String `LEFT` and join `LEFT`. |
| LIKE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| LINENO | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| LOAD | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| MERGE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| NATIONAL | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| NOCHECK | ✅ | 📝 | 📝 | ✅ | 📝 | 📝 | Recognized in `ALTER TABLE … WITH NOCHECK`; full constraint-skip semantics not complete. |
| NONCLUSTERED | ✅ | ✅ | 📝 | ✅ | 📝 | 📝 | Like `CLUSTERED`. |
| NOT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| NULL | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| NULLIF | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| OF | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| OFF | ✅ | ✅ | ✅ | — | ✅ | 📝 | SET option `... OFF`. |
| OFFSETS | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| ON | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| OPEN | ✅ | ✅ | ✅ | — | ✅ | 📝 | Cursors. |
| OPENDATASOURCE | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; linked servers not implemented. |
| OPENQUERY | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; linked servers not implemented. |
| OPENROWSET | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| OPENXML | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| OPTION | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; query hints not implemented. |
| OR | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ORDER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| OUTER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| OVER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| PERCENT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; `TOP … PERCENT` not implemented. |
| PIVOT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| PLAN | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; `SET SHOWPLAN` etc. not implemented (see `tests/vscode_mssql` track W3.3). |
| PRECISION | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| PRIMARY | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| PRINT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| PROC | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| PROCEDURE | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| PUBLIC | ✅ | ❌ | ❌ | 📝 | ❌ | ❌ | Lexer-only; security role `PUBLIC` is not enforced. |
| RAISERROR | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| READ | ✅ | 📝 | 📝 | — | 📝 | 📝 | Recognized in `WITH (READ*)` hints; isolation mapping is partial. |
| READTEXT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| RECONFIGURE | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| REFERENCES | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| REPLICATION | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; replication not implemented. |
| RESTORE | ✅ | ❌ | ❌ | — | ❌ | ❌ | Same as `BACKUP` — lexically reserved only; `phase7_admin_classification.rs` asserts execution fails. |
| RESTRICT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| RETURN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| REVERT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; `EXECUTE AS REVERT` not implemented. |
| REVOKE | ✅ | ❌ | ❌ | 📝 | ❌ | ❌ | Lexer-only; see `DENY`. |
| RIGHT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ROLLBACK | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ROWCOUNT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| ROWGUIDCOL | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only. |
| RULE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only; legacy `RULE` objects not implemented. |
| SAVE | ✅ | ✅ | ✅ | — | ✅ | 📝 | Savepoints. |
| SCHEMA | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| SECURITYAUDIT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| SELECT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| SEMANTICKEYPHRASETABLE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only; semantic search not implemented. |
| SEMANTICSIMILARITYDETAILSTABLE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only; semantic search not implemented. |
| SEMANTICSIMILARITYTABLE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Lexer-only; semantic search not implemented. |
| SESSION_USER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| SET | ✅ | ✅ | ✅ | — | ✅ | 📝 | Subset of SET options; unsupported options fall through to `DbError::Unsupported` per backlog `B020`. |
| SETUSER | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| SHUTDOWN | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; shutdown is not implemented. The previous table labeled this `✅ Implemented` on the basis of token recognition alone, which is the same conflation noted for `BACKUP`/`RESTORE`. |
| SOME | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| STATISTICS | ✅ | 📝 | 📝 | — | ❌ | ❌ | `STATISTICS IO`/`STATISTICS TIME` are accepted; `STATISTICS XML` is not produced (see `tests/vscode_mssql` track W3.3). |
| SYSTEM_USER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TABLE | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| TABLESAMPLE | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| TEXTSIZE | ✅ | ✅ | 📝 | — | ✅ | 📝 | Parsed; effect is not enforced on routing text-size clamp. |
| THEN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TO | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TOP | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TRAN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TRANSACTION | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TRIGGER | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| TRUNCATE | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| TRY_CONVERT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| TSEQUAL | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| UNION | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| UNIQUE | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| UNPIVOT | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| UPDATE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| UPDATETEXT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |
| USE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| USER | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| VALUES | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| VARYING | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| VIEW | ✅ | ✅ | ✅ | ✅ | ✅ | 📝 | |
| WAITFOR | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only; wait semantics not implemented. |
| WHEN | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| WHERE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| WHILE | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| WITH | ✅ | ✅ | ✅ | — | ✅ | 📝 | |
| WITHIN | ✅ | ✅ | ✅ | — | ✅ | 📝 | XML/JSON path. |
| WRITETEXT | ✅ | ❌ | ❌ | — | ❌ | ❌ | Lexer-only. |

**Summary:** Lexical 185/185 (100%). Parser/Execution/TDS axes vary row-by-row; see the matrix for the usable-surface summary. Treat the previous "134/185 (72.4%) Implemented" summary as withdrawn — it was the conflation this revision corrects.

## System Stored Procedures (`sp_*`)

| Procedure | Status |
| :--- | :--- |
| sp_OACreate | ❌ Pending |
| sp_OADestroy | ❌ Pending |
| sp_OAGetErrorInfo | ❌ Pending |
| sp_OAGetProperty | ❌ Pending |
| sp_OAMethod | ❌ Pending |
| sp_OASetProperty | ❌ Pending |
| sp_OAStop | ❌ Pending |
| sp_add_alert | ❌ Pending |
| sp_add_category | ❌ Pending |
| sp_add_data_file_recover_suspect_db | ❌ Pending |
| sp_add_job | ❌ Pending |
| sp_add_jobschedule | ❌ Pending |
| sp_add_jobserver | ❌ Pending |
| sp_add_jobstep | ❌ Pending |
| sp_add_maintenance_plan | ❌ Pending |
| sp_add_maintenance_plan_db | ❌ Pending |
| sp_add_maintenance_plan_job | ❌ Pending |
| sp_add_notification | ❌ Pending |
| sp_add_operator | ❌ Pending |
| sp_add_proxy | ❌ Pending |
| sp_add_schedule | ❌ Pending |
| sp_add_targetservergroup | ❌ Pending |
| sp_add_targetsvrgrp_member | ❌ Pending |
| sp_addapprole | ❌ Pending |
| sp_addaudit | ❌ Pending |
| sp_addauditaccess | ❌ Pending |
| sp_addextendedproc | ❌ Pending |
| sp_addextendedproperty | ❌ Pending |
| sp_addgroup | ❌ Pending |
| sp_addlinkedserver | ❌ Pending |
| sp_addlinkedsrvlogin | ❌ Pending |
| sp_addlogin | ❌ Pending |
| sp_addmessage | ❌ Pending |
| sp_addremotelogin | ❌ Pending |
| sp_addrole | ❌ Pending |
| sp_addrolemember | ❌ Pending |
| sp_addserver | ❌ Pending |
| sp_addservrolemember | ❌ Pending |
| sp_addsrvrolemember | ❌ Pending |
| sp_addtype | ❌ Pending |
| sp_adduser | ❌ Pending |
| sp_apply_job_to_targets | ❌ Pending |
| sp_approlepassword | ❌ Pending |
| sp_attach_schedule | ❌ Pending |
| sp_audit_write | ❌ Pending |
| sp_bindefault | ❌ Pending |
| sp_bindrule | ❌ Pending |
| sp_bindsession | ❌ Pending |
| sp_catalogs | ❌ Pending |
| sp_cdc_add_job | ❌ Pending |
| sp_cdc_change_job | ❌ Pending |
| sp_cdc_cleanup_change_table | ❌ Pending |
| sp_cdc_disable_db | ❌ Pending |
| sp_cdc_disable_table | ❌ Pending |
| sp_cdc_drop_job | ❌ Pending |
| sp_cdc_enable_db | ❌ Pending |
| sp_cdc_enable_table | ❌ Pending |
| sp_cdc_generate_wrapper_function | ❌ Pending |
| sp_cdc_get_captured_columns | ❌ Pending |
| sp_cdc_get_ddl_history | ❌ Pending |
| sp_cdc_help_change_data_capture | ❌ Pending |
| sp_cdc_scan | ❌ Pending |
| sp_certify_removable | ❌ Pending |
| sp_change_users_login | ❌ Pending |
| sp_clean_db_file_free_space | ❌ Pending |
| sp_clean_db_free_space | ❌ Pending |
| sp_column_privileges | ❌ Pending |
| sp_columns_ex | ❌ Pending |
| sp_configure | ❌ Pending |
| sp_control_plan_guide | ❌ Pending |
| sp_createstats | ❌ Pending |
| sp_cycle_agent_errorlog | ❌ Pending |
| sp_databases | ✅ Implemented |
| sp_db_increased_partitions | ❌ Pending |
| sp_dbmmonitoraddmonitoring | ❌ Pending |
| sp_dbmmonitorchangealert | ❌ Pending |
| sp_dbmmonitorchangemonitoring | ❌ Pending |
| sp_dbmmonitordropmonitoring | ❌ Pending |
| sp_dbmmonitorhelpmonitoring | ❌ Pending |
| sp_defaultdb | ❌ Pending |
| sp_defaultlanguage | ❌ Pending |
| sp_delete_alert | ❌ Pending |
| sp_delete_backuphistory | ❌ Pending |
| sp_delete_category | ❌ Pending |
| sp_delete_database_backuphistory | ❌ Pending |
| sp_delete_job | ❌ Pending |
| sp_delete_jobhistory | ❌ Pending |
| sp_delete_jobserver | ❌ Pending |
| sp_delete_jobstep | ❌ Pending |
| sp_delete_maintenance_plan | ❌ Pending |
| sp_delete_maintenance_plan_db | ❌ Pending |
| sp_delete_maintenance_plan_job | ❌ Pending |
| sp_delete_notification | ❌ Pending |
| sp_delete_operator | ❌ Pending |
| sp_delete_proxy | ❌ Pending |
| sp_delete_schedule | ❌ Pending |
| sp_delete_targetservergroup | ❌ Pending |
| sp_delete_targetsvrgrp_member | ❌ Pending |
| sp_denylogin | ❌ Pending |
| sp_depends | ❌ Pending |
| sp_describe_first_result_set | ❌ Pending |
| sp_describe_undeclared_parameters | ❌ Pending |
| sp_detach_schedule | ❌ Pending |
| sp_dropextendedproc | ❌ Pending |
| sp_dropextendedproperty | ❌ Pending |
| sp_droplinkedsrvlogin | ❌ Pending |
| sp_droplogin | ❌ Pending |
| sp_dropmessage | ❌ Pending |
| sp_dropremotelogin | ❌ Pending |
| sp_droprole | ❌ Pending |
| sp_droprolemember | ❌ Pending |
| sp_dropserver | ❌ Pending |
| sp_droptype | ❌ Pending |
| sp_dropuser | ❌ Pending |
| sp_enum_errorlogs | ❌ Pending |
| sp_enum_sqlagent_subsystems | ❌ Pending |
| sp_enumcustomresolvers | ❌ Pending |
| sp_execute_external_script | ❌ Pending |
| sp_flush_CT_internal_table_on_demand | ❌ Pending |
| sp_foreignkeys | ❌ Pending |
| sp_fulltext_catalog | ❌ Pending |
| sp_fulltext_database | ❌ Pending |
| sp_fulltext_keymappings | ❌ Pending |
| sp_fulltext_load_thesaurus_file | ❌ Pending |
| sp_fulltext_pendingchanges | ❌ Pending |
| sp_fulltext_service | ❌ Pending |
| sp_fulltextdatabase | ❌ Pending |
| sp_getDiagnosticData | ❌ Pending |
| sp_get_composite_job_info | ❌ Pending |
| sp_get_sqlagent_properties | ❌ Pending |
| sp_getagentparameterlist | ❌ Pending |
| sp_getapplock | ❌ Pending |
| sp_getbindtoken | ❌ Pending |
| sp_grantdbaccess | ❌ Pending |
| sp_grantlogin | ❌ Pending |
| sp_help | ✅ Implemented |
| sp_help_alert | ❌ Pending |
| sp_help_category | ❌ Pending |
| sp_help_downloadlist | ❌ Pending |
| sp_help_fulltext_catalogs | ❌ Pending |
| sp_help_fulltext_catalogs_cursor | ❌ Pending |
| sp_help_fulltext_columns | ❌ Pending |
| sp_help_fulltext_tables | ❌ Pending |
| sp_help_job | ❌ Pending |
| sp_help_jobactivity | ❌ Pending |
| sp_help_jobcount | ❌ Pending |
| sp_help_jobhistory | ❌ Pending |
| sp_help_jobschedule | ❌ Pending |
| sp_help_jobserver | ❌ Pending |
| sp_help_jobstep | ❌ Pending |
| sp_help_jobsteplog | ❌ Pending |
| sp_help_maintenance_plan | ❌ Pending |
| sp_help_maintenance_plan_db | ❌ Pending |
| sp_help_maintenance_plan_job | ❌ Pending |
| sp_help_operator | ❌ Pending |
| sp_help_proxy | ❌ Pending |
| sp_help_schedule | ❌ Pending |
| sp_help_stoplists | ❌ Pending |
| sp_help_targetserver | ❌ Pending |
| sp_help_targetservergroup | ❌ Pending |
| sp_help_targetsvrgrp_member | ❌ Pending |
| sp_helpconstraint | ✅ Implemented |
| sp_helpdb | ✅ Implemented |
| sp_helpdevice | ❌ Pending |
| sp_helpdownloadlist | ❌ Pending |
| sp_helpextendedproc | ❌ Pending |
| sp_helpfile | ✅ Implemented |
| sp_helpfilegroup | ✅ Implemented |
| sp_helpgroup | ❌ Pending |
| sp_helpindex | ✅ Implemented |
| sp_helplinkedsrvlogin | ❌ Pending |
| sp_helplogins | ❌ Pending |
| sp_helpremotelogin | ❌ Pending |
| sp_helpremotelogin_90 | ❌ Pending |
| sp_helpserver | ❌ Pending |
| sp_helpsrvrole | ✅ Implemented |
| sp_helpsrvrolemember | ✅ Implemented |
| sp_helptext_jobstep | ❌ Pending |
| sp_helpuser | ✅ Implemented |
| sp_indexes | ❌ Pending |
| sp_kill_filestream_non_transacted_handles | ❌ Pending |
| sp_link_publication | ❌ Pending |
| sp_linkedservers | ❌ Pending |
| sp_maintplan_delete_log | ❌ Pending |
| sp_manage_backup_devices | ❌ Pending |
| sp_mapdown_bitmap | ❌ Pending |
| sp_monitor | ✅ Implemented |
| sp_password | ❌ Pending |
| sp_pkeys | ❌ Pending |
| sp_prepare | ❌ Pending |
| sp_prepexec | ❌ Pending |
| sp_prepexecrpc | ❌ Pending |
| sp_primarykeys | ❌ Pending |
| sp_purge_jobhistory | ❌ Pending |
| sp_query_store_consistency_check | ❌ Pending |
| sp_query_store_flush_db | ❌ Pending |
| sp_query_store_force_plan | ❌ Pending |
| sp_query_store_remove_plan | ❌ Pending |
| sp_query_store_remove_query | ❌ Pending |
| sp_query_store_reset_exec_stats | ❌ Pending |
| sp_query_store_unforce_plan | ❌ Pending |
| sp_rda_reauthorize_db | ❌ Pending |
| sp_rda_reconciliation_cleanup | ❌ Pending |
| sp_rda_set_rpo_duration | ❌ Pending |
| sp_recompile | ❌ Pending |
| sp_refresh_parameter_encryption | ❌ Pending |
| sp_remoteoption | ❌ Pending |
| sp_remove_alert | ❌ Pending |
| sp_remove_category | ❌ Pending |
| sp_remove_jobschedule | ❌ Pending |
| sp_remove_jobserver | ❌ Pending |
| sp_remove_jobstep | ❌ Pending |
| sp_remove_notification | ❌ Pending |
| sp_remove_operator | ❌ Pending |
| sp_remove_proxy | ❌ Pending |
| sp_remove_schedule | ❌ Pending |
| sp_remove_targetservergroup | ❌ Pending |
| sp_remove_targetsvrgrp_member | ❌ Pending |
| sp_rename | ✅ Implemented |
| sp_resync_targetserver | ❌ Pending |
| sp_revokedbaccess | ❌ Pending |
| sp_revokelogin | ❌ Pending |
| sp_server_info | ✅ Implemented |
| sp_serveroption | ❌ Pending |
| sp_set_firewall_rule | ❌ Pending |
| sp_set_sqlagent_properties | ❌ Pending |
| sp_set_sqlagent_proxy | ❌ Pending |
| sp_setapprole | ❌ Pending |
| sp_setnetname | ❌ Pending |
| sp_special_columns | ❌ Pending |
| sp_srvrolepermission | ❌ Pending |
| sp_statistics | ❌ Pending |
| sp_stop_job | ❌ Pending |
| sp_syscollector_create_collection_item | ❌ Pending |
| sp_syscollector_create_collection_set | ❌ Pending |
| sp_syscollector_create_collector_type | ❌ Pending |
| sp_syscollector_create_logical_collection_set | ❌ Pending |
| sp_syscollector_delete_collection_item | ❌ Pending |
| sp_syscollector_delete_collection_set | ❌ Pending |
| sp_syscollector_delete_collector_type | ❌ Pending |
| sp_syscollector_execution_stats | ❌ Pending |
| sp_syscollector_flush_aggregate | ❌ Pending |
| sp_syscollector_force_flush | ❌ Pending |
| sp_syscollector_get_trace_info | ❌ Pending |
| sp_syscollector_reinit_collection_set | ❌ Pending |
| sp_syscollector_set_warehouse_database_name | ❌ Pending |
| sp_syscollector_start_collection_set | ❌ Pending |
| sp_syscollector_stop_collection_set | ❌ Pending |
| sp_syscollector_update_collection_item | ❌ Pending |
| sp_syscollector_update_collection_set | ❌ Pending |
| sp_syscollector_update_collector_type | ❌ Pending |
| sp_syspolicy_add_policy_category | ❌ Pending |
| sp_syspolicy_add_policy_category_subscription | ❌ Pending |
| sp_syspolicy_add_policy_xml | ❌ Pending |
| sp_syspolicy_configure | ❌ Pending |
| sp_syspolicy_delete_policy_category | ❌ Pending |
| sp_syspolicy_delete_policy_category_subscription | ❌ Pending |
| sp_syspolicy_delete_policy_execution_history | ❌ Pending |
| sp_syspolicy_delete_policy_xml | ❌ Pending |
| sp_syspolicy_execute_policy | ❌ Pending |
| sp_syspolicy_execute_policy_automated | ❌ Pending |
| sp_syspolicy_help_condition | ❌ Pending |
| sp_syspolicy_help_object_sets | ❌ Pending |
| sp_syspolicy_help_policy | ❌ Pending |
| sp_syspolicy_help_policy_category | ❌ Pending |
| sp_syspolicy_help_policy_category_subscription | ❌ Pending |
| sp_syspolicy_help_target_set_levels | ❌ Pending |
| sp_syspolicy_help_target_sets | ❌ Pending |
| sp_syspolicy_help_target_subsystems | ❌ Pending |
| sp_syspolicy_purge_health_state | ❌ Pending |
| sp_syspolicy_rename_condition | ❌ Pending |
| sp_syspolicy_rename_object_set | ❌ Pending |
| sp_syspolicy_rename_policy | ❌ Pending |
| sp_syspolicy_rename_target_set | ❌ Pending |
| sp_syspolicy_update_condition | ❌ Pending |
| sp_syspolicy_update_policy | ❌ Pending |
| sp_syspolicy_update_policy_category | ❌ Pending |
| sp_syspolicy_update_target_set | ❌ Pending |
| sp_table_privileges | ❌ Pending |
| sp_tablecollations_100 | ❌ Pending |
| sp_tablecollations_ex | ❌ Pending |
| sp_tableoption | ❌ Pending |
| sp_tableprivileges_ex | ❌ Pending |
| sp_tables | ✅ Implemented |
| sp_tables_ex | ❌ Pending |
| sp_trace_create | ❌ Pending |
| sp_trace_generateevent | ❌ Pending |
| sp_trace_getdata | ❌ Pending |
| sp_trace_geteventinfo | ❌ Pending |
| sp_trace_getfilterinfo | ❌ Pending |
| sp_trace_getinfo | ❌ Pending |
| sp_trace_getqueuedroppedevents | ❌ Pending |
| sp_trace_setevent | ❌ Pending |
| sp_trace_setfilter | ❌ Pending |
| sp_trace_setstatus | ❌ Pending |
| sp_unbindefault | ❌ Pending |
| sp_unbindrule | ❌ Pending |
| sp_unmap_login_from_cert | ❌ Pending |
| sp_update_alert | ❌ Pending |
| sp_update_category | ❌ Pending |
| sp_update_job | ❌ Pending |
| sp_update_jobschedule | ❌ Pending |
| sp_update_jobstep | ❌ Pending |
| sp_update_notification | ❌ Pending |
| sp_update_operator | ❌ Pending |
| sp_update_proxy | ❌ Pending |
| sp_update_schedule | ❌ Pending |
| sp_update_targetservergroup | ❌ Pending |
| sp_updateextendedproperty | ❌ Pending |
| sp_validatelogins | ❌ Pending |
| sp_validname | ❌ Pending |
| sp_who | ✅ Implemented |
| sp_xml_preparedocument | ❌ Pending |
| sp_xml_removedocument | ❌ Pending |
| sp_xp_cmdshell_proxy_account | ❌ Pending |
| sp_xtp_bind_db_resource_pool | ❌ Pending |
| sp_xtp_checkpoint_force_garbage_collection | ❌ Pending |
| sp_xtp_control_proc_exec_stats | ❌ Pending |
| sp_xtp_control_query_exec_stats | ❌ Pending |
| sp_xtp_unbind_db_resource_pool | ❌ Pending |

**Summary:** 8/320 (2.5%)

## System Catalog Views (`sys.*`)

| View | Status |
| :--- | :--- |
| sys.columns | ✅ Implemented |
| sys.index_columns | ✅ Implemented |
| sys.indexes | ✅ Implemented |
| sys.objects | ✅ Implemented |
| sys.stats | ✅ Implemented |

**Summary:** 5/5 (100.0%)
