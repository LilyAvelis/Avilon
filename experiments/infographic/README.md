# От пробы к библиотеке

В `scene.jsx` каждый этап задаётся подписью, картинкой, координатами и секундой
появления. Стрелка начинает рисоваться за 0,48 секунды до следующей карточки.
Иллюстрации взяты из оформленного отчёта МИРЫ за 23 ноября.

Для пересборки из корня МИРЫ нужны Node.js и FFmpeg в PATH.

```powershell
npm.cmd --prefix experiments/infographic ci
npm.cmd --prefix experiments/infographic run render
cargo build --offline --locked --release --target-dir target/atlas --bin mira-atlas
```

Remotion экспортирует `target/infographic/mira-cycle.mp4`. Из того же ролика
собирается `assets/atlas/mira-cycle.frames`, который атлас включает в сборку
для показа во вкладке «Анимация».

## Озвучка

Запусти `Record voice.ps1` в локальном PowerShell. При первом запуске ключ
проекта вводится в скрытом приглашении. После успешной записи скрипт
сохраняет его в `%LOCALAPPDATA%/Mira/openai-key.dpapi` с шифрованием Windows
DPAPI для текущего пользователя. Следующие запуски используют сохранённый
ключ. Для замены ключа добавь `-ReplaceKey`.
Готовые реплики сохраняются в `target/infographic/voice`.

```powershell
node experiments/infographic/mix-voice.cjs
```

Сборщик совмещает реплики с моментами появления карточек из `timing.json`.
Он создаёт озвученный `mira-cycle-voiced.mp4` и отдельную дорожку
`mira-cycle.wav`. Для воспроизведения во вкладке «Анимация» на Windows
положи WAV рядом с исполняемым файлом атласа.

Длинная версия берёт текст и время появления карточек из `explained.json`.
Для неё запусти `Record voice.ps1 -Explained`, затем выполни:

```powershell
npm.cmd --prefix experiments/infographic run render -- explained
node experiments/infographic/mix-voice.cjs explained
```

Файлы этой версии появятся в `target/infographic/explained`. Для атласа
скопируй её WAV рядом с исполняемым файлом под именем `mira-cycle-explained.wav`.

## Геометрические сцены Фенны

Пять немых сцен заданы в `shape-studies.jsx`. В разделе «Анимация» ролики
можно выбрать в содержании справа или перелистать стрелками. Пробел
повторяет текущую сцену. MP4 можно получить отдельно от атласа:

```powershell
npm.cmd --prefix experiments/infographic run render -- studies
```

Ролики появятся в `target/infographic/studies`, а кадры для встраивания
в атлас — в `assets/atlas`.
