# Everything search and Unicode emoji catalogue

Working material from Katran v3.8.0 (14e7373). This directory contains both
standalone Rust modules and exact host-dependent source snapshots. It adds no
Katran dependency to Avilon and requires no new workspace package.

## Ready modules

`everything_rank.rs` is std-only deterministic candidate ranking copied from
Katran with a neutral PathBuf record. Call `rank_folders(query, &mut candidates)`.
It promotes basename/exact/token matches, handles bounded near-name distance,
penalizes technical descendants and has deterministic ties. It ranks only the
candidates provided; it cannot prove disk-wide completeness. The original tests
are retained.

`emoji_catalogue.rs` is std-only and bundles fully-qualified Unicode16 entries,
groups, subgroups, substring search and scoped pagination. `page(query, group,
cursor, limit)` returns entries and divider positions. The UTF-8 sequence is the
neutral identity; Katran's virtual IDs and Event are absent. The cursor is opaque,
in-memory and bound to query/group/version; do not serialize its private fields
as a stable public wire format. Names are English Unicode names, not CLDR
multilingual or neural search. No font or renderer is required. `emoji_entries`
parses a pinned, trusted bundled file; it is not a general untrusted-file parser.

## Original Windows retrieval recipe

`reference/src/everything.rs` preserves the actual bounded multi-stream ES
retrieval, CSV parsing, deduplication, deadlines and incomplete flag. It is a
reference, not independently compiled here: replace the application-specific
helper resolver/cache directory and embedded helper with an explicit host-supplied
runner/path. Preserve query escaping, error versus empty-result semantics,
bounds and cancellation. Everything must be installed/running with IPC; no
binary, private index or user query/history is included. Approximate retrieval
includes bounded ASCII substitutions, not arbitrary full edit-distance recall.
`reference/src/folder_search_rank.rs` is the exact original ranker.

`reference/src/pod_metadata.rs` and `pods.rs` preserve the catalogue's production
integration and category-divider behavior. Their Service/Event/SQLite/copy paths
belong to the host; they are not dependencies of the ready module.

## Checks

From the Avilon root, compile/run each module with Rust1.95:

```powershell
rustc --edition=2024 --test materials/search-and-emoji/everything_rank.rs -o target/everything-rank-tests.exe
& ./target/everything-rank-tests.exe
rustc --edition=2024 --test materials/search-and-emoji/emoji_catalogue.rs -o target/emoji-catalogue-tests.exe
& ./target/emoji-catalogue-tests.exe
```

Tests use only invented paths and public Unicode data; no ES process, model,
Clipboard or UI is accessed. Standalone modules retain MIT attribution. Unicode
data is separately licensed in data/LICENSE.txt. Keep reference/AFIO-LICENSE and
reference/assets/everything notices when adapting the ES source. Source hashes
and original paths are recorded in provenance.json; adapted module hashes need
not equal their originals.
