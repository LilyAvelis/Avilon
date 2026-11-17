# Решения Katran 3.8 для склада Avilon

Дата: 18 ноября 2026. Аудит: Azuros Claudapi, Sol/xhigh; интеграция выводов: Polly.
Источник: Katran v3.8.0, commit 14e737348af0f6a188d2eb89f8403e06ec3e1c87.
Сопоставлены исходники Katran и C:\Avilon. Это аудит и предложение пополнения
склада; перенос кода, синхронизация пакетов и изменение Katran не выполнялись.

## Назначение Avilon

Уточнение Лилии: Avilon — склад готовых решений из разных проектов. Подключаемые
Rust-пакеты являются одним из форматов материала. Текущие приложения могут
сохранять собственные проверенные копии; их переподключение к Avilon — отдельная
задача. Выделение общего API оправдано там, где оно облегчает повторное применение.

Полезны три формата:

- **Готовый компонент:** самостоятельный модуль или существующий crate с тестами.
- **Образец реализации:** рабочий код с явными зависимостями и инструкцией адаптации.
- **Рецепт и проверки:** архитектурный контракт, воспроизводимые fixtures и тесты
  для интеграций, которым нужны собственные хранение, окна или данные приложения.

Не требуется заранее превращать каждый материал в универсальный framework.
Можно сохранить рабочий вариант и его ограничения, а обобщить при появлении
потребителя. Исходник, лицензии, проверенный сценарий и зависимости должны быть
понятны следующей команде.

## Что уже извлечено

C:\Avilon уже содержит четыре пакета, извлечённые из Katran:

| Пакет | Содержимое |
| --- | --- |
| avilon-actions | Нормализация сочетаний и политика назначения/конфликтов |
| avilon-egui | Семантические палитры, settings layout, ActionRow/KeyCaps, материалы и оптика |
| avilon-windows | Acrylic и оформление окон с явными ошибками/откатом |
| avilon-media | Ограниченное видео-воспроизведение, worker, текстуры и управление остановкой |

README Avilon указывает происхождение из checkpoint f01c33b и извлечение
11 ноября 2026. Каталог crates в Katran — отдельная проверенная копия.
Склад уже существует; повторное извлечение этих же компонентов не нужно.

В копии Katran появились новые изменения, отсутствующие в C:\Avilon:

1. avilon-actions/src/palette.rs и navigation.rs — многословный поиск команд,
   выбор, wraparound и курсор по стабильному ID. Существующая политика назначения
   хоткеев совпадает; это дополнение двух модулей и их экспортов/документации.
2. avilon-egui/src/theme.rs — accent tint для всей семантической палитры.
3. avilon-media/src/lib.rs — Windows Job Object для остановки назначенного дерева
   decoder-процессов и bounded latest-frame slot для свежего кадра.

avilon-windows совпадает побайтово. У media остаётся ограничение: тест остановки
назначенного процесса не доказывает отсутствие окна между spawn и назначением
процесса в Job. Это ограничение следует перенести вместе с материалом.
В docs/design-sdk.md также остались примеры зависимостей на ../Katran/crates;
при будущем обновлении склада их стоит привести к его самостоятельному назначению.

## Новые материалы

Пути ниже относятся к корню исходного Katran. Готовность означает пригодность
к выделению; проверки в отдельном Avilon ещё не выполнялись.

| Материал | Исходники | Подходящий формат и границы |
| --- | --- | --- |
| Everything: получение кандидатов и ранжирование | src/everything.rs, src/folder_search_rank.rs | Готовое чистое ранжирование плюс образец Windows-адаптера. Exact/name/path/approximate потоки объединяются до лимита. Helper path, каталог приложения и запуск ES задаёт потребитель. Результат должен сообщать incomplete. |
| Unicode-каталог эмодзи | src/pod_metadata.rs, src/pods.rs | Компонент данных/парсера с категориями, поиском и страницами. Возвращать sequence/name/group/subgroup/version. Katran Event, SQL, отрицательные виртуальные ID и копирование остаются образцом адаптации. Шрифт ядру не нужен. |
| Теги по областям и атомарная перестановка | src/tag_slots.rs, src/tag_service.rs | Рабочий образец и storage recipe: permutation, revision/CAS, сохранение назначений и изоляция областей. Девять слотов и связь с pin — явно описанные правила Katran. Для общего компонента идентичность тега отделяется от позиции. |
| Сортировка и согласованные страницы | src/sorting.rs, src/catalogue_query.rs, src/catalogue_snapshot.rs | Чистые natural keys и deterministic shuffle — компонент; SQL grouping/keyset pagination — рецепт. Порядок задаётся до LIMIT, ties стабильны, cursor привязан к query/sort/filter и одному snapshot. |
| Кэш изображений без мигания | src/bin/images.rs, src/bin/api_client.rs | Образец cache policy с тестами: namespace/item/content revision/edge и epoch заданий. Переносить resident decoded textures при метаданных, отвергать поздние ответы. Decoder/UI texture/metadata proof поставляет потребитель. |
| Запись физических хоткеев и Windows-регистрация | src/bin/shortcuts_ui.rs, src/overlay.rs, src/overlay_windows.rs | Reducer записи и Win32-адаптер. Удержание модификаторов и пунктуация, независимость от раскладки, явный результат регистрации. Каталог команд, настройки и состояния вкладок принадлежат потребителю. |
| Надёжная вставка в предыдущее окно | src/foreground_windows.rs, src/overlay.rs, src/copy_out.rs | Рецепт сложной Windows-интеграции с synthetic fixtures: immutable paste lease, отмена при смене визита/фокуса, owned Clipboard publication и proof после teardown. Export lifetime/selection/quota — приложение. Высокая сложность выделения. |
| Установка optional-компонентов в видимом терминале | src/ai_components.rs, src/ai_component_bundle.rs, scripts/ai-components | Образец lifecycle Install/Verify, Launcher boundary, content-addressed helpers, receipts и interrupted/unknown outcomes. Конкретные модели, scripts, profile layout и тексты остаются параметрами материала. |
| Ограниченный semantic worker и derived cache | src/folder_semantic.rs, scripts/folder-semantic/worker.py | Reference implementation: resident worker, timeout/cancellation, snapshot pages, повторное вычисление изменившихся descriptors. Everything/favorites, E5 prefixes, модель, Python/ORT/CUDA — отдельные зависимости. Высокая сложность. |

