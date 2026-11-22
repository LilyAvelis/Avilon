use std::time::Duration;

use eframe::egui::{
    self, Align2, Color32, FontId, Id, Key, Mesh, Painter, Pos2, Rect, RichText, Shape, Stroke,
    Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

use crate::{consume_key, install_fonts, normalize};

const INK: Color32 = Color32::from_rgb(230, 246, 255);
const QUIET: Color32 = Color32::from_rgb(163, 185, 210);
const DARK: Color32 = Color32::from_rgb(6, 10, 23);
const LINE: Color32 = Color32::from_rgb(43, 59, 86);

struct Phase {
    name: &'static str,
    base: Color32,
    side: Color32,
    rim: Color32,
    signal: Color32,
}

const PHASES: [Phase; 3] = [
    Phase {
        name: "Импульс",
        base: Color32::from_rgb(15, 95, 80),
        side: Color32::from_rgb(38, 105, 251),
        rim: Color32::from_rgb(71, 255, 205),
        signal: Color32::from_rgb(215, 255, 92),
    },
    Phase {
        name: "Призма",
        base: Color32::from_rgb(53, 40, 127),
        side: Color32::from_rgb(201, 57, 246),
        rim: Color32::from_rgb(127, 165, 255),
        signal: Color32::from_rgb(255, 211, 98),
    },
    Phase {
        name: "Спектр",
        base: Color32::from_rgb(102, 29, 116),
        side: Color32::from_rgb(72, 83, 252),
        rim: Color32::from_rgb(242, 131, 255),
        signal: Color32::from_rgb(116, 251, 241),
    },
];

struct Task {
    name: &'static str,
    detail: &'static str,
    pos: [f32; 2],
    needs: &'static [usize],
    seconds: f32,
}

const TASKS: [Task; 6] = [
    Task {
        name: "Источник",
        detail: "Собрать исходники в рабочем каталоге.",
        pos: [0.14, 0.53],
        needs: &[],
        seconds: 2.0,
    },
    Task {
        name: "Индекс",
        detail: "Построить индекс материалов для атласа.",
        pos: [0.38, 0.53],
        needs: &[0],
        seconds: 3.0,
    },
    Task {
        name: "Геометрия",
        detail: "Подготовить векторные слои и проверить границы фигур.",
        pos: [0.62, 0.27],
        needs: &[1],
        seconds: 8.0,
    },
    Task {
        name: "Палитра",
        detail: "Собрать цветовые варианты и проверить контраст.",
        pos: [0.62, 0.76],
        needs: &[1],
        seconds: 6.0,
    },
    Task {
        name: "Сборка",
        detail: "Объединить геометрию и палитру в один пакет.",
        pos: [0.86, 0.43],
        needs: &[2, 3],
        seconds: 5.0,
    },
    Task {
        name: "Экспорт",
        detail: "Подготовить пакет для передачи.",
        pos: [0.86, 0.76],
        needs: &[4],
        seconds: 3.0,
    },
];

const INITIAL: [f32; TASKS.len()] = [1.0, 1.0, 0.64, 0.0, 0.0, 0.0];
const SIGNAL_TRAVEL: f64 = 0.66;
const UNLOCK_GLOW: f64 = 0.78;

#[derive(Clone, Copy)]
struct Completion {
    started: f64,
    unlocks: [bool; TASKS.len()],
}

impl Completion {
    fn travel(self, now: f64) -> Option<f32> {
        let elapsed = now - self.started;
        (0.0..SIGNAL_TRAVEL)
            .contains(&elapsed)
            .then_some((elapsed / SIGNAL_TRAVEL) as f32)
    }

    fn arrival(self, target: usize, now: f64) -> f32 {
        let elapsed = now - self.started - SIGNAL_TRAVEL;
        if self.unlocks[target] && (0.0..UNLOCK_GLOW).contains(&elapsed) {
            (1.0 - elapsed / UNLOCK_GLOW).powi(2) as f32
        } else {
            0.0
        }
    }

    fn expires_at(self) -> f64 {
        self.started
            + SIGNAL_TRAVEL
            + if self.unlocks.contains(&true) {
                UNLOCK_GLOW
            } else {
                0.0
            }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Done,
    Running,
    Paused,
    Ready,
    Blocked,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Self::Done => "Готово",
            Self::Running => "В работе",
            Self::Paused => "Пауза",
            Self::Ready => "К запуску",
            Self::Blocked => "Ожидает",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Modal {
    Commands,
    Search,
    Settings,
}

#[derive(Clone, Copy)]
enum Command {
    Run,
    Select(usize),
    Phase(usize),
    Settings,
    Reset,
}

struct Nolox {
    selected: usize,
    phase: usize,
    progress: [f32; TASKS.len()],
    worker: Option<(usize, f64, f32)>,
    completions: [Option<Completion>; TASKS.len()],
    effects: bool,
    modal: Option<Modal>,
    query: String,
    cursor: usize,
    focus_search: bool,
    restore_focus: Option<Id>,
}

impl Nolox {
    fn new(ctx: &egui::Context) -> Self {
        install_fonts(ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(INK);
        visuals.weak_text_color = Some(QUIET);
        visuals.panel_fill = DARK;
        ctx.set_visuals(visuals);
        Self {
            selected: 2,
            phase: 0,
            progress: INITIAL,
            worker: None,
            completions: [None; TASKS.len()],
            effects: true,
            modal: None,
            query: String::new(),
            cursor: 0,
            focus_search: false,
            restore_focus: None,
        }
    }

    fn status(&self, index: usize) -> Status {
        if self.progress[index] >= 1.0 {
            Status::Done
        } else if self.worker.is_some_and(|(active, _, _)| index == active) {
            Status::Running
        } else if self.progress[index] > 0.0 {
            Status::Paused
        } else if TASKS[index]
            .needs
            .iter()
            .all(|&need| self.progress[need] >= 1.0)
        {
            Status::Ready
        } else {
            Status::Blocked
        }
    }

    fn run_label(&self) -> Option<&'static str> {
        match self.status(self.selected) {
            Status::Running => Some("Пауза"),
            Status::Paused if self.worker.is_none() => Some("Продолжить"),
            Status::Ready if self.worker.is_none() => Some("Запустить"),
            _ => None,
        }
    }

    fn tick(&mut self, now: f64) {
        for completion in &mut self.completions {
            if completion.is_some_and(|event| event.expires_at() <= now) {
                *completion = None;
            }
        }
        if let Some((index, start, progress)) = self.worker {
            self.progress[index] =
                (progress + (now - start) as f32 / TASKS[index].seconds).min(1.0);
            if self.progress[index] >= 1.0 {
                self.worker = None;
                if self.effects && TASKS.iter().any(|task| task.needs.contains(&index)) {
                    let unlocks = std::array::from_fn(|target| {
                        TASKS[target].needs.contains(&index) && self.status(target) == Status::Ready
                    });
                    self.completions[index] = Some(Completion {
                        started: now,
                        unlocks,
                    });
                }
            }
        }
    }

    fn open(&mut self, ctx: &egui::Context, modal: Modal) {
        if self.modal == Some(modal) {
            self.close(ctx);
            return;
        }
        if self.modal.is_none() {
            self.restore_focus = ctx.memory(|memory| memory.focused());
        }
        self.modal = Some(modal);
        self.query.clear();
        self.cursor = 0;
        self.focus_search = modal != Modal::Settings;
    }

    fn close(&mut self, ctx: &egui::Context) {
        self.modal = None;
        ctx.memory_mut(|memory| {
            if let Some(focus) = self.restore_focus.take() {
                memory.request_focus(focus);
            } else if let Some(focus) = memory.focused() {
                memory.surrender_focus(focus);
            }
        });
    }

    fn execute(&mut self, ctx: &egui::Context, command: Command, now: f64) {
        if matches!(command, Command::Settings) {
            self.open(ctx, Modal::Settings);
            return;
        }
        if self.modal.is_some() {
            self.close(ctx);
        }
        match command {
            Command::Run if self.run_label().is_some() => {
                self.worker = if self.worker.is_some() {
                    None
                } else {
                    Some((self.selected, now, self.progress[self.selected]))
                };
            }
            Command::Select(index) => {
                self.selected = index;
                ctx.memory_mut(|memory| memory.request_focus(task_id(index)));
            }
            Command::Phase(index) => self.phase = index,
            Command::Reset => {
                self.worker = None;
                self.progress = INITIAL;
                self.completions.fill(None);
                self.selected = 2;
                ctx.memory_mut(|memory| memory.request_focus(task_id(2)));
            }
            _ => {}
        }
    }

    fn keyboard(&mut self, ctx: &egui::Context, now: f64) {
        for (key, modal) in [
            (Key::K, Modal::Commands),
            (Key::F, Modal::Search),
            (Key::Comma, Modal::Settings),
        ] {
            if consume_key(ctx, key, true) > 0 {
                self.open(ctx, modal);
                return;
            }
        }
        if self.modal.is_some() {
            if consume_key(ctx, Key::Escape, false) > 0 {
                self.close(ctx);
            }
            return;
        }
        for (key, direction) in [
            (Key::ArrowLeft, vec2(-1.0, 0.0)),
            (Key::ArrowRight, vec2(1.0, 0.0)),
            (Key::ArrowUp, vec2(0.0, -1.0)),
            (Key::ArrowDown, vec2(0.0, 1.0)),
        ] {
            let presses = consume_key(ctx, key, false);
            if presses > 0 {
                ctx.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
            }
            for _ in 0..presses {
                let [x, y] = TASKS[self.selected].pos;
                let nearest = TASKS
                    .iter()
                    .enumerate()
                    .filter_map(|(index, task)| {
                        let delta = vec2(task.pos[0] - x, task.pos[1] - y);
                        let forward = delta.dot(direction);
                        (forward > 0.01).then(|| {
                            let cross = delta - direction * forward;
                            (index, forward + 2.0 * cross.length())
                        })
                    })
                    .min_by(|(_, a), (_, b)| a.total_cmp(b));
                if let Some((index, _)) = nearest {
                    self.execute(ctx, Command::Select(index), now);
                }
            }
        }
        let graph_focus = ctx.memory(|memory| {
            memory.focused().is_none() || (0..TASKS.len()).any(|i| memory.has_focus(task_id(i)))
        });
        if graph_focus && consume_key(ctx, Key::Enter, false) > 0 {
            self.execute(ctx, Command::Run, now);
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let now = ui.input(|input| input.time);
        self.tick(now);
        self.keyboard(ui.ctx(), now);
        if self.completions.iter().any(Option::is_some) {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        } else if self.worker.is_some() {
            ui.ctx()
                .request_repaint_after(Duration::from_millis(if self.effects { 24 } else { 100 }));
        }

        let area = ui.max_rect();
        let phase = &PHASES[self.phase];
        let p = ui.painter_at(area);
        p.rect_filled(area, 0, DARK);
        let outer = area.shrink2(vec2(28.0, 22.0));
        let graph = Rect::from_min_max(
            outer.min + vec2(0.0, 110.0),
            outer.right_bottom() - vec2((outer.width() * 0.28).clamp(256.0, 335.0) + 16.0, 51.0),
        );
        let inspector = Rect::from_min_max(
            pos2(graph.right() + 16.0, graph.top()),
            pos2(outer.right(), graph.bottom()),
        );
        let header = outer.min + vec2(0.0, 7.0);
        p.text(
            header,
            Align2::LEFT_TOP,
            "Nolox",
            FontId::proportional(56.0),
            INK,
        );
        p.text(
            header + vec2(168.0, 27.0),
            Align2::LEFT_CENTER,
            "Очередь сборки",
            FontId::proportional(23.0),
            INK,
        );
        label(
            &p,
            header + vec2(170.0, 51.0),
            Align2::LEFT_CENTER,
            "Мира / Атлас интерфейсов",
            QUIET,
        );

        for (i, option) in PHASES.iter().enumerate() {
            let rect = Rect::from_min_size(
                pos2(outer.right() - 294.0 + 100.0 * i as f32, header.y + 21.0),
                vec2(94.0, 31.0),
            );
            if control(
                ui,
                rect,
                ("phase", i),
                option.name,
                self.phase == i,
                true,
                option.rim,
            ) {
                self.phase = i;
            }
        }
        let finished = self.progress.iter().filter(|&&value| value >= 1.0).count();
        label(
            &p,
            pos2(outer.right(), header.y + 70.0),
            Align2::RIGHT_CENTER,
            &format!("{finished} из {} задач готовы", TASKS.len()),
            QUIET,
        );
        self.graph(ui, graph, now);
        self.inspector(ui, inspector, now);
        let footer_y = outer.bottom() - 33.0;
        p.hline(outer.x_range(), footer_y - 10.0, Stroke::new(1.0, LINE));
        for (offset, key, text, modal) in [
            (0.0, "Ctrl+K", "Действия", Modal::Commands),
            (154.0, "Ctrl+F", "Найти задачу", Modal::Search),
            (324.0, "Ctrl+,", "Настройки", Modal::Settings),
        ] {
            let rect =
                Rect::from_min_size(pos2(outer.left() + offset, footer_y), vec2(144.0, 29.0));
            let response = ui
                .interact(rect, ui.make_persistent_id(key), egui::Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::Button, true, format!("{text} · {key}"))
            });
            let hot = response.hovered() || response.has_focus();
            let color = if hot { phase.signal } else { phase.rim };
            p.add(Shape::line(
                vec![
                    rect.left_bottom() - vec2(0.0, 6.0),
                    rect.left_bottom() + vec2(6.0, 0.0),
                    rect.left_bottom() + vec2(51.0, 0.0),
                ],
                Stroke::new(1.0, color),
            ));
            p.text(
                rect.left_center() + vec2(2.0, 0.0),
                Align2::LEFT_CENTER,
                key,
                FontId::monospace(12.0),
                color,
            );
            label(
                &p,
                rect.left_center() + vec2(59.0, 0.0),
                Align2::LEFT_CENTER,
                text,
                if hot { INK } else { QUIET },
            );
            if response.clicked() {
                self.open(ui.ctx(), modal);
            }
        }
        label(
            &p,
            pos2(outer.right(), footer_y + 14.0),
            Align2::RIGHT_CENTER,
            "← ↑ ↓ →  Выбрать узел",
            QUIET,
        );
        if let Some(modal) = self.modal {
            self.show_modal(ui.ctx(), modal, now);
        }
    }

    fn graph(&mut self, ui: &mut egui::Ui, rect: Rect, now: f64) {
        let phase = &PHASES[self.phase];
        let p = ui.painter_at(rect);
        panel(&p, rect, phase);
        let backdrop = p.with_clip_rect(rect.shrink(10.0));
        let center = rect.center() + vec2(0.0, 26.0);
        if self.effects {
            aura(
                &backdrop,
                center,
                rect.size() * 0.66,
                phase.base.gamma_multiply(0.50),
            );
            aura(
                &backdrop,
                pos2(rect.right() - 82.0, rect.bottom() - 81.0),
                rect.size() * vec2(0.48, 0.57),
                phase.side.gamma_multiply(0.28),
            );
        }
        let span = rect.height();
        let crest = center + vec2(span * 0.13, -span * 0.35);
        let face = diamond(crest, vec2(span * 0.55, span * 0.48));
        let face_bounds = Rect::from_center_size(crest, vec2(span * 1.10, span * 0.96));
        gradient(
            &backdrop,
            &face,
            face_bounds,
            phase.side.gamma_multiply(0.28),
            phase.side.gamma_multiply(0.025),
        );
        for step in -16..32 {
            let x = rect.left() + step as f32 * 29.0;
            let stroke = Stroke::new(
                1.0,
                phase
                    .rim
                    .gamma_multiply(if step % 4 == 0 { 0.15 } else { 0.07 }),
            );
            backdrop.line_segment(
                [
                    pos2(x, rect.bottom()),
                    pos2(x + rect.height() * 0.6, rect.top()),
                ],
                stroke,
            );
            backdrop.line_segment(
                [
                    pos2(x, rect.top()),
                    pos2(x + rect.height() * 0.6, rect.bottom()),
                ],
                stroke,
            );
        }
        for scale in [0.34, 0.40, 0.48] {
            let radius = rect.height() * scale;
            backdrop.add(Shape::closed_line(
                diamond(center, vec2(radius * 1.7, radius)),
                Stroke::new(1.0, phase.rim.gamma_multiply(0.16)),
            ));
        }
        p.rect_filled(
            Rect::from_min_size(rect.min + vec2(20.0, 15.0), vec2(129.0, 25.0)),
            0,
            DARK,
        );
        label(
            &p,
            rect.min + vec2(28.0, 27.0),
            Align2::LEFT_CENTER,
            "Карта задач",
            INK,
        );
        let field = rect.shrink2(vec2(6.0, 24.0)).translate(vec2(0.0, 12.0));
        let nodes = TASKS.map(|task| {
            Rect::from_center_size(
                pos2(
                    field.left() + task.pos[0] * field.width(),
                    field.top() + task.pos[1] * field.height(),
                ),
                vec2((field.width() * 0.195).clamp(96.0, 164.0), 66.0),
            )
        });

        for (index, task) in TASKS.iter().enumerate() {
            for &need in task.needs {
                let from = nodes[need];
                let to = nodes[index];
                let points = if (from.center().x - to.center().x).abs() < 1.0 {
                    vec![from.center_bottom(), to.center_top()]
                } else {
                    let a = from.right_center();
                    let b = to.left_center();
                    let middle = (a.x + b.x) * 0.5;
                    vec![a, pos2(middle, a.y), pos2(middle, b.y), b]
                };
                let live = self.status(need) == Status::Done;
                let traveling = self.completions[need].and_then(|event| event.travel(now));
                let color = phase.rim.gamma_multiply(if live && traveling.is_none() {
                    0.77
                } else {
                    0.29
                });
                p.add(Shape::line(points.clone(), Stroke::new(5.0, DARK)));
                p.add(Shape::line(points.clone(), Stroke::new(1.5, color)));
                let end = *points.last().unwrap();
                p.add(Shape::convex_polygon(
                    diamond(end, vec2(4.0, 4.0)),
                    color,
                    Stroke::NONE,
                ));
                if let Some(travel) = traveling {
                    paint_signal(&p, &points, travel, phase.signal);
                } else if self.effects
                    && live
                    && index == self.selected
                    && self.worker.is_some_and(|(active, _, _)| active == index)
                {
                    let step =
                        (now as f32 * 2.2 + need as f32 * 0.24).fract() * (points.len() - 1) as f32;
                    let segment = step as usize;
                    let dot = points[segment].lerp(points[segment + 1], step.fract());
                    p.circle_filled(dot, 3.0, phase.signal);
                }
            }
        }
        for (index, node) in nodes.into_iter().enumerate() {
            let status = self.status(index);
            let response = ui
                .interact(node, task_id(index), egui::Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            response.widget_info(|| {
                WidgetInfo::selected(
                    WidgetType::SelectableLabel,
                    true,
                    self.selected == index,
                    TASKS[index].name,
                )
            });
            if response.clicked() || response.gained_focus() {
                self.selected = index;
            }
            let selected = self.selected == index;
            let arrival = TASKS[index]
                .needs
                .iter()
                .filter_map(|&need| self.completions[need])
                .map(|event| event.arrival(index, now))
                .fold(0.0_f32, f32::max);
            let edge = match status {
                Status::Blocked => phase.rim.lerp_to_gamma(LINE, 0.80),
                Status::Running | Status::Paused if selected => phase.signal,
                _ => phase.rim,
            }
            .lerp_to_gamma(phase.signal, arrival);
            if selected && self.effects {
                aura(
                    &p,
                    node.center(),
                    node.size() * vec2(0.83, 1.35),
                    edge.gamma_multiply(0.28),
                );
            }
            if arrival > 0.0 {
                aura(
                    &p,
                    node.center(),
                    node.size() * vec2(1.03, 1.52),
                    phase.signal.gamma_multiply(arrival * 0.66),
                );
                let expansion = (1.0 - arrival) * 10.0;
                p.add(Shape::closed_line(
                    chamfer(node.expand(3.0 + expansion), 13.0),
                    Stroke::new(1.4, phase.signal.gamma_multiply(arrival * 0.84)),
                ));
            }
            plate(
                &p,
                node,
                phase,
                edge,
                selected || response.hovered() || arrival > 0.4,
                status == Status::Blocked,
            );
            if arrival > 0.0 {
                gradient(
                    &p,
                    &chamfer(node.shrink(4.0), 8.0),
                    node.shrink(4.0),
                    phase.signal.gamma_multiply(arrival * 0.26),
                    Color32::TRANSPARENT,
                );
            }
            p.add(Shape::convex_polygon(
                diamond(node.min + vec2(15.0, 16.0), vec2(4.0, 4.0)),
                edge,
                Stroke::NONE,
            ));
            label(
                &p,
                node.min + vec2(25.0, 16.0),
                Align2::LEFT_CENTER,
                status.label(),
                if selected {
                    edge
                } else {
                    QUIET.lerp_to_gamma(phase.signal, arrival)
                },
            );
            p.text(
                node.min + vec2(13.0, 41.0),
                Align2::LEFT_CENTER,
                TASKS[index].name,
                FontId::proportional(17.0),
                INK,
            );
        }
    }

    fn inspector(&mut self, ui: &mut egui::Ui, rect: Rect, now: f64) {
        let phase = &PHASES[self.phase];
        let p = ui.painter_at(rect);
        panel(&p, rect, phase);
        let index = self.selected;
        let task = &TASKS[index];
        let status = self.status(index);
        let left = rect.left() + 21.0;
        label(
            &p,
            pos2(left, rect.top() + 26.0),
            Align2::LEFT_CENTER,
            "Выбранная задача",
            QUIET,
        );
        label(
            &p,
            pos2(rect.right() - 21.0, rect.top() + 26.0),
            Align2::RIGHT_CENTER,
            &format!("{:02} / {:02}", index + 1, TASKS.len()),
            phase.rim,
        );
        let center = pos2(rect.center().x, rect.top() + rect.height() * 0.31);
        let radius = (rect.height() * 0.161).clamp(52.0, 94.0);
        gauge(
            &p,
            center,
            radius,
            self.progress[index],
            phase,
            self.effects,
        );

        let detail_y = (center.y + radius + 37.0).min(rect.bottom() - 134.0);
        p.text(
            pos2(left, detail_y),
            Align2::LEFT_CENTER,
            task.name,
            FontId::proportional(26.0),
            INK,
        );
        let detail = if status == Status::Blocked {
            let names = task
                .needs
                .iter()
                .filter(|&&need| self.progress[need] < 1.0)
                .map(|&need| TASKS[need].name)
                .collect::<Vec<_>>();
            format!("Ожидает: {}", names.join(", "))
        } else {
            task.detail.to_owned()
        };
        let galley = p.layout(
            detail,
            FontId::proportional(15.0),
            QUIET,
            rect.width() - 42.0,
        );
        p.galley(pos2(left, detail_y + 21.0), galley, QUIET);
        let button = Rect::from_min_max(
            pos2(left, rect.bottom() - 60.0),
            rect.right_bottom() - vec2(21.0, 19.0),
        );
        let run = self.run_label();
        let text = run.unwrap_or(match status {
            Status::Done => "Завершено",
            Status::Blocked => "Ждёт входных данных",
            _ => "Очередь занята",
        });
        if control(
            ui,
            button,
            "run-nolox",
            text,
            run.is_some(),
            run.is_some(),
            phase.signal,
        ) {
            self.execute(ui.ctx(), Command::Run, now);
        }
    }

    fn show_modal(&mut self, ctx: &egui::Context, modal: Modal, now: f64) {
        let phase = &PHASES[self.phase];
        let frame = egui::Frame::new()
            .fill(Color32::from_rgb(13, 20, 39))
            .inner_margin(20)
            .stroke(Stroke::new(1.5, phase.rim));
        let response = egui::Modal::new(Id::new("nolox-modal"))
            .frame(frame)
            .show(ctx, |ui| {
                ui.set_width(if modal == Modal::Settings {
                    420.0
                } else {
                    482.0
                });
                ui.spacing_mut().item_spacing.y = 7.0;
                ui.label(
                    RichText::new(match modal {
                        Modal::Commands => "Действия",
                        Modal::Search => "Найти задачу",
                        Modal::Settings => "Настройки",
                    })
                    .size(25.0)
                    .color(INK),
                );
                if modal == Modal::Settings {
                    ui.label("Цветовая фаза");
                    ui.horizontal(|ui| {
                        for (index, phase) in PHASES.iter().enumerate() {
                            let rect = ui.allocate_space(vec2(127.0, 33.0)).1;
                            if control(
                                ui,
                                rect,
                                ("settings-phase", index),
                                phase.name,
                                index == self.phase,
                                true,
                                phase.rim,
                            ) {
                                self.phase = index;
                            }
                        }
                    });
                    ui.add_space(8.0);
                    if ui
                        .checkbox(&mut self.effects, "Свечение и движение сигнала")
                        .changed()
                        && !self.effects
                    {
                        self.completions.fill(None);
                    }
                    let rect = ui.allocate_space(vec2(110.0, 30.0)).1;
                    if control(
                        ui,
                        rect,
                        "close-settings",
                        "Закрыть",
                        false,
                        true,
                        phase.rim,
                    ) {
                        self.close(ctx);
                    }
                    return None;
                }

                let up = consume_key(ctx, Key::ArrowUp, false);
                let down = consume_key(ctx, Key::ArrowDown, false);
                if up + down > 0 {
                    ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
                }
                let enter = consume_key(ctx, Key::Enter, false) > 0;
                let search_id = Id::new("nolox-search");
                if self.focus_search && !ui.is_sizing_pass() {
                    ui.memory_mut(|memory| memory.request_focus(search_id));
                    self.focus_search = false;
                }
                let search = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .id(search_id)
                        .desired_width(f32::INFINITY)
                        .hint_text(if modal == Modal::Commands {
                            "Введите название команды…"
                        } else {
                            "Введите название узла…"
                        }),
                );
                if search.changed() {
                    self.cursor = 0;
                }
                let mut choices = Vec::new();
                if modal == Modal::Commands {
                    if let Some(label) = self.run_label() {
                        choices.push((label.to_owned(), Command::Run));
                    }
                    choices.extend(
                        PHASES.iter().enumerate().map(|(i, phase)| {
                            (format!("Фаза · {}", phase.name), Command::Phase(i))
                        }),
                    );
                    choices.push(("Открыть настройки".to_owned(), Command::Settings));
                    choices.push(("Сбросить демонстрацию".to_owned(), Command::Reset));
                } else {
                    choices.extend(
                        TASKS
                            .iter()
                            .enumerate()
                            .map(|(i, task)| (task.name.to_owned(), Command::Select(i))),
                    );
                }
                let query = normalize(&self.query);
                choices.retain(|(label, _)| normalize(label).contains(query.trim()));
                if choices.is_empty() {
                    ui.label(RichText::new("Ничего не найдено").color(QUIET));
                    return None;
                }
                self.cursor = (self.cursor as isize + down as isize - up as isize)
                    .rem_euclid(choices.len() as isize) as usize;
                let mut selection = None;
                for (index, (name, command)) in choices.into_iter().enumerate() {
                    let rect = ui.allocate_space(vec2(ui.available_width(), 38.0)).1;
                    if command_row(ui, rect, index, &name, self.cursor == index, phase)
                        || (enter && self.cursor == index)
                    {
                        selection = Some(command);
                    }
                }
                ui.label(
                    RichText::new("↑ ↓  Выбрать     Enter  Выполнить     Esc  Закрыть")
                        .color(QUIET)
                        .size(12.0),
                );
                selection
            });
        if let Some(command) = response.inner {
            self.execute(ctx, command, now);
        } else if response.should_close() && self.modal.is_some() {
            self.close(ctx);
        }
    }
}

fn task_id(index: usize) -> Id {
    Id::new(("nolox-task", index))
}

fn label(p: &Painter, where_: Pos2, align: Align2, text: &str, color: Color32) {
    p.text(where_, align, text, FontId::proportional(13.0), color);
}

fn chamfer(rect: Rect, cut: f32) -> Vec<Pos2> {
    vec![
        rect.left_top(),
        pos2(rect.right() - cut, rect.top()),
        pos2(rect.right(), rect.top() + cut),
        rect.right_bottom(),
        pos2(rect.left() + cut, rect.bottom()),
        pos2(rect.left(), rect.bottom() - cut),
    ]
}

fn diamond(center: Pos2, radius: Vec2) -> Vec<Pos2> {
    vec![
        center - vec2(0.0, radius.y),
        center + vec2(radius.x, 0.0),
        center + vec2(0.0, radius.y),
        center - vec2(radius.x, 0.0),
    ]
}

fn gradient(p: &Painter, polygon: &[Pos2], rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = Mesh::default();
    for &point in polygon {
        mesh.colored_vertex(
            point,
            top.lerp_to_gamma(bottom, (point.y - rect.top()) / rect.height()),
        );
    }
    for i in 1..polygon.len() as u32 - 1 {
        mesh.add_triangle(0, i, i + 1);
    }
    p.add(Shape::mesh(mesh));
}

fn aura(p: &Painter, center: Pos2, radius: Vec2, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for i in 0..32 {
        let angle = i as f32 * std::f32::consts::TAU / 32.0;
        mesh.colored_vertex(
            center + vec2(angle.cos(), angle.sin()) * radius,
            Color32::TRANSPARENT,
        );
        mesh.add_triangle(0, i + 1, (i + 1) % 32 + 1);
    }
    p.add(Shape::mesh(mesh));
}

fn paint_signal(p: &Painter, points: &[Pos2], travel: f32, signal: Color32) {
    let length: f32 = points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum();
    let head = length * travel;
    let mut tip = points[0];
    let mut start = 0.0;
    for pair in points.windows(2) {
        let span = pair[0].distance(pair[1]);
        if span == 0.0 {
            continue;
        }
        let end = start + span;
        if start <= head && head <= end {
            tip = pair[0].lerp(pair[1], ((head - start) / span).clamp(0.0, 1.0));
        }
        for (tail, width, brightness) in [(72.0, 10.0, 0.16), (52.0, 5.0, 0.45), (29.0, 2.5, 0.94)]
        {
            let first = (head - tail).max(start);
            let last = head.min(end);
            if first < last {
                p.line_segment(
                    [
                        pair[0].lerp(pair[1], (first - start) / span),
                        pair[0].lerp(pair[1], (last - start) / span),
                    ],
                    Stroke::new(width, signal.gamma_multiply(brightness)),
                );
            }
        }
        start = end;
    }
    aura(p, tip, Vec2::splat(19.0), signal.gamma_multiply(0.62));
    p.add(Shape::convex_polygon(
        diamond(tip, vec2(5.5, 5.5)),
        signal.lerp_to_gamma(INK, 0.55),
        Stroke::new(1.0, signal),
    ));
}

fn panel(p: &Painter, rect: Rect, phase: &Phase) {
    let points = chamfer(rect.shrink(1.0), 15.0);
    gradient(p, &points, rect, phase.base.lerp_to_gamma(DARK, 0.75), DARK);
    p.add(Shape::closed_line(
        points,
        Stroke::new(1.0, phase.rim.lerp_to_gamma(LINE, 0.72)),
    ));
}

fn plate(p: &Painter, rect: Rect, phase: &Phase, rim: Color32, raised: bool, blocked: bool) {
    let front = chamfer(rect, 10.0);
    let depth = vec2(7.0, 7.0);
    for (a, b) in [(2, 3), (3, 4), (4, 5)] {
        p.add(Shape::convex_polygon(
            vec![front[a], front[b], front[b] + depth, front[a] + depth],
            phase.side.lerp_to_gamma(
                DARK,
                if blocked {
                    0.88
                } else if raised {
                    0.22
                } else {
                    0.47
                },
            ),
            Stroke::new(1.0, rim.gamma_multiply(0.54)),
        ));
    }
    let top = phase.base.lerp_to_gamma(
        DARK,
        if blocked {
            0.86
        } else if raised {
            0.10
        } else {
            0.41
        },
    );
    gradient(p, &front, rect, top, DARK);
    p.add(Shape::closed_line(
        front,
        Stroke::new(if raised { 2.0 } else { 1.0 }, rim),
    ));
    let inset = rect.shrink(4.0);
    p.add(Shape::closed_line(
        chamfer(inset, 8.0),
        Stroke::new(1.0, rim.gamma_multiply(0.27)),
    ));
    p.hline(
        rect.left() + 12.0..=rect.right() - 19.0,
        rect.bottom() - 5.0,
        Stroke::new(1.0, rim.gamma_multiply(0.35)),
    );
    if raised {
        p.line_segment(
            [
                rect.left_top() + vec2(8.0, 1.0),
                rect.left_top() + vec2(38.0, 1.0),
            ],
            Stroke::new(2.5, INK),
        );
    }
}

fn command_row(
    ui: &mut egui::Ui,
    rect: Rect,
    index: usize,
    name: &str,
    active: bool,
    phase: &Phase,
) -> bool {
    let response = ui
        .interact(
            rect,
            ui.make_persistent_id(("command", index)),
            egui::Sense::click(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    response.widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, true, active, name));
    let hot = active || response.hovered() || response.has_focus();
    let color = if hot {
        phase.signal
    } else {
        phase.rim.gamma_multiply(0.5)
    };
    let p = ui.painter();
    if hot {
        gradient(
            p,
            &chamfer(rect, 8.0),
            rect,
            phase.base.gamma_multiply(0.54),
            DARK,
        );
    }
    p.add(Shape::convex_polygon(
        diamond(rect.left_center() + vec2(14.0, 0.0), Vec2::splat(5.0)),
        if hot { color } else { DARK },
        Stroke::new(1.0, color),
    ));
    p.hline(
        rect.left() + 27.0..=rect.right(),
        rect.bottom() - 1.0,
        Stroke::new(1.0, color.gamma_multiply(0.5)),
    );
    p.text(
        rect.left_center() + vec2(28.0, 0.0),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(16.0),
        INK,
    );
    response.clicked()
}

fn control(
    ui: &mut egui::Ui,
    rect: Rect,
    id: impl egui::AsIdSalt,
    text: &str,
    active: bool,
    enabled: bool,
    rim: Color32,
) -> bool {
    let response = ui.interact(
        rect,
        ui.make_persistent_id(id),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, text));
    let hot = enabled && (response.hovered() || response.has_focus());
    let p = ui.painter();
    let bright = active || hot;
    let color = if bright { rim } else { QUIET };
    let body = if active {
        rim.gamma_multiply(0.14)
    } else {
        DARK
    };
    p.add(Shape::convex_polygon(
        chamfer(rect.shrink(1.0), 7.0),
        body,
        Stroke::new(
            if bright { 1.35 } else { 1.0 },
            if bright { rim } else { LINE },
        ),
    ));
    if active {
        p.line_segment(
            [
                rect.left_top() + vec2(3.0, 6.0),
                rect.left_bottom() - vec2(-3.0, 10.0),
            ],
            Stroke::new(2.5, rim),
        );
    }
    label(p, rect.center(), Align2::CENTER_CENTER, text, color);
    enabled
        && response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
}

fn gauge(p: &Painter, center: Pos2, radius: f32, progress: f32, phase: &Phase, effects: bool) {
    if effects {
        aura(
            p,
            center,
            Vec2::splat(radius * 1.62),
            phase.rim.gamma_multiply(0.29),
        );
    }
    p.circle(
        center,
        radius * 0.74,
        DARK,
        Stroke::new(1.0, phase.rim.gamma_multiply(0.48)),
    );
    p.add(Shape::closed_line(
        diamond(center, Vec2::splat(radius * 1.21)),
        Stroke::new(1.0, phase.rim.gamma_multiply(0.35)),
    ));
    for i in 0..42 {
        let theta = 2.38 + i as f32 / 41.0 * 4.66;
        let dir = vec2(theta.cos(), theta.sin());
        let mat = if (i + 1) as f32 / 42.0 <= progress {
            phase.signal
        } else {
            phase.rim.gamma_multiply(0.29)
        };
        p.line_segment(
            [center + dir * radius * 0.83, center + dir * radius],
            Stroke::new(if i % 7 == 0 { 3.0 } else { 2.0 }, mat),
        );
    }
    p.text(
        center - vec2(0.0, 4.0),
        Align2::CENTER_CENTER,
        format!("{:.0}%", progress * 100.0),
        FontId::proportional(radius * 0.5),
        INK,
    );
    label(
        p,
        center + vec2(0.0, radius * 0.38),
        Align2::CENTER_CENTER,
        "готовность",
        QUIET,
    );
}

impl eframe::App for Nolox {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

pub fn run_nolox() -> eframe::Result {
    eframe::run_native(
        "Nolox · Мира",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1216.0, 730.0])
                .with_min_inner_size([880.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(Nolox::new(&cc.egui_ctx)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_sends_a_signal_but_only_the_final_dependency_lights_the_node() {
        let ctx = egui::Context::default();
        let mut app = Nolox::new(&ctx);

        app.execute(&ctx, Command::Run, 0.0);
        app.tick(2.0);
        assert!(app.completions[2].is_none());
        app.tick(3.0);
        let geometry = app.completions[2].unwrap();
        assert!(geometry.travel(3.0 + SIGNAL_TRAVEL / 2.0).is_some());
        assert_eq!(geometry.arrival(4, 3.0 + SIGNAL_TRAVEL + 0.1), 0.0);
        assert!(app.status(4) == Status::Blocked);

        app.selected = 3;
        app.execute(&ctx, Command::Run, 4.0);
        app.tick(10.0);
        let palette = app.completions[3].unwrap();
        assert!(app.status(4) == Status::Ready);
        assert!(app.completions[2].is_none());
        assert_eq!(palette.arrival(4, 10.0 + SIGNAL_TRAVEL / 2.0), 0.0);
        assert!(palette.travel(10.0 + SIGNAL_TRAVEL + 0.01).is_none());
        assert!(palette.arrival(4, 10.0 + SIGNAL_TRAVEL + 0.01) > 0.9);
        assert!(palette.arrival(4, 10.0 + SIGNAL_TRAVEL + UNLOCK_GLOW / 2.0) < 0.26);
        assert_eq!(palette.arrival(5, 10.0 + SIGNAL_TRAVEL + 0.01), 0.0);
        app.tick(10.0 + SIGNAL_TRAVEL + UNLOCK_GLOW + 0.01);
        assert!(app.completions.iter().all(Option::is_none));

        app.effects = false;
        app.selected = 4;
        app.execute(&ctx, Command::Run, 12.0);
        app.tick(17.0);
        assert!(app.status(5) == Status::Ready);
        assert!(app.completions[4].is_none());

        app.effects = true;
        app.execute(&ctx, Command::Reset, 18.0);
        app.execute(&ctx, Command::Run, 18.0);
        app.tick(21.0);
        assert!(app.completions[2].is_some());
        app.execute(&ctx, Command::Reset, 21.1);
        assert!(app.completions.iter().all(Option::is_none));
    }

    #[test]
    fn keyboard_commands_preserve_selection_and_unlock_the_dependent_task() {
        let ctx = egui::Context::default();
        let mut app = Nolox::new(&ctx);
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        };
        let ctrl = egui::Modifiers::CTRL;
        let plain = egui::Modifiers::NONE;
        let frame = |app: &mut Nolox, time, events: Vec<egui::Event>| {
            let events = events
                .into_iter()
                .flat_map(|event| {
                    let mut released = event.clone();
                    if let egui::Event::Key { pressed, .. } = &mut released {
                        *pressed = false;
                        [Some(event), Some(released)]
                    } else {
                        [Some(event), None]
                    }
                })
                .flatten()
                .collect();
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(880.0, 600.0))),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ui| app.show(ui),
            )
            .textures_delta
            .clear();
        };
        frame(&mut app, 0.0, vec![]);
        frame(&mut app, 0.1, vec![key(Key::K, ctrl)]);
        assert!(app.modal == Some(Modal::Commands));
        frame(&mut app, 0.2, vec![key(Key::ArrowUp, plain)]);
        assert_eq!(app.cursor, 5);
        frame(&mut app, 0.3, vec![egui::Event::Text("призма".to_owned())]);
        assert_eq!(app.query, "призма");
        frame(&mut app, 0.4, vec![key(Key::Enter, plain)]);
        assert_eq!(app.phase, 1);
        assert_eq!(app.selected, 2);
        assert!(app.modal.is_none());

        frame(&mut app, 0.5, vec![key(Key::Enter, plain)]);
        assert!(app.status(2) == Status::Running);
        frame(&mut app, 0.9, vec![key(Key::Enter, plain)]);
        assert!(app.status(2) == Status::Paused);
        let paused = app.progress[2];
        frame(&mut app, 1.1, vec![]);
        assert_eq!(app.progress[2], paused);
        frame(&mut app, 1.2, vec![key(Key::Enter, plain)]);
        frame(&mut app, 4.3, vec![]);
        assert!(app.status(2) == Status::Done);
        assert!(app.status(4) == Status::Blocked);

        frame(&mut app, 4.4, vec![key(Key::ArrowDown, plain)]);
        assert_eq!(app.selected, 3);
        frame(&mut app, 4.5, vec![key(Key::Enter, plain)]);
        frame(&mut app, 10.6, vec![]);
        assert!(app.status(4) == Status::Ready);
        frame(&mut app, 10.7, vec![key(Key::F, ctrl)]);
        assert!(app.modal == Some(Modal::Search));
        frame(&mut app, 10.8, vec![egui::Event::Text("сборка".to_owned())]);
        frame(&mut app, 10.9, vec![key(Key::Enter, plain)]);
        assert_eq!(app.selected, 4);
        frame(&mut app, 11.0, vec![key(Key::Comma, ctrl)]);
        assert!(app.modal == Some(Modal::Settings));
        frame(&mut app, 11.1, vec![key(Key::Escape, plain)]);
        assert!(app.modal.is_none());
        assert_eq!(app.selected, 4);
    }
}
