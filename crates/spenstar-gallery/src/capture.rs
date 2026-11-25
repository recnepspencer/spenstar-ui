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

use eframe::egui::{ColorImage, Context, ScreenshotCallback, ViewportCommand};
use spenstar_ui::{
    Theme,
    layout::{CollapseMode, Sidebar, SidebarVariant},
    theme::{Accent, Mode},
};

use crate::{Gallery, Page};

pub(super) const SIZE: [f32; 2] = [1280., 820.];
const PIXELS: [usize; 2] = [2560, 1640];
const SETTLE: Duration = Duration::from_millis(350);
const TICK: Duration = Duration::from_millis(40);
const TIMEOUT: Duration = Duration::from_secs(10);

struct Scene {
    filename: &'static str,
    page: Page,
    mode: Mode,
    inspector: bool,
}

impl Scene {
    const fn new(filename: &'static str, page: Page, mode: Mode) -> Self {
        Self {
            filename,
            page,
            mode,
            inspector: false,
        }
    }
}

use Mode::{Dark, Light};

const SCENES: &[Scene] = &[
    Scene::new("01-overview-dark.png", Page::Overview, Dark),
    Scene::new("02-overview-light.png", Page::Overview, Light),
    Scene {
        inspector: true,
        ..Scene::new("03-overview-inspector-dark.png", Page::Overview, Dark)
    },
    Scene::new("04-buttons-dark.png", Page::Buttons, Dark),
    Scene::new("05-buttons-light.png", Page::Buttons, Light),
    Scene::new("06-icons-dark.png", Page::Icons, Dark),
    Scene::new("07-icons-light.png", Page::Icons, Light),
    Scene::new("08-tokens-dark.png", Page::Tokens, Dark),
    Scene::new("09-forms-dark.png", Page::Forms, Dark),
    Scene::new("10-forms-light.png", Page::Forms, Light),
    Scene::new("11-pickers-dark.png", Page::Pickers, Dark),
    Scene::new("12-pickers-light.png", Page::Pickers, Light),
    Scene::new("13-inbox-light.png", Page::Inbox, Light),
    Scene::new("14-inbox-dark.png", Page::Inbox, Dark),
    Scene::new("15-settings-dark.png", Page::Settings, Dark),
    Scene::new("16-settings-light.png", Page::Settings, Light),
    Scene::new("17-workspace-dark.png", Page::Workspace, Dark),
    Scene::new("18-orders-light.png", Page::Orders, Light),
    Scene::new("19-tracker-light.png", Page::Tracker, Light),
];

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
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: spenstar-gallery [--capture-dir <dir>]",
            ));
        };
        if args.next().is_some() || directory.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: spenstar-gallery [--capture-dir <dir>]",
            ));
        }
        let directory = PathBuf::from(directory);
        fs::create_dir_all(&directory)?;
        Ok(Some(Self {
            directory,
            scene: 0,
            state: State::Configure,
            result: Arc::default(),
        }))
    }

    pub(super) fn result(&self) -> ResultSlot {
        self.result.clone()
    }

    pub(super) fn open_select(&self) -> bool {
        SCENES
            .get(self.scene)
            .is_some_and(|scene| scene.page == Page::Pickers)
    }

    pub(super) fn prepare(&mut self, gallery: &mut Gallery, ctx: &Context) {
        if !matches!(self.state, State::Configure) {
            return;
        }
        let scene = &SCENES[self.scene];
        if self.scene == 0 && (ctx.pixels_per_point() - 2.).abs() > f32::EPSILON {
            ctx.set_pixels_per_point(2.);
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(SIZE.into()));
        }
        gallery.page = scene.page;
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
        self.state = State::Settle {
            since: Instant::now(),
            frames: 0,
        };
    }

    pub(super) fn after_frame(&mut self, ctx: &Context) {
        match &mut self.state {
            State::Settle { since, frames } => {
                *frames += 1;
                if *frames >= 5 && since.elapsed() >= SETTLE {
                    let (sender, result) = mpsc::channel();
                    let filename = SCENES[self.scene].filename;
                    let path = self.directory.join(filename);
                    let ctx = ctx.clone();
                    let repaint = ctx.clone();
                    ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(
                        move |image| {
                            let outcome = write_image(&path, &image)
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
                    println!("Captured {}", SCENES[self.scene].filename);
                    self.scene += 1;
                    if self.scene == SCENES.len() {
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
                            SCENES[self.scene].filename
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

fn write_image(path: &Path, image: &ColorImage) -> Result<(), Box<dyn std::error::Error>> {
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
        PIXELS[0] as u32,
        PIXELS[1] as u32,
    );
    encoder.set_compression(png::Compression::Fast);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let pixels: Vec<_> = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_srgba_unmultiplied())
        .collect();
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    writer.finish()?;
    fs::rename(pending, path)?;
    Ok(())
}
