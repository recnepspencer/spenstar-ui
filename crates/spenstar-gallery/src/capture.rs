use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, TryRecvError},
    },
    time::{Duration, Instant},
};

use eframe::egui::{
    ColorImage, Context, Event, Key, Modifiers, RawInput, ScreenshotCallback, ViewportCommand,
};
use spenstar_ui::{
    Theme,
    layout::{CollapseMode, Sidebar, SidebarVariant},
    theme::{Accent, Mode},
};

use crate::{Gallery, Page, overlays::OverlayScene};

pub(super) const SIZE: [f32; 2] = [1280., 820.];
const PIXELS: [usize; 2] = [2560, 1640];
const SETTLE: Duration = Duration::from_millis(580);
const TICK: Duration = Duration::from_millis(40);
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy)]
pub(super) struct Scene {
    pub(super) filename: &'static str,
    pub(super) page: Page,
    pub(super) mode: Mode,
    pub(super) inspector: bool,
    pub(super) command: bool,
    pub(super) overlay: OverlayScene,
}

impl Scene {
    pub(super) const fn w4(
        filename: &'static str,
        page: Page,
        mode: Mode,
        overlay: OverlayScene,
    ) -> Self {
        Self {
            overlay,
            ..Self::new(filename, page, mode)
        }
    }

    pub(super) const fn new(filename: &'static str, page: Page, mode: Mode) -> Self {
        Self {
            filename,
            page,
            mode,
            inspector: false,
            command: false,
            overlay: OverlayScene::None,
        }
    }
}

type ResultSlot = Arc<Mutex<Option<Result<(), String>>>>;

enum State {
    Configure,
    Settle {
        since: Instant,
        frames: usize,
    },
    Receive {
        since: Instant,
        result: Receiver<Result<(), String>>,
    },
    Finished,
}

pub(super) struct Capture {
    directory: PathBuf,
    scene: usize,
    scenes: Vec<Scene>,
    export_size: [usize; 2],
    state: State,
    result: ResultSlot,
}

impl Capture {
    pub(super) fn from_args() -> io::Result<Option<Self>> {
        let mut args = std::env::args_os().skip(1);
        let Some(flag) = args.next() else {
            return Ok(None);
        };
        let Some(directory) = args.next().filter(|_| flag == "--capture-dir") else {
            return Err(usage());
        };
        use crate::capture_scenes::{LEGACY_SCENES, WAVE3_SCENES, WAVE4_SCENES};
        let (scenes, export_size) = match args.next() {
            None => (WAVE4_SCENES.to_vec(), [1280, 820]),
            Some(flag) if flag == "--wave3" => (WAVE3_SCENES.to_vec(), PIXELS),
            Some(flag) if flag == "--legacy" => ([LEGACY_SCENES, WAVE3_SCENES].concat(), PIXELS),
            _ => return Err(usage()),
        };
        if args.next().is_some() || directory.is_empty() {
            return Err(usage());
        }
        let directory = PathBuf::from(directory);
        fs::create_dir_all(&directory)?;
        Ok(Some(Self {
            directory,
            scene: 0,
            scenes,
            export_size,
            state: State::Configure,
            result: Arc::default(),
        }))
    }

    pub(super) fn result(&self) -> ResultSlot {
        self.result.clone()
    }

    pub(super) fn open_select(&self) -> bool {
        self.scenes
            .get(self.scene)
            .is_some_and(|scene| scene.page == Page::Pickers)
    }

