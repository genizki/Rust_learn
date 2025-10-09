use eframe::egui::{self, Button, Color32, Rect, vec2};
use serde::{Deserialize, Serialize};

// laod .env variables
use dotenv::dotenv;
use std::{env, f32};

// Api crates
use reqwest::Client;
use tokio::{self}; //asynch

// const
pub const WIDTH: f32 = 120.0;
pub const HEIGHT: f32 = 120.0;

#[cfg(target_os = "macos")]
const _DOWNLOAD_PATH: &str = "~/Downloads";

#[cfg(target_os = "windows")]
const DOWNLOAD_PATH: &str = "%USERPROFILE%\\Downloads";

#[cfg(target_os = "windows")]
const YT_DLP_BINARY: &str = "./yt_dlp/yt-dlp.exe";

#[cfg(target_os = "macos")]
const YT_DLP_BINARY: &str = "./yt_dlp/yt-dlp_macos";

enum WorkerMessage {
    Data(SearchResponse),
    Progress(u32),
    Error(String),
    Done(usize),
}

struct TokioWorker {
    tx: tokio::sync::mpsc::Sender<WorkerMessage>,
    rx: tokio::sync::mpsc::Receiver<WorkerMessage>,
}
impl Default for TokioWorker {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(102);
        Self { tx, rx }
    }
}

#[derive(Default)]
enum AppState {
    #[default]
    App,
    Settings,
    Test,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct SettingsState {
    max_results: i8,
    first_run: bool,
    download_path: String,
}
impl SettingsState {
    fn default() -> Self {
        Self {
            max_results: 8,
            first_run: true,
            download_path: _DOWNLOAD_PATH.to_string(),
        }
    }
}

#[derive(Default)]
struct YtGUI {
    data: SearchResponse,
    search_item: Vec<SearchResponseMeta>,
    search_text: String,
    side_width: f32,
    settings_state: SettingsState,
    image_loader_installed: bool,
    app_state: AppState,
    tokio_worker: TokioWorker,
}

impl YtGUI {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings_state: SettingsState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        Self {
            settings_state,
            ..Default::default()
        }
    }
    fn search_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::default()
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2 { x: 0.0, y: 0.0 };
                ui.vertical_centered(|ui| {
                    ui.horizontal_top(|ui| {
                        // ui.add_space();
                        // println!("{}", ui.available_width());
                        let avaibale_width = ui.available_width();
                        let searchfield_width = avaibale_width * 0.40;
                        let search_button_width = avaibale_width * 0.10;
                        let spacing =
                            (avaibale_width - (searchfield_width + search_button_width)) / 2.0;

                        ui.add_space(spacing);
                        let searchfield = ui.add(
                            egui::TextEdit::singleline(&mut self.search_text)
                                .hint_text("Search here")
                                .desired_width(searchfield_width)
                                .min_size(vec2(330.0, 20.0)),
                        );
                        let search_button = ui.add(Button::new("🔍"));

                        if searchfield.clicked() {
                            searchfield.request_focus();
                        }
                        if !searchfield.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            || search_button.clicked()
                        {
                            let search_string = self.search_text.clone();
                            let max_reults = self.settings_state.max_results.clone();
                            let rx = self.tokio_worker.tx.clone();
                            let ctx_giver = ctx.clone();

                            tokio::spawn(async move {
                                let data = call_yt_api(search_string, max_reults).await.unwrap();
                                rx.send(WorkerMessage::Data(data)).await.unwrap();
                                // rx.send({ data })
                                ctx_giver.request_repaint();
                            });

                            self.search_text.clear();
                        }

                        ui.add_space(spacing);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.add(Button::new("⚙")).clicked() {
                                self.app_state = AppState::Settings;
                            }
                        });
                    })
                    .response;
                    ui.allocate_space(vec2(ui.available_width(), 10.0));

                    ui.add_space(40.0);
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                for (index, item) in &mut self.data.items.iter().enumerate() {
                                    self.search_item.push(SearchResponseMeta {
                                        is_enabled: true,
                                        download_progress: 0,
                                    });
                                    ui.horizontal(|ui| {
                                        let thumbnail_url: &str = if let Some(ref thumb) =
                                            item.snippet.thumbnails.default
                                        {
                                            &thumb.url
                                        } else {
                                            "notfound"
                                        };

                                        let image = egui::Image::from_uri(thumbnail_url)
                                            .fit_to_exact_size(vec2(WIDTH, HEIGHT));
                                        ui.add(image);

                                        ui.add_space(40.0);
                                        ui.vertical(|ui| {
                                            ui.label(&item.snippet.title);
                                            ui.colored_label(
                                                Color32::GRAY,
                                                &item.snippet.channel_title,
                                            );
                                            ui.add_space(10.0);

                                            if ui
                                                .add_enabled(
                                                    self.search_item[index].is_enabled,
                                                    egui::Button::new("Download"),
                                                )
                                                .clicked()
                                            {
                                                self.search_item[index].is_enabled = false;
                                                if let Some(video_id) = &item.id.video_id {
                                                    let yt_link = format!(
                                                        "https://www.youtube.com/watch?v={}",
                                                        video_id
                                                    );
                                                    println!(
                                                        "{}",
                                                        &self.settings_state.download_path
                                                    );
                                                    let path =
                                                        self.settings_state.download_path.clone();
                                                    let yt_link = yt_link.clone(); // auch Borrow zu String machen!
                                                    let item_id = index.clone();

                                                    let tx = self.tokio_worker.tx.clone();
                                                    tokio::spawn(async move {
                                                        downlaod_from_dlp(
                                                            tx, item_id, &yt_link, &path, "aac",
                                                        )
                                                        .await;
                                                    });
                                                } else {
                                                    println!(
                                                        "Fehler Video_id nicht gefunden. Think"
                                                    );
                                                }
                                            }
                                        });
                                    });
                                    ui.add_space(20.0);
                                    ui.add(egui::Separator::default());
                                    ui.add_space(20.0);
                                }
                            });
                        ui.allocate_space(ui.available_size());
                    });
                });
            })
            .response;
    }
}

