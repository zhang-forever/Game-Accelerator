use egui::{Color32, Pos2, Rect, Stroke};

#[derive(Clone, Copy)]
pub enum Icon {
    Overview,
    Processes,
    Gpu,
    Sliders,
    Settings,
    Shield,
    Memory,
}

pub fn paint(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let point = |x: f32, y: f32| {
        Pos2::new(
            rect.left() + x * rect.width() / 20.0,
            rect.top() + y * rect.height() / 20.0,
        )
    };
    let stroke = Stroke::new(1.5, color);
    let line = |points: &[(f32, f32)]| {
        painter.add(egui::Shape::line(
            points.iter().map(|(x, y)| point(*x, *y)).collect(),
            stroke,
        ))
    };
    match icon {
        Icon::Overview => {
            for (x, y) in [(3.0, 3.0), (11.0, 3.0), (3.0, 11.0), (11.0, 11.0)] {
                painter.rect_stroke(
                    Rect::from_min_max(point(x, y), point(x + 6.0, y + 6.0)),
                    egui::Rounding::same(1.5),
                    stroke,
                );
            }
        }
        Icon::Processes => {
            for y in [5.0, 10.0, 15.0] {
                painter.circle_filled(point(3.0, y), 1.2, color);
                line(&[(7.0, y), (17.0, y)]);
            }
        }
        Icon::Gpu => {
            painter.rect_stroke(
                Rect::from_min_max(point(2.0, 4.0), point(17.0, 15.0)),
                egui::Rounding::same(2.0),
                stroke,
            );
            painter.circle_stroke(point(8.0, 9.5), 3.0, stroke);
            line(&[(18.0, 2.0), (18.0, 17.0)]);
            for x in [4.0, 7.0, 10.0, 13.0] {
                line(&[(x, 15.0), (x, 17.0)]);
            }
        }
        Icon::Sliders => {
            for (x, y) in [(4.0, 7.0), (10.0, 13.0), (16.0, 6.0)] {
                line(&[(x, 3.0), (x, 17.0)]);
                painter.circle_filled(point(x, y), 2.4, super::theme::SIDEBAR_BG);
                painter.circle_stroke(point(x, y), 2.4, stroke);
            }
        }
        Icon::Settings => {
            painter.circle_stroke(point(10.0, 10.0), rect.width() * 0.28, stroke);
            painter.circle_stroke(point(10.0, 10.0), rect.width() * 0.10, stroke);
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::FRAC_PI_4;
                line(&[
                    (10.0 + angle.sin() * 6.0, 10.0 + angle.cos() * 6.0),
                    (10.0 + angle.sin() * 8.5, 10.0 + angle.cos() * 8.5),
                ]);
            }
        }
        Icon::Shield => {
            line(&[
                (10.0, 2.0),
                (17.0, 5.0),
                (16.0, 12.0),
                (10.0, 18.0),
                (4.0, 12.0),
                (3.0, 5.0),
                (10.0, 2.0),
            ]);
            line(&[(7.0, 10.0), (9.0, 12.0), (13.0, 8.0)]);
        }
        Icon::Memory => {
            painter.rect_stroke(
                Rect::from_min_max(point(4.0, 4.0), point(16.0, 16.0)),
                egui::Rounding::same(2.0),
                stroke,
            );
            for n in [6.0, 10.0, 14.0] {
                line(&[(n, 1.0), (n, 4.0)]);
                line(&[(n, 16.0), (n, 19.0)]);
                line(&[(1.0, n), (4.0, n)]);
                line(&[(16.0, n), (19.0, n)]);
            }
        }
    }
}

pub fn show(ui: &mut egui::Ui, icon: Icon, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    paint(ui.painter(), rect, icon, color);
}
