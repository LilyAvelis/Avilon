# Avilon

The ready Rust desktop component subset of the Avilon solution warehouse.
Originally extracted from Katran; consumers adopt it through their own adapters. Internal 0.1 packages, MIT, `publish = false`; no registry release or
machine-wide installation. Pin a Git revision or use workspace/path dependencies.

## Design library

| Package | Reusable implementation | Host responsibility |
| --- | --- | --- |
| [avilon-egui](../crates/avilon-egui/README.md) | Semantic Standard/Blue palette, independent selection tint, shared spacing, responsive settings layout, ActionRow/KeyCaps, CPU reference lens | Theme preferences, persistence, commands, fonts and assets |
| `avilon-egui` with `glow` | Live GPU refraction, blur, highlights, chromatic adjustment and selection tint over an application-owned background texture | Texture lifetime, GL context and shutdown cleanup |
| [avilon-windows](../crates/avilon-windows/README.md) | Explicit Desktop Acrylic and corner request, typed failures and partial-application rollback | Existing owned HWND, transparent viewport/clear/panels and opaque fallback |
| [avilon-media](../crates/avilon-media/README.md) | Bounded looping playback policy; with `egui`, decoder worker, texture reuse, seam blend and pause/cleanup | Validated media path/metadata, FFmpeg command factory, asset ownership and configuration |
| [avilon-actions](../crates/avilon-actions/README.md) | Shortcut normalization and conflict assignment policy | Action catalogue, persistence and execution |

```toml
[dependencies]
avilon-egui = { path = "../Avilon/crates/avilon-egui", features = ["glow"] }
avilon-media = { path = "../Avilon/crates/avilon-media", features = ["egui"] }
avilon-windows = { path = "../Avilon/crates/avilon-windows" }
```

The example paths assume a sibling Avilon checkout; adapt them to your layout.
Katran is not automatically reconnected to the canonical warehouse.

Use `theme::PaletteOptions` to resolve semantic colors, `settings_layout` for
responsive rows, and `optical_gpu::shape` below foreground text. Retain the
background `TextureHandle`; cards and menus can sample that same current video
texture. Call `optical_gpu::destroy` on the matching render thread/context before
shutdown. Acrylic and in-app optics are independent layers and can be combined.

The video host supplies `FnOnce() -> Result<std::process::Command, String>` when
ticking its player. No download, media discovery or user profile is hidden inside
the SDK. The current decoder is Windows D3D11VA, bounded to 24fps/1280×720; frames
pass through CPU memory before upload. It is not zero-copy. A hidden window should
tick inactive, which stops the decoder and frees its loop cache while retaining
the last texture. See the media README for limits and error handling.

No lily artwork, Apple font, personal video or Katran database is bundled in
Avilon. No desktop capture or refraction of arbitrary external windows is
implemented. The reusable optical effect samples application-owned content.

Run a neutral host without a native window:

```powershell
cargo run --offline -p avilon-egui --example design
```

This is a shared source SDK for applications that adopt it, not a service that
restyles other Windows programs. Integration recipes and input/layout rules are
in the design-pattern documentation of the consumer application.

