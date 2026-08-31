use gtk::prelude::*;
use gtk::{glib, Align, Orientation, PolicyType};
use relm4::prelude::*;
use std::time::Duration;

use aviet_shared::models::*;
use aviet_shared::api::AvietApiClient;

#[derive(Debug, Clone)]
enum AppMsg {
    CheckServerStatus,
    ServerStatusReceived(Result<ServerStatus, String>),
    PhoneChanged(String),
    PasswordChanged(String),
    Login,
    LoginResult(Result<LoginResponse, String>),
    Logout,
    StartAutoPlay,
    StopAutoPlay,
    FetchGameState,
    GameStateReceived(Result<GameStateResponse, String>),
    FetchPayouts,
    PayoutsReceived(Result<PayoutsResponse, String>),
    FetchBalance,
    BalanceReceived(Result<BalanceResponse, String>),
    UpdateStatus(String),
    ToggleTheme,
    ClearLogs,
}

struct App {
    api_client: AvietApiClient,
    session_id: Option<uuid::Uuid>,
    phone: String,
    password: String,
    status_message: String,
    is_dark_mode: bool,
    logs: Vec<String>,
    game_state: Option<GameState>,
    balance: String,
    payouts: Vec<String>,
    auto_play_active: bool,
}

#[relm4::component]
impl SimpleComponent for App {
    type Input = AppMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::ApplicationWindow {
            set_size_request: (800, 700),
            set_title: Some("Aviet Desktop"),

            gtk::Box {
                set_orientation: Orientation::Vertical,
                set_margin_all: 12,
                set_spacing: 12,

                gtk::Box {
                    set_orientation: Orientation::Horizontal,
                    set_spacing: 12,

                    gtk::Label {
                        set_label: "Aviet Desktop",
                        add_css_class: "title-1",
                    },

                    gtk::Button {
                        #[watch]
                        set_icon_name: if model.is_dark_mode { "weather-clear-symbolic" } else { "weather-clear-night-symbolic" },
                        connect_clicked => AppMsg::ToggleTheme,
                    },
                },

                gtk::Frame {
                    set_label: Some("Server Connection"),

                    gtk::Box {
                        set_orientation: Orientation::Horizontal,
                        set_spacing: 12,
                        set_margin_all: 12,

                        gtk::Label {
                            #[watch]
                            set_label: &model.status_message,
                        },

                        gtk::Button {
                            set_label: "Refresh",
                            connect_clicked => AppMsg::CheckServerStatus,
                        },
                    },
                },

                gtk::Frame {
                    set_label: Some("Authentication"),

                    gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_spacing: 8,
                        set_margin_all: 12,

                        gtk::Entry {
                            set_placeholder_text: Some("Phone Number"),
                            set_text: &model.phone,
                            connect_changed[sender] => move |entry| {
                                sender.input(AppMsg::PhoneChanged(entry.text().to_string()));
                            }
                        },

                        gtk::Entry {
                            set_placeholder_text: Some("Password"),
                            set_visibility: false,
                            set_text: &model.password,
                            connect_changed[sender] => move |entry| {
                                sender.input(AppMsg::PasswordChanged(entry.text().to_string()));
                            }
                        },

                        gtk::Button {
                            set_label: "Login",
                            add_css_class: "suggested-action",
                            #[watch]
                            set_sensitive: model.session_id.is_none(),
                            connect_clicked => AppMsg::Login,
                        },

                        gtk::Button {
                            set_label: "Logout",
                            #[watch]
                            set_sensitive: model.session_id.is_some(),
                            connect_clicked => AppMsg::Logout,
                        },
                    },
                },

                gtk::Frame {
                    set_label: Some("Game Controls"),

                    gtk::Box {
                        set_orientation: Orientation::Horizontal,
                        set_spacing: 12,
                        set_margin_all: 12,

                        gtk::Button {
                            #[watch]
                            set_label: if model.auto_play_active { "Stop Auto Play" } else { "Start Auto Play" },
                            #[watch]
                            set_sensitive: model.session_id.is_some(),
                            connect_clicked => if model.auto_play_active {
                                AppMsg::StopAutoPlay
                            } else {
                                AppMsg::StartAutoPlay
                            },
                        },

                        gtk::Button {
                            set_label: "Refresh State",
                            connect_clicked => AppMsg::FetchGameState,
                        },

                        gtk::Button {
                            set_label: "Get Balance",
                            connect_clicked => AppMsg::FetchBalance,
                        },

                        gtk::Button {
                            set_label: "Get Payouts",
                            connect_clicked => AppMsg::FetchPayouts,
                        },
                    },
                },

                gtk::Frame {
                    set_label: Some("Game Status"),

                    gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_margin_all: 12,

                        gtk::Label {
                            #[watch]
                            set_label: &format!("Session: {:?}\nBalance: {}\nPayouts: {}\nAuto Play: {}\nPhase: {:?}",
                                model.session_id,
                                model.balance,
                                model.payouts.len(),
                                model.auto_play_active,
                                model.game_state.as_ref().map(|s| s.phase.clone()).unwrap_or(GamePhase::Idle)
                            ),
                            set_xalign: 0.0,
                        },
                    },
                },

