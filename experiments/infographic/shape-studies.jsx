import React from 'react';
import {Composition, Easing, interpolate, useCurrentFrame} from 'remotion';

const width = 960;
const height = 540;
const fps = 24;
const clamp = {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'};
const ramp = (time, start, end, easing = Easing.linear) =>
  interpolate(time, [start, end], [0, 1], {...clamp, easing});
const mix = (a, b, progress) => a + (b - a) * progress;
const easeOut = Easing.bezier(0.16, 0.72, 0.24, 1);
const easeInOut = Easing.bezier(0.43, 0.02, 0.2, 1);

function Canvas({background, children}) {
  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      width={width}
      height={height}
      style={{display: 'block', background, fontFamily: 'Segoe UI, sans-serif'}}
    >
      {children}
    </svg>
  );
}

function MassBlock({x, floor, compression}) {
  const side = 57;
  const broad = side * (1 + compression * 0.5);
  const tall = side * (1 - compression);
  const left = x - broad / 2;
  const high = floor - tall;
  return (
    <g>
      <ellipse cx={x + 5} cy={floor + 3} rx={broad * 0.68} ry={7} fill="#292028" opacity=".1" />
      <path d={`M${left} ${high} l13 -10 h${broad} l-13 10Z`} fill="#534451" />
      <path d={`M${left + broad} ${high} l13 -10 v${tall} l-13 10Z`} fill="#2d2532" />
      <rect x={left} y={high} width={broad} height={tall} fill="#3c303e" />
      <path d={`M${left + 8} ${high + 8}h${broad - 16}`} stroke="#7b6676" strokeWidth="1.4" />
    </g>
  );
}

function Weight() {
  const time = useCurrentFrame() / fps;
  const start = 206;
  const finish = 760;
  const lightProgress = ramp(time, 0.5, 1.8, Easing.bezier(0.28, 0.08, 0.36, 1));
  const heavyProgress = interpolate(
    time, [0.5, 0.82, 1.12, 1.46, 1.8, 2.17, 2.26, 2.52, 2.72],
    [0, 0.035, 0.16, 0.36, 0.64, 0.98, 1.026, 0.997, 1], clamp,
  );
  const compression = interpolate(
    time, [2.13, 2.25, 2.43, 2.55, 2.72], [0, 0.27, 0.07, 0.12, 0], clamp,
  );
  const finalLabels = ramp(time, 2.72, 3.02, easeOut);
  const impact = interpolate(time, [2.16, 2.23, 2.58], [0, 1, 0], clamp);
  return (
    <Canvas background="#f6efe2">
      <path d="M64 43H896" stroke="#c8b7ae" />
      <text x="62" y="125" fill="#362735" fontFamily="Georgia, serif" fontSize="76">Вес</text>
      <path d="M64 158H896 M64 474H896" stroke="#c8b7ae" />
      <path d="M206 184V438" stroke="#a797a4" strokeDasharray="3 6" />
      <path d="M760 184V438" stroke="#a24053" strokeWidth="1.5" />
      {[260, 404].map((floor) => (
        <g key={floor}>
          <path d={`M88 ${floor}H872`} stroke="#9c8b99" />
          {[206, 298, 390, 482, 574, 666, 760].map((x) => (
            <path key={x} d={`M${x} ${floor}v9`} stroke={x === finish ? '#a24053' : '#9c8b99'} />
          ))}
        </g>
      ))}
      <path d="M747 427H773 M738 437H782" stroke="#a24053" strokeWidth="2" opacity={impact} />
      <MassBlock x={mix(start, finish, lightProgress)} floor={260} compression={0} />
      <MassBlock x={mix(start, finish, heavyProgress)} floor={404} compression={compression} />
      <g fill="#704153" fontSize="17" opacity={finalLabels}>
        <text x="91" y="294">Лёгкий</text>
        <text x="91" y="438">Тяжёлый</text>
      </g>
    </Canvas>
  );
}

