# Katran3.8 warehouse import

18 November2026. Source Katran v3.8.0, revision
14e737348af0f6a188d2eb89f8403e06ec3e1c87. Canonical destination C:\Avilon.
Katran remains an unchanged donor with its own verified dependency snapshot.
No consumer migration, registry release, GitHub publication or model installation.

## Delivered

Existing actions package gains action palette and stable-ID list navigation;
egui gains accent tint; media gains assigned-process-tree cleanup and latest
frame delivery. Existing windows package is unchanged. Existing SDK documentation
now references the independent warehouse checkout instead of the Katran copy.

Three shelves add **93 material files / 2,110,927 bytes**, excluding the inventory
itself, and **74 exact source snapshots**. Original snapshot hashes are retained;
all74 also match their Git source at the recorded revision after normalizing
checkout LF/CRLF. `materials/INVENTORY.json` binds the actual copied bytes and
records Git blob hashes separately. Local workstation paths were removed from
structured provenance; relative project paths and the source revision suffice.

- Search/emoji: standalone std-only ranker and Unicode16 read-only catalogue,
  category/search/page API, data notices and original retrieval/integration code.
- State/view: original scoped tags/reorder/CAS, sorted query snapshots and image
  cache code, project tests, independent synthetic SQLite/cache models.
- Windows/AI: original physical shortcut, paste lease/Clipboard proof, visible
  installer and semantic worker/cache source; explicit dependencies and offline
  checks. Missing public datasets are declared, not silently assumed bundled.

Reference code intentionally keeps its original host dependencies. It is not
advertised as a self-contained Rust library. Ready modules and reference recipes
are labelled separately, with license notices and adaptation boundaries.

## Verification

- Existing four-component Rust workspace, allfeatures: **41 tests passed**,
  including synthetic assigned-process cleanup and latest-frame behavior.
- Standalone ranker/catalogue: **4 tests passed**, no dependencies/downloads.
- State/view synthetic fixtures: **4 tests passed** (Nemi's unchanged fixture run).
- Windows/AI offline fixtures: **8 tests passed**, hashes41/41 and Python AST7/7
  (Azuros's unchanged fixture run; see its VERIFICATION.json).
- Aggregate inventory verified **93/93 files**, **74/74 originals** and source
  revision normalized-byte agreement.
- Workspace formatter and alltargets/allfeatures Clippy passed.

The original Avilon lockfile contained unused dependencies from its extraction.
Locked preflight rejected it before compilation. Offline resolution pruned the
lockfile to this workspace, then the locked gate passed. Clippy exposed an old
item-after-tests declaration in transition.rs; only declaration order was moved
within Avilon, with unchanged logic. Final Clippy passed. The successful Rust
behavior gate was retained rather than rebuilding Katran or repeating all tests.

No personal Clipboard, foreground windows, browser UI, install scripts, GPU/model
or Katran release CI was exercised. A synthetic child process test establishes
cleanup after Job assignment; spawn-to-assignment remains a documented limit.
Reference fixtures establish the illustrated contracts, not generic compatibility
with arbitrary hosts or third-party applications.

## Preservation and use

Canonical files are checked against the pre-import baseline before publishing;
no unrelated source is overwritten. The source-only import receipt is a local
Git checkpoint. Build outputs remain in the existing C:\Avilon\target cache.
Run `python -B scripts/verify-materials.py` after copying the warehouse, then the
specific standalone checks in a shelf's README. Do not create or archive a full
Cargo target tree for every recipe.