    pub(super) fn input(&self, input: &mut RawInput) {
        input.events.clear();
        if self
            .scenes
            .get(self.scene)
            .is_some_and(|scene| scene.overlay == OverlayScene::Dropdown)
            && let State::Settle { frames, .. } = self.state
        {
            let event = match frames {
                3 => Some((Key::ArrowDown, true)),
                4 => Some((Key::ArrowDown, false)),
                5 => Some((Key::ArrowRight, true)),
                6 => Some((Key::ArrowRight, false)),
                _ => None,
            };
            if let Some((key, pressed)) = event {
                input.events.push(Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                });
            }
        }
    }

    pub(super) fn prepare(&mut self, gallery: &mut Gallery, ctx: &Context) {
        if !matches!(self.state, State::Configure) {
            return;
        }
        let scene = &self.scenes[self.scene];
        if self.scene == 0 && (ctx.pixels_per_point() - 2.).abs() > f32::EPSILON {
            ctx.set_pixels_per_point(2.);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(SIZE.into()));
        }
        gallery.page = scene.page;
        if scene.command {
            gallery.command.open();
        } else {
            gallery.command.close();
        }
        gallery.inspector_open = scene.inspector;
        gallery.shell.sidebar = if scene.inspector {
            Sidebar {
                variant: SidebarVariant::Floating,
                mode: CollapseMode::IconRail,
            }
        } else if scene.page == Page::Inbox {
            Sidebar {
                mode: CollapseMode::IconRail,
                ..Sidebar::default()
            }
        } else {
            Sidebar::default()
        };
        gallery.theme = Theme::new(scene.mode, Accent::Blue);
        gallery.theme.apply(ctx);
        spenstar_ui::feedback::ToastQueue::clear_context(ctx);
        gallery.overlays.capture(ctx, scene.overlay);
        gallery
            .workbench
            .capture_drawer(scene.overlay == OverlayScene::WorkbenchDrawer);
        self.state = State::Settle {
            since: Instant::now(),
            frames: 0,
        };
    }

    pub(super) fn after_frame(&mut self, ctx: &Context) {
        match &mut self.state {
            State::Settle { since, frames } => {
                *frames += 1;
                if *frames >= 12 && since.elapsed() >= SETTLE {
                    let (sender, result) = mpsc::channel();
                    let filename = self.scenes[self.scene].filename;
                    let path = self.directory.join(filename);
                    let export_size = self.export_size;
                    let ctx = ctx.clone();
                    let repaint = ctx.clone();
                    ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(
                        move |image| {
                            let outcome = write_image(&path, &image, export_size)
                                .map_err(|err| format!("{filename}: {err}"));
                            let _ = sender.send(outcome);
                            repaint.request_repaint();
                        },
                    )));
                    self.state = State::Receive {
                        since: Instant::now(),
                        result,
                    };
                }
            }
            State::Receive { since, result } => match result.try_recv() {
                Ok(Ok(())) => {
                    println!("Captured {}", self.scenes[self.scene].filename);
                    self.scene += 1;
                    if self.scene == self.scenes.len() {
                        self.finish(ctx, Ok(()));
                    } else {
                        self.state = State::Configure;
                    }
                }
                Ok(Err(error)) => self.finish(ctx, Err(error)),
                Err(TryRecvError::Disconnected) => {
                    self.finish(ctx, Err("screenshot callback disconnected".into()))
                }
                Err(TryRecvError::Empty) if since.elapsed() >= TIMEOUT => {
                    self.finish(
                        ctx,
                        Err(format!(
                            "timed out capturing {}",
                            self.scenes[self.scene].filename
                        )),
                    );
                }
                Err(TryRecvError::Empty) => {}
            },
            State::Configure | State::Finished => {}
        }
        if !matches!(self.state, State::Finished) {
            ctx.request_repaint_after(TICK);
        }
    }

    fn finish(&mut self, ctx: &Context, result: Result<(), String>) {
        if let Ok(mut slot) = self.result.lock() {
            *slot = Some(result);
        }
        self.state = State::Finished;
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn usage() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "usage: spenstar-gallery [--capture-dir <dir> [--wave3|--legacy]]",
    )
}

fn write_image(
    path: &Path,
    image: &ColorImage,
    size: [usize; 2],
) -> Result<(), Box<dyn std::error::Error>> {
    if image.size != PIXELS {
        return Err(format!(
            "expected {} × {} pixels, got {} × {}",
            PIXELS[0], PIXELS[1], image.size[0], image.size[1]
        )
        .into());
    }
    let pending = path.with_extension("png.part");
    let mut encoder = png::Encoder::new(
        BufWriter::new(File::create(&pending)?),
        size[0] as u32,
        size[1] as u32,
    );
    encoder.set_compression(png::Compression::Fast);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let pixels: Vec<u8> = if size == PIXELS {
        image
            .pixels
            .iter()
            .flat_map(|pixel| pixel.to_srgba_unmultiplied())
            .collect()
    } else {
        let mut pixels = Vec::with_capacity(size[0] * size[1] * 4);
        for pair in image.pixels.as_chunks::<{ PIXELS[0] * 2 }>().0 {
            let (top, bottom) = pair.split_at(PIXELS[0]);
            for (a, b) in top.as_chunks::<2>().0.iter().zip(bottom.as_chunks::<2>().0) {
                let samples = [a[0], a[1], b[0], b[1]].map(|pixel| pixel.to_srgba_unmultiplied());
                for channel in 0..4 {
                    let sum: u16 = samples.iter().map(|color| u16::from(color[channel])).sum();
                    pixels.push(((sum + 2) / 4) as u8);
                }
            }
        }
        pixels
    };
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    writer.finish()?;
    fs::rename(pending, path)?;
    Ok(())
}
