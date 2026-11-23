const {execFileSync} = require('node:child_process');
const {mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} = require('node:fs');
const {tmpdir} = require('node:os');
const path = require('node:path');
const {bundle} = require('@remotion/bundler');
const {renderMedia, selectComposition} = require('@remotion/renderer');

const cuts = {
  short: [['MiraCycle', '', 'mira-cycle']],
  explained: [['MiraCycleExplained', 'explained', 'mira-cycle', 'mira-cycle-explained']],
  studies: [
    ['MiraWeight', 'studies', 'weight'],
    ['MiraRelay', 'studies', 'relay'],
    ['MiraListToMap', 'studies', 'list-to-map'],
    ['MiraNewFocus', 'studies', 'new-focus'],
    ['MiraSync', 'studies', 'sync'],
  ],
};

async function main() {
  const root = path.resolve(__dirname, '../..');
  const mode = process.argv[2] ?? 'short';
  if (!Object.hasOwn(cuts, mode)) throw new Error(`Unknown render mode: ${mode}`);
  const serveUrl = await bundle({entryPoint: path.join(__dirname, 'scene.jsx')});
  const options = {serveUrl, browserExecutable: process.env.MIRA_RENDER_BROWSER || undefined};
  for (const [id, directory, filename, asset = filename] of cuts[mode]) {
    await renderCut(root, options, id, directory, filename, asset);
  }
}

async function renderCut(root, options, id, directory, filename, asset) {
  const output = path.join(root, 'target', 'infographic', directory);
  mkdirSync(output, {recursive: true});
  const composition = await selectComposition({...options, id});
  const video = path.join(output, `${filename}.mp4`);
  await renderMedia({...options, composition, codec: 'h264', muted: true, outputLocation: video, concurrency: 4});

  const temporary = mkdtempSync(path.join(tmpdir(), 'mira-cycle-'));
  try {
    execFileSync('ffmpeg', [
      '-v', 'error', '-i', video, '-vf', 'scale=768:-2', '-q:v', '4',
      path.join(temporary, '%06d.jpg'),
    ], {stdio: 'inherit', windowsHide: true});
    const frames = readdirSync(temporary).sort().map((name) => readFileSync(path.join(temporary, name)));
    if (frames.length !== composition.durationInFrames) {
      throw new Error(`Expected ${composition.durationInFrames} frames, got ${frames.length}`);
    }
    // Little-endian frame rate, count, and byte offsets into the JPEG payload.
    const index = Buffer.alloc(8 + (frames.length + 1) * 4);
    index.writeUInt32LE(composition.fps, 0);
    index.writeUInt32LE(frames.length, 4);
    let offset = 0;
    frames.forEach((frame, number) => {
      index.writeUInt32LE(offset, 8 + number * 4);
      offset += frame.length;
    });
    index.writeUInt32LE(offset, 8 + frames.length * 4);
    writeFileSync(path.join(root, 'assets', 'atlas', `${asset}.frames`), Buffer.concat([index, ...frames]));
    console.log(video);
  } finally {
    rmSync(temporary, {recursive: true});
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
