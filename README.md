# Avilon

Independent Rust desktop SDK. Canonical local repository: C:\Avilon.

- crates/avilon-actions: shortcut policy and conflicts.
- crates/avilon-egui: semantic palette, responsive settings, action rows, keycaps and optional GPU optics.
- crates/avilon-windows: explicit Acrylic material requests and rollback.
- crates/avilon-media: bounded video backgrounds with optional egui adapter.

Each package has its own README and MIT license. Packages are internal and are not published to a registry.

Katran is a consumer. Its crates directory currently retains the tested dependency snapshot; it is not the canonical Avilon repository. Changes to this SDK must be deliberately integrated and verified in consumers.

No Katran storage, application code, artwork, fonts, videos or credentials are included.

Run the neutral example:

    cargo run --offline -p avilon-egui --example design

Origin: Katran local checkpoint f01c33b. Extraction: 2026-11-11. Existing component tests were run before separation; workspace metadata is checked independently here. Native appearance is not established by these checks.
