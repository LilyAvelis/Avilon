# Avilon

A warehouse of reusable desktop solutions from real projects. Canonical local
repository: C:\Avilon. Materials can be ready Rust components, working reference
implementations, or recipes with synthetic fixtures. A source project does not
have to adopt Avilon as a live dependency to contribute a proven solution.

## Ready Rust components

- [avilon-actions](crates/avilon-actions/README.md): shortcut policy/conflicts,
  multiword action palettes and stable-ID list navigation.
- [avilon-egui](crates/avilon-egui/README.md): semantic palettes with accent tint,
  responsive settings, action rows/keycaps, glass tabs/materials and GPU optics.
- [avilon-windows](crates/avilon-windows/README.md): explicit Acrylic/corner
  requests and rollback.
- [avilon-media](crates/avilon-media/README.md): bounded video backgrounds, latest
  frame delivery and cleanup of an assigned decoder process tree.

Packages are internal MIT components (`publish = false`), not registry releases.
Use path/Git dependencies when useful, or adapt a checked source snapshot.
Host-owned configuration, assets, windows and persistence remain explicit.

## GUI

[Темы МИРЫ](gui/mira/README.md) — десять нативных шаблонов, атлас запуска,
пресеты egui и две базовые компоновки для рабочих интерфейсов. [Раздел GUI](gui/README.md)
связывает темы с компонентами и визуальными эффектами Avilon.

## Material shelves

| Shelf | Ready to reuse | Reference implementations / recipes |
| --- | --- | --- |
| [Search and emoji](materials/search-and-emoji/README.md) | std-only Everything candidate ranker and Unicode16 emoji catalogue with groups/search/pages | Bounded ES retrieval, production catalogue integration and original tests |
| [State and view](materials/state-and-view/README.md) | Independent SQLite/image policy fixtures | Scoped tags, CAS/reorder, grouped/keyset sorting, resident image cache and production tests |
| [Windows and local AI](materials/windows-and-ai/README.md) | Offline cache/diagnostic checks | Physical hotkeys, Windows registration, paste leases/Clipboard proof, visible component setup and persistent semantic worker |

A reference snapshot is intentionally host-dependent, not a standalone Cargo
package. Each shelf records provenance, licenses, dependencies, adaptation points
and test scope. Public Unicode data has its own notice. No personal profile,
database, exports, trained weights, runtime environment, artwork, fonts or videos
are included. Host application source is a reference rather than a new SDK API.

## Checks and provenance

```powershell
cargo test --offline --locked --workspace --all-features --target-dir target -j2
python -B scripts/verify-materials.py
```

See each shelf for standalone checks that do not require Katran. Preserve the
source revision and file hashes when copying a material to another project.
[Design SDK guide](docs/design-sdk.md) describes the ready UI subset.
[Katran3.8 audit](docs/katran-3.8-reuse-audit-2026-11-18.md) is the historical
selection record; [import record](docs/katran-3.8-import-2026-11-18.md) records the
implemented batch and exact verification scope.

Katran remains unchanged and keeps its own verified crate snapshot. No registry
publication or automatic consumer migration is part of this import. Build/test
outputs belong in this repository's existing target cache; do not add a cache
for every material or archive binaries with source recipes.
