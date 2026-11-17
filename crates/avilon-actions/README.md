# avilon-actions

Pure Rust shortcut normalization and assignment policy, extracted from Katran.
Hosts supply action definitions, reserved-chord rules, override storage and all
input, UI and dispatch behavior. The crate does not read or write preferences.

An absent override uses the definition's default; `None` explicitly unbinds it.
Catalogues must have unique, nonempty action IDs and canonical defaults (or an
empty default). `validate_catalogue` checks these constraints without rejecting
context-dependent default overlaps. `plan_set` and `validate_overrides` validate
automatically; call it explicitly before using the infallible `binding` lookup.
`validate_overrides` normalizes stored values and checks their action IDs and
rebindability, but deliberately does not reject pre-existing conflicts or
reserved keys. `plan_set` returns a new map without changing its input. It checks
reset defaults too; replacing conflicts removes only overlapping alternatives
and cannot displace a protected action. Conflict order follows the catalogue.
`/` separates alternatives, not sequential key combinations.

See `examples/shortcuts.rs` for a neutral host example. The normalized key names
are policy values; platform-specific shortcut registration and availability are
host responsibilities. This internal, unpublished 0.1 crate uses Rust 2024.

Shortcut policy adapted from Katran; see the accompanying MIT license.

## Palette and visible-list navigation

`palette::Palette` filters a host-supplied catalogue by multiple case-insensitive
words, resets selection on query changes, clamps to the visible list and wraps
navigation. The host draws rows, executes actions and owns persistence.
`navigation::ItemCursor<T>` tracks a stable host ID, enters the first or last
visible item, and clears a cursor absent from the new list.

```rust
use avilon_actions::palette::{Direction, Entry, Palette};
let entries = [Entry { id: "open", label: "Open document", group: "File",
    search_terms: "read", hidden_when_empty: false }];
let mut palette = Palette::default();
palette.set_query("file read");
let visible = palette.visible(&entries);
palette.step(visible.len(), Direction::Next);
assert_eq!(palette.selected_id(&entries, &visible), Some("open"));
```

The two modules use only the Rust standard library. Adapted from Katran
`14e737348af0f6a188d2eb89f8403e06ec3e1c87`; their source contains synthetic unit
tests. The host remains responsible for keyboard bindings and accessibility.