const stations = [
  {x: 147, y: 350, arrival: 0},
  {x: 364, y: 230, arrival: 1.38},
  {x: 590, y: 359, arrival: 2.59},
  {x: 814, y: 218, arrival: 3.88},
];
const relays = [
  {from: 0, to: 1, start: 0.48, elbow: [{x: 238, y: 350}, {x: 315, y: 230}]},
  {from: 1, to: 2, start: 1.66, elbow: [{x: 432, y: 230}, {x: 524, y: 359}]},
  {from: 2, to: 3, start: 2.87, elbow: [{x: 667, y: 359}, {x: 765, y: 218}]},
];

function pointAlong(points, progress) {
  const lengths = points.slice(1).map((point, index) =>
    Math.hypot(point.x - points[index].x, point.y - points[index].y),
  );
  let remaining = lengths.reduce((total, length) => total + length, 0) * progress;
  for (let i = 0; i < lengths.length; i += 1) {
    if (remaining <= lengths[i]) {
      const local = remaining / lengths[i];
      return {x: mix(points[i].x, points[i + 1].x, local), y: mix(points[i].y, points[i + 1].y, local)};
    }
    remaining -= lengths[i];
  }
  return points[points.length - 1];
}

function Relay() {
  const time = useCurrentFrame() / fps;
  return (
    <Canvas background="#17162a">
      <path d="M64 152H896 M64 457H896" stroke="#42415b" />
      <text x="61" y="114" fill="#e9e7ff" fontFamily="Consolas, monospace" fontSize="61" letterSpacing="-3">ПЕРЕДАЧА</text>
      {relays.map(({from, to, start, elbow}) => {
        const end = stations[to].arrival;
        const points = [stations[from], ...elbow, stations[to]];
        const path = points.map(({x, y}, index) => `${index === 0 ? 'M' : 'L'}${x} ${y}`).join(' ');
        const progress = ramp(time, start, end);
        const carry = pointAlong(points, progress);
        return (
          <g key={from}>
            <path d={path} fill="none" stroke="#454361" strokeWidth="3" strokeLinejoin="round" />
            {progress > 0 && (
              <path
                d={path} pathLength="1" strokeDasharray={`${progress} 1`}
                fill="none" stroke="#72c5ba" strokeWidth="3" strokeLinejoin="round"
              />
            )}
            {time >= start && time < end && (
              <g>
                <circle cx={carry.x} cy={carry.y} r="13" fill="#83f5d5" opacity=".18" />
                <circle cx={carry.x} cy={carry.y} r="5.5" fill="#c5ffe5" />
              </g>
            )}
          </g>
        );
      })}
      {stations.map(({x, y, arrival}, index) => {
        const active = time >= arrival;
        const flash = ramp(time, arrival, arrival + 0.55, easeOut);
        return (
          <g key={arrival}>
            {index > 0 && active && flash < 1 && (
              <circle
                cx={x} cy={y} r={mix(23, 47, flash)} fill="none"
                stroke="#84f2cc" strokeWidth="2" opacity={0.66 * (1 - flash)}
              />
            )}
            <circle cx={x} cy={y} r="22" fill="#17162a" stroke={active ? '#79d8c5' : '#686585'} strokeWidth="2" />
            <circle cx={x} cy={y} r={index === 0 ? 8 : 7} fill={active ? '#b7f9de' : '#53516a'} />
          </g>
        );
      })}
    </Canvas>
  );
}

const mapNodes = [
  {label: 'Набережная', tone: '#335caa', x: 173, y: 242, bend: [235, 185, 182, 206], labelSide: 1},
  {label: 'Мост', tone: '#b43f32', x: 398, y: 307, bend: [340, 255, 367, 282], labelSide: 1},
  {label: 'Рынок', tone: '#8150a4', x: 605, y: 221, bend: [415, 317, 487, 204], labelSide: -1},
  {label: 'Площадь', tone: '#286e62', x: 759, y: 346, bend: [418, 379, 556, 293], labelSide: 1},
  {label: 'Сад', tone: '#5a6c30', x: 538, y: 440, bend: [390, 474, 492, 473], labelSide: 1},
];
const mapLinks = [
  {from: 0, to: 1, start: 4.0, duration: 0.7, main: true},
  {from: 1, to: 2, start: 4.55, duration: 0.67, main: true},
  {from: 1, to: 4, start: 4.68, duration: 0.68},
  {from: 2, to: 3, start: 5.12, duration: 0.76, main: true},
  {from: 4, to: 3, start: 5.26, duration: 0.82},
];
const listX = 292;
const rowY = (index) => 178 + index * 64;

