# Windows and local AI reference shelf

This is a collection of **host-dependent reference implementations**, captured
from Katran on 2026-11-18. It is not a standalone SDK or Cargo package, and adding
it to Avilon does not reconnect or change Katran. Source files retain their
original module names and exact bytes so another project can study or adapt a
working solution without losing its tests and failure semantics.

`provenance.json` records each original path, byte count and SHA-256. Source
revision `14e737348af0f6a188d2eb89f8403e06ec3e1c87` was supplied by the integration
lead; this import did not run Git. The material's identity is its recorded file
hashes. Katran application data, logs, databases, runtime environments, models,
fonts, artwork and videos are not included.

## Contents and reuse boundaries

The `reference/` tree preserves the original `src/`, `scripts/` and `tests/`
layout. Rust tests are examples to adapt with the implementation; they do not
compile as an independent test suite here.

| Solution | Implementation and fixtures | Host-dependent boundary |
| --- | --- | --- |
| Physical shortcut recording and Windows registration | `src/bin/shortcuts_ui.rs`, `src/bin/tab_hotkey_ui.rs`; shared `src/overlay.rs`, `src/overlay_windows.rs`; `tests/global_hotkey_keyboard.rs`, `tests/tab_global_hotkeys_api.rs`, `tests/tab_hotkey_windows.rs` | Recorder presentation uses egui and Katran action/settings APIs. Replace labels, persistence, action IDs and `ShelfTab`; registration success is separate from saving a binding. Windows reservations can reject it. |
| Previous-window paste intent and file publication proof | `src/foreground_windows.rs`, `src/overlay.rs`, `src/overlay_windows.rs`, `src/overlay_service.rs`, `src/copy_out.rs`; supporting `src/text_clipboard.rs`, `src/table_clipboard.rs`; `tests/overlay.rs`, `tests/overlay_api.rs`, `tests/paste_intent_api.rs`; `scripts/paste-receiver.html` | The pure visit/lease rules can be adapted independently. Win32 effects, copy-job ownership, revision-bound selection, exports and retention need explicit host adapters. `copy_out.rs` is not a generic file-transfer library. |
| Visible optional-component lifecycle | `src/ai_components.rs`, `src/ai_component_bundle.rs`, `src/folder_cpu_runtime.rs`, `src/folder_gpu_runtime.rs`, `src/emoji_ai/runtime.rs`; `scripts/ai-components/`, setup/manifest scripts; `tests/ai_components_api.rs` | Component IDs, request/receipt storage, bundled resources, model manifests, terminal launch and application wording are Katran-specific. Status does not start installation and file presence is not verification. |
| Resident semantic worker and persistent derived cache | `src/folder_semantic.rs`, `scripts/folder-semantic/worker.py`; `tests/folder_semantic_api.rs`, `tests/folder_semantic_cache_test.py` | `Backend`/`Source`, framed worker lifecycle and cache identity are reusable patterns. Everything, favorites/history, cancellation type, E5 descriptors, dimensions, Python/ORT/CUDA resolution and profile paths need a host. |

Paths in this table are relative to `reference/`. The full shared overlay files
are stored once, rather than duplicated between hotkey and paste examples.

## Preserved contracts

### Shortcuts

Physical modifier press/release tracking supplements each key event's modifier
snapshot; focus loss clears held state. Logical and physical recording are
different choices. The main key is not a modifier. Canonical punctuation names
such as `Slash` must not be confused with slash-separated shortcut alternatives
in action policy. Global registration belongs to an explicit owner with Drop
cleanup; library loading must not start a listener. A tab/global binding being
persisted does not prove that Windows registered it. Release old registrations
before swapped assignments, and suspend registration while recording.

### Paste and Clipboard

Start native observation explicitly in a production host before it activates its
window. Ordinary constructors/tests must not install hooks. Keep current identity,
the last external candidate and visit epoch in RAM, without titles, content or
history. Own Settings transitions preserve the target; unavailable or ambiguous
observations must not restore an older destination.

Begin freezes owner, exact target and visit in a lease. A changed visit while copy
is pending cancels delivery and cannot retarget it. A successful or partial input
attempt must not be silently retried. An observation-cleanup failure after a
successful send remains `sent` with a warning. A new explicit summon from the
same still-foreground destination is a fresh intent, not reuse of the old lease.