                gtk::Frame {
                    set_label: Some("Logs"),
                    set_vexpand: true,

                    gtk::Box {
                        set_orientation: Orientation::Vertical,

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_vscrollbar_policy: PolicyType::Automatic,

                            gtk::Label {
                                #[watch]
                                set_label: &model.logs.join("\n"),
                                set_xalign: 0.0,
                                set_yalign: 0.0,
                                set_margin_all: 12,
                            },
                        },

                        gtk::Button {
                            set_label: "Clear Logs",
                            set_halign: Align::End,
                            set_margin_all: 8,
                            connect_clicked => AppMsg::ClearLogs,
                        },
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = App {
            api_client: AvietApiClient::new(),
            session_id: None,
            phone: String::new(),
            password: String::new(),
            status_message: "Click Refresh to check server".to_string(),
            is_dark_mode: true,
            logs: vec!["[INIT] Desktop client started".to_string()],
            game_state: None,
            balance: "0.00".to_string(),
            payouts: Vec::new(),
            auto_play_active: false,
        };

        let widgets = view_output!();

        let sender_poll = sender.clone();
        glib::timeout_add_local(Duration::from_secs(2), move || {
            sender_poll.input(AppMsg::FetchGameState);
            glib::ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::CheckServerStatus => {
                let client = self.api_client.clone();
                tokio::spawn(async move {
                    match client.server_status().await {
                        Ok(status) => sender.input(AppMsg::ServerStatusReceived(Ok(status))),
                        Err(e) => sender.input(AppMsg::ServerStatusReceived(Err(e.to_string()))),
                    }
                });
            }

            AppMsg::ServerStatusReceived(result) => {
                match result {
                    Ok(status) => {
                        self.status_message = format!("Server v{} | Uptime: {}s | Sessions: {}",
                            status.version, status.uptime_seconds, status.active_sessions);
                        self.add_log(&format!("[SERVER] Connected, version {}", status.version));
                    }
                    Err(e) => {
                        self.status_message = format!("Server offline: {}", e);
                        self.add_log(&format!("[ERROR] {}", e));
                    }
                }
            }

            AppMsg::PhoneChanged(phone) => self.phone = phone,
            AppMsg::PasswordChanged(password) => self.password = password,

            AppMsg::Login => {
                if self.phone.is_empty() || self.password.is_empty() {
                    self.status_message = "Enter phone and password".to_string();
                    return;
                }
                let client = self.api_client.clone();
                let phone = self.phone.clone();
                let password = self.password.clone();
                tokio::spawn(async move {
                    match client.login(&phone, &password, "Betika").await {
                        Ok(resp) => sender.input(AppMsg::LoginResult(Ok(resp))),
                        Err(e) => sender.input(AppMsg::LoginResult(Err(e.to_string()))),
                    }
                });
            }

            AppMsg::LoginResult(result) => {
                match result {
                    Ok(resp) => {
                        if resp.success {
                            self.session_id = Some(resp.session_id);
                            self.status_message = format!("Logged in: {}", resp.message);
                            self.add_log(&format!("[LOGIN] Success, session {}", resp.session_id));
                        } else {
                            self.status_message = resp.message.clone();
                            self.add_log(&format!("[LOGIN] Failed: {}", resp.message));
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Login error: {}", e);
                        self.add_log(&format!("[LOGIN] Error: {}", e));
                    }
                }
            }

            AppMsg::Logout => {
                if let Some(session_id) = self.session_id {
                    let client = self.api_client.clone();
                    tokio::spawn(async move {
                        let _ = client.logout(session_id).await;
                    });
                }
                self.session_id = None;
                self.auto_play_active = false;
                self.add_log("[LOGOUT] Session cleared");
            }

            AppMsg::StartAutoPlay => {
                if let Some(session_id) = self.session_id {
                    let client = self.api_client.clone();
                    tokio::spawn(async move {
                        let req = AutoPlayRequest {
                            session_id,
                            strategy_id: uuid::Uuid::new_v4(),
                            bet_amount: None,
                            demo_mode: false,
                        };
                        match client.start_auto_play(req).await {
                            Ok(resp) => {
                                if resp.success {
                                    sender.input(AppMsg::UpdateStatus("Auto-play started".to_string()));
                                }
                            }
                            Err(e) => sender.input(AppMsg::UpdateStatus(format!("Error: {}", e))),
                        }
                    });
                    self.auto_play_active = true;
                }
            }

            AppMsg::StopAutoPlay => {
                if let Some(session_id) = self.session_id {
                    let client = self.api_client.clone();
                    tokio::spawn(async move {
                        let req = StopAutoPlayRequest { session_id };
                        let _ = client.stop_auto_play(req).await;
                    });
                    self.auto_play_active = false;
                }
            }

            AppMsg::FetchGameState => {
                if let Some(session_id) = self.session_id {
                    let client = self.api_client.clone();
                    tokio::spawn(async move {
                        match client.get_game_state(session_id).await {
                            Ok(state) => sender.input(AppMsg::GameStateReceived(Ok(state))),
                            Err(e) => sender.input(AppMsg::GameStateReceived(Err(e.to_string()))),
                        }
                    });
                }
            }

            AppMsg::GameStateReceived(result) => {
                if let Ok(resp) = result {
                    self.game_state = Some(resp.state.clone());
                    self.balance = resp.state.current_balance.clone();
                    self.auto_play_active = resp.state.auto_play_active;
                }
            }

            AppMsg::FetchPayouts => {
                if let Some(session_id) = self.session_id {
                    let client = self.api_client.clone();
                    tokio::spawn(async move {
                        match client.get_payouts(session_id).await {
                            Ok(payouts) => sender.input(AppMsg::PayoutsReceived(Ok(payouts))),
                            Err(e) => sender.input(AppMsg::PayoutsReceived(Err(e.to_string()))),
                        }
                    });
                }
            }

            AppMsg::PayoutsReceived(result) => {
                if let Ok(resp) = result {
                    self.payouts = resp.payouts;
                }
            }

            AppMsg::FetchBalance => {
                if let Some(session_id) = self.session_id {
                    let client = self.api_client.clone();
                    tokio::spawn(async move {
                        match client.get_balance(session_id).await {
                            Ok(balance) => sender.input(AppMsg::BalanceReceived(Ok(balance))),
                            Err(e) => sender.input(AppMsg::BalanceReceived(Err(e.to_string()))),
                        }
                    });
                }
            }

            AppMsg::BalanceReceived(result) => {
                if let Ok(resp) = result {
                    if resp.success {
                        self.balance = resp.balance.unwrap_or_else(|| "0.00".to_string());
                    }
                }
            }

            AppMsg::UpdateStatus(msg) => {
                self.status_message = msg.clone();
                self.add_log(&format!("[STATUS] {}", msg));
            }

            AppMsg::ToggleTheme => {
                self.is_dark_mode = !self.is_dark_mode;
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_gtk_application_prefer_dark_theme(self.is_dark_mode);
                }
            }

            AppMsg::ClearLogs => {
                self.logs.clear();
                self.add_log("[LOGS] Cleared");
            }
        }
    }
}

impl App {
    fn add_log(&mut self, msg: &str) {
        let now = chrono::Local::now().format("%H:%M:%S");
        self.logs.push(format!("[{}] {}", now, msg));
        if self.logs.len() > 200 {
            self.logs.remove(0);
        }
    }
}

fn main() {
    let app = RelmApp::new("com.aviet.desktop");
    app.run::<App>(());
}