function cubic(start, controlA, controlB, end, progress) {
  const remaining = 1 - progress;
  return remaining ** 3 * start + 3 * remaining ** 2 * progress * controlA +
    3 * remaining * progress ** 2 * controlB + progress ** 3 * end;
}

function MapMark({node, index}) {
  const shape = [
    <circle r="10" />,
    <path d="M0 -12L12 0 0 12 -12 0Z" />,
    <rect x="-9.5" y="-9.5" width="19" height="19" rx="2" />,
    <path d="M0 -11L11 -4 8 10H-8L-11 -4Z" />,
    <path d="M0 -12L11 9H-11Z" />,
  ][index];
  return (
    <g transform={`translate(${node.cx} ${node.cy})`}>
      <circle r={index === 1 ? 23 : 16} fill="#eeecf6" />
      {index === 1 && <circle r="20" fill="none" stroke={node.tone} strokeWidth="1.2" />}
      <g fill={node.tone}>{shape}</g>
    </g>
  );
}

function ListToMap() {
  const time = useCurrentFrame() / fps;
  const fold = ramp(time, 1.12, 1.95, easeInOut);
  const mapVisible = ramp(time, 2.35, 3.96, easeInOut);
  const labelsVisible = ramp(time, 4.6, 5.35, easeOut);
  const nodes = mapNodes.map((node, index) => {
    const travel = ramp(time, 1.78 + index * 0.09, 3.82 + index * 0.09, easeInOut);
    return {
      ...node,
      cx: cubic(listX, node.bend[0], node.bend[2], node.x, travel),
      cy: cubic(rowY(index), node.bend[1], node.bend[3], node.y, travel),
    };
  });
  return (
    <Canvas background="#eeecf6">
      <text x="62" y="102" fill="#272944" fontSize="58" fontWeight="650" letterSpacing="-1.9">Из списка в карту</text>
      <path d="M64 128H896 M64 486H896" stroke="#aaa8bf" />
      <text x="896" y="108" textAnchor="end" fill="#5c5578" fontSize="17" opacity={1 - fold}>Список</text>
      <text x="896" y="108" textAnchor="end" fill="#5c5578" fontSize="17" opacity={mapVisible}>Карта</text>
      <g stroke="#d5d2e3" opacity={mapVisible}>
        <path d="M112 201H848 M112 287H848 M112 373H848 M112 451H848" strokeDasharray="2 7" />
        <path d="M142 165V458 M298 165V458 M449 165V458 M600 165V458 M752 165V458" strokeDasharray="2 7" />
      </g>
      {mapNodes.map(({label, tone}, index) => {
        const anchor = index === 1;
        const textOpacity = 1 - ramp(time, 1.1, 1.67, easeOut);
        return (
          <g key={label} opacity={1 - fold}>
            <rect x={listX - 29} y={rowY(index) - 24} width={mix(434, 59, fold)} height="48" rx="6"
              fill={anchor ? '#f3d9d5' : '#f9f7fe'} stroke={anchor ? '#b9695f' : '#cecadb'} />
            <text x={listX + 41} y={rowY(index) + 6} fill={anchor ? tone : '#272944'}
              fontSize="19" fontWeight={anchor ? 650 : 450} opacity={textOpacity}>{label}</text>
          </g>
        );
      })}
      {mapLinks.map(({from, to, start, duration, main}) => {
        const draw = ramp(time, start, start + duration, easeInOut);
        const initial = nodes[from];
        const final = nodes[to];
        const middleX = (initial.cx + final.cx) / 2;
        if (draw === 0) return null;
        return (
          <path
            key={`${from}-${to}`}
            d={`M${initial.cx} ${initial.cy} C${middleX} ${initial.cy}, ${middleX} ${final.cy}, ${final.cx} ${final.cy}`}
            pathLength="1" strokeDasharray={`${draw} 1`} fill="none" strokeLinecap="round"
            stroke={main ? '#a65b57' : '#9086ac'} strokeWidth={main ? 2.7 : 2}
          />
        );
      })}
      {nodes.map((node, index) => (
        <g key={node.label}>
          <MapMark node={node} index={index} />
          <text x={node.cx} y={node.cy + node.labelSide * (index === 1 ? 42 : 36)}
            fill={node.tone} fontSize="18" fontWeight={index === 1 ? 700 : 600}
            textAnchor="middle" opacity={labelsVisible}>{node.label}</text>
        </g>
      ))}
    </Canvas>
  );
}