File publication owns a real message-only HWND through CloseClipboard. The
receipt is settled **after** that guard closes and its owner is destroyed. The
settlement compares the exact fresh marker, HDROP, actually published DIBV5 and
copy effect under a read lock, with bounded sizes, a format whitelist and a stable
sequence before/after reading and after closing. Failed proof is successful copy
with `sequence=0`: preserve exports and withhold automatic input. Returning an
ordinary copy error at this stage would incorrectly delete paths already handed
to Clipboard. Do not add a fixed sequence delta, delay or general late file
fallback. Export lifetime remains a host responsibility.

The included browser receiver is an **owner-operated fixture**, not an automatic
test. It has no network access and reports paste events and format/file metadata,
not file names or bytes. Opening it does not prove any editor accepted a file;
only a deliberate paste event supplies Clipboard data.

### Optional installation and semantic cache

The terminal flow separates launch acknowledgement, installed files and successful
functional verification. An explicit Install/Verify action may open a visible
terminal, create profile-local request/log/receipt files, install Python packages
and download pinned resources. Closing early and unfinished operations after
restart have explicit failure/unknown states. Existing runtimes are not silently
overwritten. These scripts retain Katran component names and paths as a reference;
adapt them before choosing a new application's installation destination.

`ai_component_bundle.rs` and the emoji setup scripts reference Unicode/CLDR data
that is **not bundled in this section**. `provenance.json` lists the omitted
public resources and their hashes. Supply a separately licensed, version-bound
data package when adapting the full emoji installer. Do not treat these Rust
snapshots as a complete compilable bundle. Model weights and generated emoji
indexes are also absent.

The Folder worker only imports NumPy/ONNX Runtime/tokenizers inside its inference
entry point. Its descriptor and cache helpers can therefore be exercised with
stdlib fakes. The cache binds model/tokenizer/sidecar hashes, preprocessing,
pooling/prefixes, dimensions, ORT and CPU/GPU identity. It stores hashed descriptor
keys and validated vectors, not plain paths or query history. It has bounded
rows/frames and transaction-based writes; optional cache failure preserves RAM
search. Hashed descriptors are not an anonymization guarantee.

## What can run independently

The only automated checks recommended directly from this material are the cheap,
offline checks below. They use Python's standard library and temporary synthetic
files. No model, package installation, live index, native hook, window or personal
Clipboard is involved. No bytecode cache is written.

```powershell
python -B materials/windows-and-ai/checks/check_material.py
python -B materials/windows-and-ai/checks/check_material.py --synthetic
```

Run these from the Avilon root. The first verifies snapshot hashes and parses
Python sources without importing them. The second additionally runs:

- `reference/tests/folder_semantic_cache_test.py`: cold/reordered/changed
  descriptors, process restart, prune, identity changes, corruption, interrupted
  writes, locks and unavailable cache paths; every path/database/model byte is
  synthetic and temporary.
- `reference/scripts/ai-components/test_verify.py`: missing model, invalid
  existing runtime and digest mismatch, without calling inference/installers.

All other scripts are references requiring deliberate selection and a host.
Do not include installer entry points, model workers, HTML browser interaction
or `tab_hotkey_windows.rs` in a generic automatic shelf check. The Windows test
registers a real chord when enabled. `test-contract.ps1` is preserved for its
synthetic terminal cases but launches PowerShell and writes under its local
reference tree; it is intentionally outside the default checks.

## Evidence limits and adoption

The source contains synthetic Rust tests for modifier state, registration policy,
visit/lease cancellation, partial input, publication proof, failed-proof retention,
terminal lifecycle and semantic snapshot validation. They depend on the original
host crate and are not recompiled by this material. The copied Python cache and
diagnostic fixtures are independently runnable; local results are recorded in
`VERIFICATION.json`.

Katran's prior owner confirmation covered the reported same-file Notion browser
paste after receipt settlement. It does not establish every file type, editor or
new host's native behavior. WinEvent observation and input remain non-atomic;
the bounded barrier can fail closed but does not prove every OS event was seen.
Clean-machine Python/CPU/CUDA installation and native appearance remain separate
acceptance tasks. A synthetic cache test does not execute the model or prove CUDA.

For adoption, begin with a neutral Host/Launcher/Backend and synthetic fixtures,
then wire the consumer's command/storage API. Enable native effects or visible
installation only at the consumer's explicit action boundary. Neither storing
this material nor updating Avilon's existing crates requires changing Katran.

See `LICENSE`, `notices/AFIO-LICENSE`, `notices/UNICODE-LICENSE.txt`,
`notices/CLDR-LICENSE` and `notices/THIRD-PARTY.md` before adapting distribution.