Обычный поиск Unicode-каталога сейчас использует английские Unicode names и
названия групп. Русские/английские CLDR-аннотации относятся к AI-индексу; их
не следует выдавать за уже существующий многоязычный literal search.
Приближённое извлечение Everything ограничено в том числе ASCII substitutions;
это не доказательство полного edit-distance recall по всему диску.

## Рекомендуемый порядок пополнения

1. Вернуть в существующие компоненты palette/navigation, tint и media corrections.
   Это наиболее готовый материал, без нового слоя абстракции.
2. Сохранить Everything retrieval/ranking и Unicode emoji catalogue как отдельные
   самостоятельные решения, с маленьким нейтральным примером каждого.
3. Сохранить теги и сортировку вместе с транзакциями, revisions и cursor contract
   как рабочий набор рецептов. Отдельный универсальный tag engine сейчас не нужен.
4. Добавить образец image cache и проверки, которые воспроизводят исправленное
   мигание и отбрасывание запоздалых результатов.
5. Windows paste, видимый installer и semantic worker оформить более подробными
   reference implementations: здесь важны границы платформы и внешних зависимостей.

## Что переносить вместе с решением

У каждого материала полезно хранить исходный проект/commit и список файлов,
задачу и рабочий пример, обязательные зависимости, тесты, ограничения и лицензию.
Минимальная проверка — материал работает на вымышленных данных без профиля Katran.
Сборка и интеграция в будущий проект выполняются отдельно от пополнения склада.

Ключевые fixtures:

- Everything: exact hit при усечённом broad stream, Unicode/escaping, dedup,
  stable ties, ошибка helper отдельно от пустой выдачи.
- Emoji: fully-qualified sequences, группы на границах страниц, cursor другой
  версии/фильтра и отсутствие mutation paths для виртуальных записей.
- Tags: полная permutation, повторы ID, scope isolation, stale save после reorder,
  сохранение назначений, rollback при ошибке и persistent revision против ABA.
- Sorting: grouping до LIMIT, stable ties и stale cursor после внешнего writer.
- Images: тот же TextureId при metadata edit, late-reply rejection, новая content
  revision, смена namespace, Shell visuals и bounded queues.
- Windows paste: sent подтверждает отправку ввода, а не принятие файла любым
  редактором. Имеется ручная приёмка конкретного исправленного Notion-сценария.
- AI: наличие runtime не подтверждает inference; наличие semantic snapshot не
  означает полный индекс диска. GPU Verify должен доказывать реальные CUDA nodes.

## Границы содержимого и лицензий

В материалах сохраняются исходные LICENSE и notices. Собственный Rust-код Katran
и Avilon снабжён MIT-лицензией LilyAvelis. Unicode/CLDR имеют отдельные Unicode
License V3 notices; ES helper — отдельный MIT notice voidtools и требует
работающего Everything с IPC; Twemoji graphics — CC BY 4.0.

Формат хранения решений не требует включать Katran schema v13, миграции,
Service/dispatcher, реальные профили, экспорты, историю папок, журналы установок,
векторные кэши и готовые virtualenv. Artwork, Apple font, личные видео и шрифты
не являются необходимой частью каталога эмодзи. Модель и runtime могут быть
описанными optional-зависимостями с собственными version/license manifests.

Итог: Avilon уже выполняет роль склада части решений Katran. Его имеет смысл
пополнять сильными, проверенными материалами сейчас, пока контекст свежий.
Текущий Katran продолжает использовать свою принятую версию; перенос склада
сам по себе не требует менять его зависимости или переписывать приложение.