const focusTiles = [
  {paper: '#d7ddba', ink: '#435a36', mark: <path d="M-38 24L0 -35 38 24Z" />},
  {paper: '#edab8d', ink: '#873936', mark: <circle r="28" />},
  {paper: '#bccdd5', ink: '#3a576c', mark: <path d="M-35 18V-20H2V18ZM9 18V-20H35V18Z" />},
  {paper: '#dfcfb1', ink: '#735739', mark: <path d="M-37 -21H37V-12H-37ZM-37 -5H37V4H-37ZM-37 11H37V20H-37Z" />},
  {paper: '#dcc7d9', ink: '#765375', mark: <path d="M0 -30L32 0 0 30 -32 0Z" />},
  {paper: '#c5d1c6', ink: '#3a6656', mark: <path d="M-35 7Q-35 -22 -15 -22Q0 -22 0 7Q0 24 -16 24Q-35 24 -35 7ZM3 -7Q3 -24 19 -24Q38 -24 38 -7Q38 22 18 22Q3 22 3 -7Z" />},
];

function FocusTile({tile, x, y, w, h, raised = 0}) {
  const scale = Math.min(w / 242, h / 118);
  return (
    <g>
      {raised > 0 && (
        <rect x={x + 3} y={y + 4 + raised * 5} width={w} height={h} rx="3"
          fill="#493930" opacity={raised * 0.17} />
      )}
      <rect x={x} y={y} width={w} height={h} rx="3"
        fill={tile.paper} stroke={raised > 0 ? '#873936' : '#fff8e8'} strokeWidth={1 + 1.3 * raised} />
      <g transform={`translate(${x + w / 2} ${y + h / 2}) scale(${scale})`} fill={tile.ink}>
        {tile.mark}
      </g>
    </g>
  );
}

function NewFocus() {
  const time = useCurrentFrame() / fps;
  const lead = ramp(time, 0.7, 1.25, easeOut);
  const makeSpace = ramp(time, 1.05, 1.75, easeInOut);
  const lower = ramp(time, 1.22, 1.75, easeInOut);
  const slide = ramp(time, 1.76, 2.65, easeInOut);
  const expand = ramp(time, 2.29, 3.56, easeInOut);
  const settle = ramp(time, 3.38, 4.03, easeOut);
  const boxes = [
    {x: 79, y: 164, w: 244, h: 118},
    {x: 358, y: 164, w: 244, h: 118},
    {x: 637, y: 164, w: 244, h: 118},
    {x: 79, y: mix(324, 295, makeSpace), w: 244, h: mix(118, 80, makeSpace)},
    {x: mix(358, 79, slide), y: mix(324, 389, lower), w: 244, h: mix(118, 80, lower)},
    {x: 637, y: 324, w: 244, h: 118},
  ];
  boxes[1] = {
    x: mix(358, 338, expand),
    y: 164 - 12 * lead - 5 * expand + 5 * settle,
    w: mix(244, 284, expand),
    h: mix(118, 311, expand),
    raised: lead * (1 - 0.63 * settle),
  };

  return (
    <Canvas background="#eae2ce">
      <text x="59" y="105" fill="#302c26" fontFamily="Georgia, serif" fontSize="57">Новое главное</text>
      <path d="M64 129H896 M64 497H896" stroke="#a89c83" />
      {boxes.map((box, index) => (
        <FocusTile key={index} tile={focusTiles[index]} {...box} />
      ))}
    </Canvas>
  );
}

const syncChannels = [
  {name: 'А', offset: 0.12, drift: -0.19, tone: '#91f3c2'},
  {name: 'Б', offset: -0.25, drift: 0.045, tone: '#ffd47e'},
  {name: 'В', offset: 0.38, drift: 0.18, tone: '#efaec0'},
];
const syncRate = 1.08;
const alignAt = 3.86;
const traceSeconds = 2.62;
const traceLeft = 124;
const traceRight = 857;
const channelLine = (index) => 224 + index * 103;

