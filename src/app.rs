use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, Thread, JoinHandle, sleep},
    vec,
    time::{Duration, Instant}, io::ErrorKind,
};

// use super::drag_and_drop::DragAndDropDemo;
// use crate::drag_and_drop::Demo;
use egui::{pos2, Color32, Id, ScrollArea};
// use futures::channel::oneshot::{self, };
use tokio_modbus::{client::sync::Context, prelude::SyncReader};
static INDEX: [&str; 5] = [
    " Address  ",
    "  Enable  ",
    "SendFailed",
    "RecvFailed",
    " BadData  ",
];

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    // label: String,
    mullabel: String,
    open: bool,
    #[serde(skip)]
    open_window: HashMap<String, bool>, // 窗口是否打开
    #[serde(skip)]
    socket_window: HashMap<String, Arc<Mutex<Vec<u16>>>>, // 线程共享数组map
    // #[serde(skip)]
    // stat_window: HashMap<String, Vec<u16>>,
    #[serde(skip)]
    shut_thread: HashMap<String, Sender<bool>>, // 线程结束发送器
    #[serde(skip)]
    start_time: Vec<Instant>,
    host: String,
    log: Arc<Mutex<String>>,
    // this how you opt-out of serialization of a member
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            // label: "Hello World!".to_owned(),
            mullabel: "/dev/ttyS0".to_owned(),
            host: "127.0.0.1:502".to_owned(),
            log: Arc::new(Mutex::new(String::new())),
            open_window: HashMap::new(),
            socket_window: HashMap::new(),
            // stat_window: HashMap::new(),
            // dd: DragAndDropDemo::default(),
            shut_thread: HashMap::new(),
            start_time: vec![],
            open: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
    pub fn connect(
        host: &String,
        socket_window: &mut HashMap<String, Arc<Mutex<Vec<u16>>>>,
        shut_thread: &mut HashMap<String, Sender<bool>>,
        cnt: usize,
        log: Arc<Mutex<String>>,
    ) {
        host.lines().for_each(|s| {
            let (tx, rx) = channel();
            // shut_thread.insert((*(host.clone())).to_string(), tx);
            shut_thread.insert(s.to_string().clone(), tx);
            if let Ok(addr) = s.parse() {
                socket_window.insert(s.to_string(), Arc::new(Mutex::new(vec![0; cnt])));
                let _thread = Self::thread_query(
                    addr,
                    socket_window[&s.to_string()].clone(),
                    rx,
                    cnt,
                    log.clone(),
                );
            } else {
                println!("parse {} to IPAddr failed", s);
                // return Err(ErrorKind::AddrNotAvailable);
                return ()
            }
        });
    }
    pub fn thread_query(
        addr: SocketAddr,
        tx: Arc<Mutex<Vec<u16>>>,
        shot: Receiver<bool>, //线程结束接收器
        cnt: usize,
        log: Arc<Mutex<String>>,
    ) -> Result<JoinHandle<()>, std::io::Error> {
        thread::Builder::new()
            .name(addr.to_string())
            .spawn(move || {
                println!("start {:?} thread", thread::current());
                if let Ok(mut cxt) = tokio_modbus::client::sync::tcp::connect(addr) {
                    println!(
                        "{:?} thread connect slave ok,cnt {:?} ",
                        thread::current(),
                        cnt
                    );
                    loop {
                        sleep(Duration::from_secs_f32(0.5));
                        if let Ok(s) = cxt.read_input_registers(0, cnt as u16) {
                            println!("from {:?} recv {:?}", thread::current(), &s);
                            if let Ok(mut v) = tx.lock() {
                                v.clear();
                                s.iter().for_each(|&x| v.push(x));
                            }
                            if let Err(TryRecvError::Disconnected) = shot.try_recv() {
                                println!("thread {:?} exit", thread::current());
                                break;
                            }
                        }
                    }
                } else {
                    println!(
                        "{:?} thread connect slave failed. exit thread",
                        thread::current()
                    );
                }
            })
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            // label,
            mullabel,
            host,
            open: _,
            open_window,
            socket_window,
            // stat_window,
            start_time,
            shut_thread,
            log,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            // Tell the backend to repaint as soon as possible
            ctx.request_repaint();
            // // let the computer rest for a bit
            // ctx.request_repaint_after(std::time::Duration::from_secs_f32(
            //     self.repaint_after_seocnds,
            // ));            
            ui.add_space(10.0);
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.heading("SerialPort");
            ui.text_edit_multiline(mullabel);
            ui.heading("Host");
            ui.text_edit_multiline(host);
            // ui.heading("Log");
            // ui.text_edit_multiline(log);
            ui.horizontal(|ui| {
                ui.add_space(30.0);
                ui.spacing_mut().item_spacing.x = 100.0;
                if ui.button("connect").clicked() {
                    Self::connect(
                        host,
                        socket_window,
                        shut_thread,
                        mullabel.lines().count(),
                        log.clone(),
                    );
                    open_window.iter_mut().for_each(|(_, v)| *v = true);
                    self.open = true;
                    start_time.push(Instant::now());
                }
                if ui.button("disconnect").clicked() {
                    // tokio sync socket 不能disconnected
                    log.lock()
                        .unwrap()
                        .push_str(&format!("colse all connect.\n"));
                    self.open = false;
                    shut_thread.clear();
                    start_time.clear();
                }
            });
            if ui.button("Clear").clicked() {
                log.lock().unwrap().clear();
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
                if self.open  {
                    let text = format!("connect time: {:?}",start_time[0].elapsed().as_secs());
                    ui.label(text);
                }
            });
        });

        let mut new_window = |host: String| {
            if !open_window.contains_key(&host) {
                let _ = open_window.insert(host.clone(), true);
            };
            egui::Window::new(host.clone())
                .open(&mut open_window.get_mut(&host).unwrap())
                .resizable(true)
                .auto_sized()
                .show(ctx, |ui| {
                    // println!("flash windows");
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        ui.add_space(10.0);
                        for &s in INDEX.iter() {
                            ui.label(s);
                        }
                    });
                    let set_led = |b: bool| {
                        if b {
                            led(Color32::LIGHT_GREEN)
                        } else {
                            led(Color32::LIGHT_RED)
                        }
                    };
                    if let Some(data) = socket_window.get_mut(&host) {
                        if let Ok(stat) = data.lock() {
                            println!("{:?} stat is {:?}",host.clone(), stat);
                            for (cnt, s) in mullabel.lines().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(10.0);
                                        ui.spacing_mut().item_spacing.x = 35.0;
                                        ui.label(s);
                                        for i in 1..INDEX.len() {
                                            // println!("{:?}{:?}", INDEX[i],stat[cnt] >> (i - 1) & 1 == 1);
                                            ui.add_space(10.0);
                                            ui.add(set_led(stat[cnt] >> (i - 1) & 1 == 1));
                                        }
                                    });
                                });
                            }
                        } else {
                            println!("{:?} lock stat failed ", &host);
                        }
                    } else {
                        println!("get {:?} from socket window failed", &host);
                    }
                })
        };
        // dd.show(ctx, &mut self.open);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                let (current_scroll, max_scroll) = ScrollArea::vertical()
                    .max_height(400.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for s in log.lock().unwrap().lines() {
                                ui.label(s);
                            }
                        });
                        let margin = ui.visuals().clip_rect_margin;
                        let current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                        let max_scroll =
                            ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;
                        (current_scroll, max_scroll)
                    })
                    .inner;
                ui.label(format!(
                    "Scroll offset: {:.0}/{:.0} px",
                    current_scroll, max_scroll
                ));
            });
        });
        if self.open {
            for s in host.lines() {
                new_window(s.to_string());
            }
        }
    }

    fn on_exit_event(&mut self) -> bool {
        true
    }

    fn on_exit(&mut self, _gl: &eframe::glow::Context) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::Vec2::INFINITY
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }
}

fn led_ui_compact(ui: &mut egui::Ui, fill_color: impl Into<Color32>) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(1.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Button));
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        ui.painter()
            .circle_filled(rect.center(), 0.75 * radius, fill_color);
    }
    response
}
// A led show stat: `ui.add(led(Color32::LIGHT_GREEN))`
///
/// ## Example:
/// ``` ignore
/// ui.add(led(Color32::LIGHT_GREEN));
/// ```
pub fn led<'a>(fill_color: impl Into<Color32> + 'a) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| led_ui_compact(ui, fill_color)
}
