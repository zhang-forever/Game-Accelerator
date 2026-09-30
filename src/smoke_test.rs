use crate::app::Page;
use crate::monitor::SystemStats;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const PAGES: [(Page, &str); 6] = [
    (Page::Dashboard, "dashboard"),
    (Page::Settings, "settings"),
    (Page::Gpu, "gpu"),
    (Page::SystemOpt, "system"),
    (Page::Process, "processes"),
    (Page::Process, "process-list"),
];

/// Capture the live renderer while the app filters all interactive input.
pub struct SmokeTest {
    directory: PathBuf,
    started: Instant,
    page_index: usize,
    requested: bool,
    pub completed: bool,
}

#[derive(Serialize)]
struct GuiReport<'a> {
    version: &'static str,
    read_only: bool,
    rendered_pages: Vec<&'static str>,
    stats: &'a SystemStats,
}

impl SmokeTest {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            started: Instant::now(),
            page_index: 0,
            requested: false,
            completed: false,
        }
    }

    pub fn poll(
        &mut self,
        ctx: &egui::Context,
        stats: &SystemStats,
    ) -> Result<Option<Page>, String> {
        if self.completed {
            return Ok(None);
        }
        if self.started.elapsed() > Duration::from_secs(45) {
            return Err("GUI smoke test timed out before all pages were rendered".to_string());
        }
        let screenshot = ctx.input(|input| {
            input.events.iter().find_map(|event| {
                if let egui::Event::Screenshot { image, .. } = event {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });
        if let Some(image) = screenshot.filter(|_| self.requested) {
            std::fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
            let pixels: Vec<u8> = image
                .pixels
                .iter()
                .flat_map(|pixel| pixel.to_array())
                .collect();
            image::save_buffer(
                self.directory
                    .join(format!("{}.png", PAGES[self.page_index].1)),
                &pixels,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
            self.page_index += 1;
            self.requested = false;
            if self.page_index == PAGES.len() {
                crate::cli::write_report(
                    &self.directory.join("report.toml"),
                    &GuiReport {
                        version: env!("CARGO_PKG_VERSION"),
                        read_only: true,
                        rendered_pages: PAGES.iter().map(|(_, name)| *name).collect(),
                        stats,
                    },
                )?;
                self.completed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return Ok(None);
            }
            return Ok(Some(PAGES[self.page_index].0));
        }
        Ok(None)
    }

    pub fn process_list_view(&self) -> bool {
        self.page_index == 5
    }

    pub fn request_capture(&mut self, ctx: &egui::Context, stats: &SystemStats) {
        if !self.completed
            && !self.requested
            && stats.flags_ready
            && stats.cpu_threads > 0
            && stats.ram_total_gb > 0.0
        {
            self.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        }
    }
}
