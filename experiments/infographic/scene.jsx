import React from 'react';
import {Composition, Img, interpolate, registerRoot, useCurrentFrame} from 'remotion';
import experimentArt from './experiment.jpg';
import reviewArt from './review.jpg';
import libraryArt from './library.jpg';
import reuseArt from './reuse.jpg';
import revealTimes from './timing.json';
import explainedCut from './explained.json';
import {ShapeStudies} from './shape-studies.jsx';

const fps = 24;
const stages = [
  {x: 60, y: 134, title: 'Пробуем эффект', detail: 'Собираем сцену, которую можно потрогать.', art: experimentArt},
  {x: 510, y: 134, title: 'Смотрим, что сработало', detail: 'Проверяем движение и выбираем удачное.', art: reviewArt},
  {x: 510, y: 339, title: 'Сохраняем в Avilon', detail: 'У эффекта появляется программный интерфейс.', art: libraryArt},
  {x: 60, y: 339, title: 'Берём в новый проект', detail: 'Готовый эффект становится частью новой сцены.', art: reuseArt},
];
const links = [
  {from: [453, 207], to: [503, 207]},
  {from: [705, 287], to: [705, 332]},
  {from: [503, 412], to: [457, 412]},
];
const progress = (time, at, duration) =>
  interpolate(time, [at, at + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

function Cycle({explained = false}) {
  const time = useCurrentFrame() / fps;
  const starts = explained ? explainedCut.starts : revealTimes;
  return (
    <div style={{position: 'absolute', inset: 0, background: '#eee9dc', color: '#202a25', fontFamily: 'Segoe UI, sans-serif'}}>
      <svg viewBox="0 0 960 540" width="960" height="540" style={{position: 'absolute', inset: 0}}>
        <text x="60" y="48" fill="#77492e" fontSize="14" letterSpacing="3">МИРА / AVILON</text>
        <text x="58" y="97" fontFamily="Georgia, serif" fontSize="46">От пробы к библиотеке</text>
        <path d="M60 113H900 M60 513H900" stroke="#b9b6a6" />
        {links.map(({from, to}, index) => {
          const draw = progress(time, starts[index + 1] - (explained ? 0 : 0.48), 0.44);
          if (draw === 0) return null;
          const dx = to[0] - from[0];
          const dy = to[1] - from[1];
          const length = Math.hypot(dx, dy);
          const ux = dx / length;
          const uy = dy / length;
          const x = from[0] + dx * draw;
          const y = from[1] + dy * draw;
          const head = Math.min(7, length * draw / 2);
          return (
            <path key={index}
              d={`M${from[0]} ${from[1]}L${x} ${y} M${x - ux * head - uy * head / 2} ${y - uy * head + ux * head / 2}L${x} ${y}L${x - ux * head + uy * head / 2} ${y - uy * head - ux * head / 2}`}
              stroke="#8c5738" strokeWidth="2.4" fill="none" strokeLinecap="round" strokeLinejoin="round" />
          );
        })}
      </svg>
      {stages.map(({x, y, title, detail, art}, index) => {
        const reveal = progress(time, starts[index], 0.44);
        const offset = 10 * (1 - reveal) ** 3;
        const engraving = explained && index === 0;
        return (
          <div key={title} style={{
            position: 'absolute', left: x, top: y + offset, width: 390, height: 146,
            boxSizing: 'border-box', padding: '10px 12px', border: '1px solid #b9b6a6',
            borderRadius: 9, background: '#faf6ec', opacity: reveal,
          }}>
            <div style={{height: 74, overflow: 'hidden', borderRadius: 4}}>
              <Img src={engraving ? libraryArt : art} style={{
                width: engraving ? '142%' : '100%',
                height: '100%', objectFit: 'cover', display: 'block',
                marginLeft: engraving ? '-42%' : 0,
              }} />
            </div>
            <div style={{fontFamily: 'Georgia, serif', fontSize: 25, marginTop: 5}}>{title}</div>
            <div style={{fontSize: 14, color: '#50574a', marginTop: 1}}>{explained ? explainedCut.details[index] : detail}</div>
          </div>
        );
      })}
    </div>
  );
}

registerRoot(() => (
  <>
    <Composition id="MiraCycle" component={Cycle} width={960} height={540}
      fps={fps} durationInFrames={fps * 9} />
    <Composition id="MiraCycleExplained" component={Cycle} width={960} height={540}
      fps={fps} durationInFrames={fps * explainedCut.seconds} defaultProps={{explained: true}} />
    <ShapeStudies />
  </>
));
