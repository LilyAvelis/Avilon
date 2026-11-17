# Scoped state, ordered pages and resident images

A shelf of working Katran 3.8 source references plus a small independent behavioral fixture. This is **not** a standalone Rust crate or a drop-in SDK. The original implementations were kept as source snapshots to adapt inside a host with its own database, dispatcher and UI. `fixture/` is a separate neutral illustration; it is not production Katran code and passing it does not establish that the reference Rust compiles in another project.

## Contents and provenance

- `reference/src/tag_slots.rs`, `tag_service.rs`: scoped definitions/assignments, checked saves and atomic reorder; `desktop_pins.rs` shows a different identity domain.
- `reference/src/sorting.rs`, `catalogue_query.rs`, `catalogue_snapshot.rs`, `desktop_projection.rs`: sort keys, grouped SQL, keyset cursor and host read snapshot.
- `reference/src/bin/images.rs`, `api_client.rs`, `tag_material_ui.rs`: image request epochs, resident textures, validated metadata-only transitions and stale cursor handling.
- `reference/tests/` and `reference/docs/`: original synthetic project tests and contract notes. They depend on the Katran host and are **not** independently runnable here.
- `fixture/state_contract.py`, `image_contract.py`, `test_contracts.py`: independent Python standard-library examples and synthetic behavior tests. They are intentionally smaller than the originals and have their own fixture schema, no real images and no Katran profile.
- `SOURCE-INVENTORY.json` lists each copied file's original relative path, byte count, SHA-256 and snapshot destination. The audit named Katran checkpoint `14e737348af0f6a188d2eb89f8403e06ec3e1c87`; that historical audit identifier does **not** assert that these later-read file bytes match that commit. Verify the recorded hashes for the actual snapshots.
- `LICENSE` is the source MIT license, copyright © 2026 LilyAvelis. No runtime, model, actual database, private paths or media are included.

## Tag reorder and checked writes

Katran assigns nine positional slots to each tab/scope. `order[new_position - 1]` contains the old slot number: `[2,3,1,4,...]` moves the former first tag and its assignments to position three. In one SQLite IMMEDIATE transaction it validates a complete permutation, compares a scoped revision, rewrites **all** definition fields at target positions, remaps assignments with a validated `CASE`, bumps the scoped persisted counter and commits. Any SQL or validation error rolls back the whole operation. The separately keyed Desktop assignments require their own remap. A checked editor save uses the same revision/counter boundary so an editor holding an obsolete position cannot overwrite a different tag.

The revision combines the scope, complete ordered definitions and a persistent counter. The counter is bumped even for an identical-looking reorder: otherwise identical tags with different assignments permit an ABA stale-save error. Counter corruption and overflow are explicit failures. Assignment changes themselves do not bump this definition revision; grouping cursors additionally bind the assignment map. Names, colours, materials and positions are examples of one tag's definition; do not copy Katran's pin-only assignment policy or nine-slot limit as universal rules. Consumers can instead use stable tag IDs separate from display position if their data model permits it. Preserve scope isolation, atomicity, lossless assignment remapping and the application-specific rules deliberately chosen for the destination.

## Grouping, keysets and snapshot boundaries

The `Tags` sort joins scoped assignment before sorting and before the final `LIMIT`: tagged buckets precede untagged, with stable ID tie-breaks. The keyset predicate uses the same order as the `ORDER BY`; sorting only the already limited page is incorrect. Each cursor is bound to tab, query, sort, filters, generation and a tag-generation fingerprint. That fingerprint includes ordered assignments plus the definition revision, detecting relevant edits from another Service before a later page. Folder/virtual catalogue/desktop cases have their own domain rules; see the original docs/tests instead of assuming they share capture-event IDs.

`query_sorted_slice` opens a read transaction when called on an autocommit connection; otherwise it uses the caller's existing transaction. Consequently the generation fingerprint and page query share that transaction. `catalogue_snapshot.rs` supplies its own encompassing read transaction for the page and associated metadata. Preserve this wrapper when adapting the SQL: do not call the inner query outside a transaction. The copied second-writer tests exercise stale-cursor rejection **between** page requests; the neutral fixture illustrates that behavior but does not independently reproduce concurrent in-call scheduling. Revalidate asynchronous snapshots before reuse, using the same connection's stamp as shown in the host reference.

## Images and metadata changes

An image key combines stable event ID, requested edge, content revision and an epoch for the browse context. Namespace includes tab, folder and catalogue. Generation changes always advance the epoch and cancel/ignore queued, pending and late work; geometry is cleared. Across a generation change in the **same** namespace, the implementation may rekey only resident successful decoded `image` textures with the same ID/edge/revision. A changed content revision issues a new request; a different namespace drops resident entries. Shell thumbnails/icons and fallback/unavailable results are not generally portable across arbitrary generations: Windows provider, original path and event name can affect those visuals.

A checked metadata-only ACK can bridge context generation and retain in-flight work only when the old and new interaction snapshots prove the exact permitted transition. For item edits that may affect membership/order, the UI still requeries metadata after the bridge. For tag-style-only edits the existing page may remain, but an old generation-bound page cursor must be discarded and rebuilt before loading more. `images.rs` also demonstrates bounded queues, worker count, result handling and resident texture limits; adapt those limits, decoder validation and texture lifetime to the host. `fixture/image_contract.py` is a small behavioral model, not an egui cache implementation.

## Quick standalone check

From this directory, with Python 3 and its standard library:

```powershell
python -B -X utf8 -m unittest discover -s fixture -p test_contracts.py -v
```

The four tests create only a temporary SQLite file and synthetic data. They check scoped remapping, identical-label ABA protection, transactional rollback, grouping before page limit, cursor rejection after an external write, decoded-only resident carry, late work, changed revisions and namespace changes. The reference Rust tests require the original project's dependencies and API wiring; no Cargo configuration or full product build is included here.

For source integrity, hash each path in `SOURCE-INVENTORY.json` under `reference/` with SHA-256 and compare its recorded length/hash. This checks copied files only; it does not establish the source commit, independent compilation, native image decoding, real concurrency timing or production UI acceptance.
