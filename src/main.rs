use eframe::egui::{self, Button, Color32, Rect, vec2};
use serde::{Deserialize, Serialize};

// laod .env variables
use dotenv::dotenv;
use std::{env, f32};

// Api crates
use reqwest::Client;
use tokio; //asynch

// const
pub const WIDTH: f32 = 120.0;
pub const HEIGHT: f32 = 120.0;

#[derive(Default)]
enum AppState {
    #[default]
    App,
    Settings,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct SettingsState {
    max_results: i8,
    first_run: bool,
}
impl SettingsState {
    fn _default() -> Self {
        Self {
            max_results: 8,
            first_run: true,
        }
    }
}

#[derive(Default)]
struct YtGUI {
    data: SearchResponse,
    search_text: String,
    side_width: f32,
    settings_state: SettingsState,
    image_loader_installed: bool,
    app_state: AppState,
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
    fn search_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default().show(ui, |ui| {
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
                        self.data = call_yt_api(&self.search_text, self.settings_state.max_results)
                            .unwrap();
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
                            for item in &mut self.data.items {
                                ui.horizontal(|ui| {
                                    let thumbnail_url: &str =
                                        if let Some(ref thumb) = item.snippet.thumbnails.default {
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
                                        if ui.add(Button::new("Download")).clicked() {
                                            if let Some(video_id) = &item.id.video_id {
                                                let yt_link = format!(
                                                    "https://www.youtube.com/watch?v={}",
                                                    video_id
                                                );
                                                println!("python socket... {}", yt_link);
                                            } else {
                                                println!("Fehler Video_id nicht gefunden. Think");
                                            }
                                        };
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
        });
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
        }
        let screen_rect = ctx.screen_rect();
        let panel_size = calc_grid_size(&screen_rect, None);
        self.side_width = panel_size.side_width;

        if !self.image_loader_installed {
            egui_extras::install_image_loaders(ctx);
            self.image_loader_installed = true
        }

        match self.app_state {
            AppState::App => {
                // println!("{}", screen_rect);
                // println!("{}, {central_width}", self.side_width);

                egui::SidePanel::left(egui::Id::new("left_side"))
                    .exact_width(self.side_width)
                    .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
                    .show_separator_line(false)
                    .resizable(false)
                    .show(ctx, |_ui| {});

                egui::SidePanel::right(egui::Id::new("right_side"))
                    .exact_width(self.side_width)
                    .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
                    .show_separator_line(false)
                    .resizable(false)
                    .show(ctx, |_ui| {});

                egui::CentralPanel::default().show(ctx, |ui| {
                    self.search_bar(ui);
                });
            }
            AppState::Settings => {
                egui::SidePanel::left(egui::Id::new("left_side"))
                    .exact_width(self.side_width)
                    .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
                    .show(ctx, |_ui| {});

                egui::SidePanel::right(egui::Id::new("right_side"))
                    .exact_width(self.side_width)
                    .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
                    .show(ctx, |_ui| {});

                egui::CentralPanel::default().show(ctx, |ui| {
                    if ui.button("back to app").clicked() {
                        self.app_state = AppState::App;
                    }
                    ui.add(egui::Slider::new(
                        &mut self.settings_state.max_results,
                        0..=25,
                    ));
                });
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

fn main() {
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

#[tokio::main]
async fn call_yt_api(
    query: &String,
    max_results: i8,
) -> Result<SearchResponse, Box<dyn std::error::Error>> {
    let yt_key: String = env::var("YT_API").unwrap();
    // let yt_key: String = "nein NeinA".to_string();

    let url = format!(
        "https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&key={}&maxResults={}&type=video",
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
