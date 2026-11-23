const {execFileSync} = require('node:child_process');
const {mkdirSync, rmSync, writeFileSync} = require('node:fs');
const path = require('node:path');
const WebSocket = require('ws');

const mode = process.argv[2];
const explained = mode === 'explained';
const sample = mode === 'sample';
const english = sample && process.argv[4] === 'en';
const explainedLines = require('./explained.json').lines;
const englishSample = 'First, we try the effect. Move the magnifying glass over the engraving and watch the lines bend. Then we see what works. We liked how the lens enlarges the lines, and how the little bulge drifts across the engraving on its own.';
const lines = sample ? [english ? englishSample : explainedLines.slice(0, 2).join(' ')] : explained ? explainedLines : [
  'Пробуем эффект.',
  'Смотрим, что сработало.',
  'Сохраняем в Авилон.',
  'И берём в новый проект.',
];
const model = sample ? process.argv[3] : 'gpt-realtime-2.1-mini';
const output = path.resolve(__dirname, '../../target/infographic',
  sample ? `voice-comparison/${english ? 'en/' : ''}${model}` : explained ? 'explained/voice' : 'voice');

async function main() {
  const key = (await Array.fromAsync(process.stdin)).map(String).join('').trim();
  if (!key) throw new Error('No API key received.');
  const socket = new WebSocket(`wss://api.openai.com/v1/realtime?model=${model}`, {
    headers: {Authorization: `Bearer ${key}`},
    handshakeTimeout: 15000,
  });
  const send = (event) => socket.send(JSON.stringify(event));
  mkdirSync(output, {recursive: true});
  const ready = path.join(output, 'ready');
  rmSync(ready, {force: true});
  let index = 0;
  let chunks = [];
  const transcripts = [];
  const usage = [];

  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('OpenAI audio request timed out.')), 120000);
    const finish = () => { clearTimeout(timer); resolve(); };
    const fail = (error) => { clearTimeout(timer); reject(error); };
    function next() {
      send({
        type: 'response.create',
        response: {
          conversation: 'none',
          input: [],
          instructions: english
            ? 'Read the following passage verbatim in natural conversational English, in one continuous take. Explain it to a friend. Link ideas smoothly, articulate clearly, and keep the delivery relaxed. Start directly with the passage.\n\n' + lines[index]
            : sample
            ? 'Произнеси следующий текст дословно одним связным дублем на естественном русском. Представь, что объясняешь знакомому, как устроен эффект. Держи разговорный ритм и связывай фразы по смыслу. Без вступления.\n\n' + lines[index]
            : 'Прочитай ровно текст в кавычках, естественным спокойным голосом диктора, без вступления и дополнительных слов. Произноси «Авило́н». Текст: «' + lines[index] + '»',
        },
      });
    }
    socket.on('open', () => send({
      type: 'session.update',
      session: {
        type: 'realtime',
        output_modalities: ['audio'],
        audio: {output: {voice: 'marin', format: {type: 'audio/pcm', rate: 24000}}},
      },
    }));
    socket.on('error', fail);
    socket.on('close', () => fail(new Error('OpenAI closed the connection.')));
    socket.on('message', (payload) => {
      try {
        const event = JSON.parse(payload.toString());
        if (event.type === 'session.updated') next();
        if (event.type === 'response.output_audio.delta') chunks.push(Buffer.from(event.delta, 'base64'));
        if (event.type === 'error') {
          throw new Error(`${event.error.code ?? 'OpenAI'}: ${event.error.message}`);
        }
        if (event.type !== 'response.done') return;
        if (event.response.status !== 'completed' || chunks.length === 0) {
          throw new Error(`No audio for line ${index + 1}: ${event.response.status_details?.error?.message ?? event.response.status}`);
        }
        transcripts.push(event.response.output.flatMap((item) => item.content ?? [])
          .map((part) => part.transcript ?? '').join(' ').trim());
        usage.push(event.response.usage ?? null);
        const wave = path.join(output, `line-${index + 1}.wav`);
        execFileSync('ffmpeg', [
          '-y', '-v', 'error', '-f', 's16le', '-ar', '24000', '-ac', '1', '-i', 'pipe:0',
          '-af', 'silenceremove=start_periods=1:start_duration=0.03:start_threshold=-47dB',
          wave,
        ], {input: Buffer.concat(chunks), windowsHide: true});
        chunks = [];
        index += 1;
        console.log(`Recorded ${index} / ${lines.length}`);
        if (index === lines.length) {
          writeFileSync(path.join(output, 'transcripts.json'), JSON.stringify(transcripts, null, 2));
          writeFileSync(path.join(output, 'usage.json'), JSON.stringify({model, responses: usage}, null, 2));
          writeFileSync(ready, model + '\n');
          finish();
        } else {
          next();
        }
      } catch (error) {
        fail(error);
      }
    });
  }).finally(() => socket.close());
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