impl eframe::App for YtGUI {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.settings_state);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.settings_state.first_run {
            global_fontsize(ctx);
            self.settings_state.first_run = false;
            self.settings_state.download_path = _DOWNLOAD_PATH.to_string();
        }
        let screen_rect = ctx.screen_rect();
        let panel_size = calc_grid_size(&screen_rect, None);
        self.side_width = panel_size.side_width;

        if !self.image_loader_installed {
            egui_extras::install_image_loaders(ctx);
            self.image_loader_installed = true
        }
        if let Ok(msg) = self.tokio_worker.rx.try_recv() {
            match msg {
                WorkerMessage::Done(index) => {
                    self.search_item[index].is_enabled = true;
                }
                WorkerMessage::Progress(progress_value) => {}
                WorkerMessage::Error(error_msg) => {}
                WorkerMessage::Data(data) => {
                    self.data = data;
                }
            }
        }

        match self.app_state {
            AppState::App => {
                layout(self.side_width, ctx, |ui| self.search_bar(ctx, ui), false);
            }
            AppState::Settings => {
                layout(
                    self.side_width,
                    ctx,
                    |ui| {
                        egui::Grid::new("settings_header")
                            .num_columns(3)
                            .spacing([ui.available_width() / 3.0, 0.0])
                            .show(ui, |ui| {
                                let av_space = ui.available_width();
                                let spacer = av_space / 2.0;
                                if ui.button("back to app").clicked() {
                                    self.app_state = AppState::App;
                                }
                                let button_size = av_space - ui.available_width();
                                let button_spacer = spacer - button_size;

                                // ui.add_space(spacer - button_size);
                                ui.label("settings");
                                // ui.add_space(spacer);
                                ui.end_row();
                            });
                        ui.add_space(40.0);
                        ui.add(egui::Slider::new(
                            &mut self.settings_state.max_results,
                            0..=25,
                        ));
                        if ui.button("press me").clicked() {
                            let output = std::process::Command::new("pwd").output();
                            println!("{:?}", output);
                        }
                        if ui.button("test me").clicked() {
                            let output = std::process::Command::new(YT_DLP_BINARY)
                                .arg("--version")
                                .output();
                            println!("{:?}", output);
                        }
                    },
                    false,
                );
            }
            AppState::Test => {
                layout(
                    self.side_width,
                    ctx,
                    |ui| {
                        if ui.button("yt_dlp me").clicked() {
                            let args = [
                                "-x",
                                "--audio-format",
                                "aac",
                                "-o",
                                "~/Downloads/%(title)s.%(ext)s",
                                "--add-metadata",
                                "https://www.youtube.com/watch?v=5kfPCxXZPdA",
                                "--ffmpeg-location",
                                "./ffmpeg/ffmpeg",
                            ];
                            let output = std::process::Command::new(YT_DLP_BINARY)
                                .args(&args)
                                .output()
                                .expect("Failed halt");
                            println!("{:?}", output);
                            println!("Exit status: {}", output.status);

                            // stdout (normale Ausgabe)
                            let stdout_str = String::from_utf8_lossy(&output.stdout);
                            if !stdout_str.trim().is_empty() {
                                println!("stdout:\n{}", stdout_str);
                            }

                            // stderr (Fehlermeldung)
                            let stderr_str = String::from_utf8_lossy(&output.stderr);
                            if !stderr_str.trim().is_empty() {
                                println!("stderr:\n{}", stderr_str);
                            }
                        }
                    },
                    true,
                );
            }
        }
    }
}

