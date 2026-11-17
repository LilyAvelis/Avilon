# avilon-media

A reusable, bounded FFmpeg video-background player for egui hosts. The host supplies an FFmpeg command factory and owns discovery, installation, and configuration of the executable. Enable the `egui` feature for the player and texture adapter.

```rust,no_run
use avilon_media::{State, VideoSource};
use std::{path::PathBuf, process::Command};

let mut player = State::default();
let source = VideoSource {
    path: PathBuf::from("background.mp4"),
    width: 1280,
    height: 720,
    duration_ms: 10_000,
    smooth_loop: true,
};
let ctx = egui::Context::default();
let texture = player.tick(&ctx, Some(source), true, || Ok(Command::new("ffmpeg")));
// Call tick on subsequent frames; display the returned texture in your UI.
```

The player uses FFmpeg's Windows D3D11VA decoder and downloads NV12 frames for processing. Output is RGBA in CPU memory and uploaded to egui as a texture; this is not zero-copy. Video is scaled to fit 1280×720 and paced at 24 fps. A bounded 512 MiB estimated NV12 loop cache is used when possible; longer videos use streaming. Optional four-frame seam smoothing applies only to cached loops. Pausing stops the decoder and retains the last texture. Decoder errors are exposed through `State::error`; no fallback decoder is selected automatically. Source metadata must be supplied by the host (dimensions up to 3840×2160 and duration up to 120 seconds). The host must provide a compatible FFmpeg build and handle file permissions and user consent.

## Resume after hiding

The player retains the last texture and the next delivered frame position while inactive. On show it restarts decoding and trims output after the full source loop, preserving the complete cycle instead of looping only a sought tail. The frame offset is bounded by the duration-derived cycle estimate; inaccurate duration metadata can affect resume precision. Decoder startup latency remains, and a full application restart does not retain this in-memory state. No playback runs while hidden.

## Bounded frames and Windows shutdown

The decoder queue retains the first pending frame plus a bounded latest-frame
slot, allowing the presenter to catch up without accumulating a backlog. On
Windows the decoder is assigned to a Job Object configured to terminate assigned
processes when the job closes; shutdown closes the job before waiting on the
reader. A synthetic Windows test exercises stopping an assigned child, and a
headless presenter test checks the latest frame. These checks do not establish
that descendants cannot escape during the window between process spawn and Job
assignment. Assignment failure stops the direct child and reports an error;
stronger spawn-time containment remains future work.

Adapted from Katran `14e737348af0f6a188d2eb89f8403e06ec3e1c87`; the host still
owns FFmpeg discovery, permissions and renderer lifecycle. No additional package
dependency is needed for the Windows Job Object bindings.