function sampleSignal(time, channel) {
  // Preserve a continuous phase while converging to the common oscillator.
  const aligning = ramp(time, 0.7, alignAt);
  const drift = (1 - aligning) ** 2 * (1 + 2 * aligning);
  const phase = time * syncRate + (channel.offset + time * channel.drift) * drift;
  const cycle = phase - Math.round(phase);
  return Math.exp(-((cycle / 0.11) ** 2)) - 0.18 * Math.exp(-(((cycle - 0.23) / 0.085) ** 2));
}

function SignalTrace({time, channel, index}) {
  const line = channelLine(index);
  const pointCount = 144;
  const samples = Array.from({length: pointCount + 1}, (_, step) => {
    const position = step / pointCount;
    const at = time - traceSeconds * (1 - position);
    const x = mix(traceLeft, traceRight, position);
    const y = line - 41 * sampleSignal(at, channel);
    return `${step === 0 ? 'M' : 'L'}${x.toFixed(1)} ${y.toFixed(1)}`;
  });
  return (
    <g>
      <path d={`M${traceLeft} ${line}H${traceRight}`} stroke="#385962" strokeDasharray="3 5" />
      <text x="99" y={line + 4} fill={channel.tone}
        fontFamily="Consolas, monospace" fontSize="15" textAnchor="middle">{channel.name}</text>
      <path d={samples.join(' ')} fill="none" stroke={channel.tone} strokeWidth="2.8"
        strokeLinecap="round" strokeLinejoin="round" />
      <circle cx={traceRight} cy={line - 41 * sampleSignal(time, channel)} r="4" fill={channel.tone} />
    </g>
  );
}

function Sync() {
  const time = useCurrentFrame() / fps;
  const resolved = ramp(time, 4.34, 4.93, easeOut);
  const beat = Math.floor(time * syncRate);
  return (
    <Canvas background="#102933">
      <text x="61" y="108" fill="#e8f0e6" fontSize="63" fontWeight="650" letterSpacing="-1.8">Настройка</text>
      <g opacity={resolved}>
        <path d="M799 101H826" stroke="#baf2d7" strokeWidth="2" />
        <text x="894" y="106" textAnchor="end" fill="#baf2d7" fontSize="16">В такт</text>
      </g>
      <rect x="64" y="135" width="832" height="337" rx="5" fill="#0b2028" stroke="#4e6e73" />
      <g stroke="#2b4851">
        <path d="M64 248H896 M64 351H896" />
        {Array.from({length: 10}, (_, index) => {
          const x = mix(traceLeft, traceRight, index / 9);
          return <path key={x} d={`M${x} 152V454`} strokeDasharray="2 6" />;
        })}
      </g>
      {[beat - 1, beat].map((number) => {
        const at = number / syncRate;
        const age = time - at;
        if (at < alignAt || age > traceSeconds) return null;
        const x = mix(traceRight, traceLeft, age / traceSeconds);
        const glow = ramp(age, 0, 0.17, easeOut) * (1 - age / traceSeconds);
        return (
          <g key={number} opacity={glow}>
            <rect x={x - 4} y="153" width="8" height="299" fill="#92d4bd" opacity=".14" />
            <path d={`M${x} 153V452`} stroke="#82b4b2" strokeWidth="1" />
          </g>
        );
      })}
      {syncChannels.map((channel, index) => (
        <SignalTrace key={channel.name} time={time} channel={channel} index={index} />
      ))}
      <path d="M64 494H896" stroke="#4e6e73" />
    </Canvas>
  );
}

export function ShapeStudies() {
  return (
    <>
      <Composition id="MiraWeight" component={Weight} width={width} height={height} fps={fps} durationInFrames={fps * 4} />
      <Composition id="MiraRelay" component={Relay} width={width} height={height} fps={fps} durationInFrames={fps * 5} />
      <Composition id="MiraListToMap" component={ListToMap} width={width} height={height} fps={fps} durationInFrames={fps * 8} />
      <Composition id="MiraNewFocus" component={NewFocus} width={width} height={height} fps={fps} durationInFrames={fps * 5} />
      <Composition id="MiraSync" component={Sync} width={width} height={height} fps={fps} durationInFrames={fps * 6} />
    </>
  );
}