fn global_fontsize(ctx: &egui::Context) {
    ctx.style_mut(|style| {
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(32.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(20.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
    });
}

fn calc_grid_size(screen_rect: &Rect, scaling_factor: Option<f32>) -> PanelSize {
    const WIDTH_THRESHOLD: f32 = 1000.0;
    // let screen_min = screen_rect.min;

    let screen_max = screen_rect.max;
    let mut side_width: f32 = 0.0;
    let mut central_width: f32;
    let max_width: f32 = screen_max.x;

    central_width = max_width;

    if central_width >= WIDTH_THRESHOLD {
        side_width = (max_width - WIDTH_THRESHOLD) / scaling_factor.unwrap_or(2.5);
        central_width = central_width - side_width;
    }

    // println!("central:{central_width}, side: {side_width}");
    // let right_side = left_side.clone();
    PanelSize {
        side_width,
        _central_width: central_width,
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Enviroment variablen aus der .env laden
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            // title: (),
            // app_id: (),
            // position: (),
            // inner_size: (),
            min_inner_size: Some(egui::vec2(800.0, 600.0)),
            // max_inner_size: (),
            // clamp_size_to_monitor_size: (),
            // fullscreen: (),
            // maximized: (),
            // resizable: (),
            // transparent: (),
            // decorations: (),
            // icon: (),
            // active: (),
            // visible: (),
            // fullsize_content_view: (),
            // movable_by_window_background: (),
            // title_shown: (),
            // titlebar_buttons_shown: (),
            // titlebar_shown: (),
            // has_shadow: (),
            // drag_and_drop: (),
            // taskbar: (),
            // close_button: (),
            // minimize_button: (),
            // maximize_button: (),
            // window_level: (),
            // mouse_passthrough: (),
            // window_type: (),
            ..Default::default()
        },
        ..Default::default()
    };

    let app = eframe::run_native(
        "Hier Name",
        options,
        Box::new(|cc| Ok(Box::new(YtGUI::new(cc)))),
    );
    if let Err(error) = app {
        eprint!("Fehler beim Starten der App: {}", error);
    }

    // let _data = call_yt_api("this is the best song".to_string(), 10);
}

fn layout<Central>(
    side_width: f32,
    ctx: &egui::Context,
    // leftside_content: egui::Response,
    // rightside_content: egui::Response,
    central_content: Central,
    dev_mode: bool,
) where
    Central: FnOnce(&mut egui::Ui),
{
    egui::SidePanel::left(egui::Id::new("left_side"))
        .exact_width(side_width)
        .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
        .show_separator_line(dev_mode)
        .resizable(false)
        .show(ctx, |_ui| {});

    egui::SidePanel::right(egui::Id::new("right_side"))
        .exact_width(side_width)
        .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
        .show_separator_line(dev_mode)
        .resizable(false)
        .show(ctx, |_ui| {});

    egui::CentralPanel::default().show(ctx, |_ui| {
        central_content(_ui);
    });
}

async fn call_yt_api(
    query: String,
    max_results: i8,
) -> Result<SearchResponse, Box<dyn std::error::Error>> {
    let yt_key: String = env::var("YT_API").unwrap();
    // let yt_key: String = "nein NeinA".to_string();

    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&key={}&maxResults={}&type=video&videCategoryId=10",
        query.replace(" ", "%20"),
        yt_key,
        max_results
    );
    println!("{url}");

    let client = Client::new();
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        println!("Request failed: {}", response.status());
    }
    let data: SearchResponse = response.json::<SearchResponse>().await?;
    println!("Alle Youtube Title: ");
    let mut index: i8 = 1;
    for item in &data.items {
        let video_title = &item.snippet.title;
        println!("{index}: {video_title}");
        index += 1;
    }
    Ok(data) // main must return something, in this case (). Finish request block and change Result type to SeachResponse
}

struct SearchResponseMeta {
    is_enabled: bool,
    download_progress: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SearchResponse {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    etag: String,
    #[serde(rename = "nextPageToken", default)]
    next_page_token: String,
    #[serde(rename = "regionCode", default)]
    region_code: String,
    #[serde(rename = "pageInfo", default)]
    page_info: Option<PageInfo>,
    #[serde(default)]
    items: Vec<SearchItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchItem {
    pub kind: String,
    pub etag: String,
    pub id: Id,
    pub snippet: Snippet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "totalResults")]
    pub total_results: u64,
    #[serde(rename = "resultsPerPage")]
    pub results_per_page: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Id {
    pub kind: String,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snippet {
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: Thumbnails,
    #[serde(rename = "channelTitle")]
    pub channel_title: String,
    #[serde(rename = "liveBroadcastContent")]
    pub live_broadcast_content: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnails {
    pub default: Option<ThumbnailData>,
    pub medium: Option<ThumbnailData>,
    pub high: Option<ThumbnailData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThumbnailData {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

struct PanelSize {
    side_width: f32,
    _central_width: f32,
}

async fn downlaod_from_dlp(
    rx: tokio::sync::mpsc::Sender<WorkerMessage>,
    item_id: usize,
    url: &String,
    download_path: &String,
    audio_format: &'static str,
) {
    let download_string = format!("{download_path}/%(title)s.%(ext)s");

    let command = [
        "-x",
        "--audio-format",
        { &audio_format },
        "-o",
        { &download_string },
        "--add-metadata",
        "--ffmpeg-location",
        "./ffmpeg/ffmpeg",
        // "--write-thumbnail",
        { &url },
    ];

    tokio::process::Command::new(YT_DLP_BINARY)
        .args(&command)
        .stdout(std::process::Stdio::piped());

    rx.send(WorkerMessage::Done(item_id)).await.unwrap();
}
