const {execFileSync} = require('node:child_process');
const path = require('node:path');
const defaultOutput = path.resolve(__dirname, '../../target/infographic');
const explained = process.argv[2] === 'explained';
const output = explained ? path.join(defaultOutput, 'explained') : process.argv[2] ?? defaultOutput;
const revealTimes = explained ? require('./explained.json').starts : require('./timing.json');
const voice = path.join(output, 'voice');
const silentVideo = path.join(output, 'mira-cycle.mp4');
const soundtrack = path.join(output, 'mira-cycle.wav');
const narratedVideo = path.join(output, 'mira-cycle-voiced.mp4');

function run(program, args) {
  return execFileSync(program, args, {encoding: 'utf8', windowsHide: true});
}

function duration(file) {
  const seconds = Number(run('ffprobe', [
    '-v', 'error', '-show_entries', 'format=duration',
    '-of', 'default=noprint_wrappers=1:nokey=1', file,
  ]).trim());
  if (!Number.isFinite(seconds) || seconds <= 0) throw new Error(`Invalid duration for ${file}`);
  return seconds;
}

const runtime = duration(silentVideo);
const lines = revealTimes.map((_, i) => path.join(voice, `line-${i + 1}.wav`));
const filters = lines.map((file, i) => {
  const start = revealTimes[i];
  const end = revealTimes[i + 1] ?? runtime;
  const speed = Math.max(1, duration(file) / (end - start - 0.15));
  const delay = Math.round(start * 1000);
  return `[${i}:a]atempo=${speed},adelay=${delay}[line${i}]`;
});
const inputs = lines.map((_, i) => `[line${i}]`).join('');
filters.push(`${inputs}amix=inputs=${lines.length}:normalize=0,apad=whole_dur=${runtime}[voice]`);
run('ffmpeg', [
  '-v', 'error', '-y', ...lines.flatMap((file) => ['-i', file]),
  '-filter_complex', filters.join(';'), '-map', '[voice]', '-t', String(runtime),
  '-c:a', 'pcm_s16le', '-ar', '24000', '-ac', '1', soundtrack,
]);
run('ffmpeg', [
  '-v', 'error', '-y', '-i', silentVideo, '-i', soundtrack,
  '-map', '0:v:0', '-map', '1:a:0', '-c:v', 'copy', '-c:a', 'aac', '-b:a', '96k',
  '-metadata:s:a:0', 'title=AI-generated narration (OpenAI)', '-movflags', '+faststart', narratedVideo,
]);
console.log(narratedVideo);
