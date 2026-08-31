use thirtyfour::prelude::*;
use thirtyfour::WebDriver;
use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::mpsc;
use uuid::Uuid;
use tracing::info;

use aviet_shared::models::*;

#[derive(Debug, Clone)]
pub enum BrowserEvent {
    PayoutChanged(Vec<String>),
    BalanceChanged(String),
    GameStateChanged(GameState),
    BetConfirmed,
    CashoutConfirmed(f64),
    RoundCrashed,
    Error(String),
    Log(String),
}

pub struct BrowserSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub driver: WebDriver,
    pub site: SiteProfile,
    pub event_tx: mpsc::UnboundedSender<BrowserEvent>,
    pub game_phase: GamePhase,
    pub current_balance: String,
    pub payouts: Vec<String>,
    pub auto_play_active: bool,
    pub strategy: Option<StrategyData>,
    pub game_history: GameHistoryData,
    pub is_demo_mode: bool,
    pub last_bet_timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct StrategyData {
    pub tuples: Vec<(f64, f64)>,
    pub expected_amount: f64,
    pub current_index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct GameHistoryData {
    pub total_wins: u32,
    pub total_losses: u32,
    pub current_strategy_index: usize,
}

impl BrowserSession {
    pub async fn new(
        session_id: Uuid,
        user_id: Uuid,
        geckodriver_url: &str,
        headless: bool,
        site: SiteProfile,
        event_tx: mpsc::UnboundedSender<BrowserEvent>,
    ) -> Result<Self, String> {
        info!("Creating new browser session {} for site {}", session_id, site.name);

        let driver = Self::create_driver(geckodriver_url, headless).await?;

        driver.goto(&site.url).await.map_err(|e| format!("Failed to navigate: {}", e))?;
        sleep(Duration::from_secs(3)).await;

        Ok(Self {
            session_id,
            user_id,
            driver,
            site,
            event_tx,
            game_phase: GamePhase::Idle,
            current_balance: "0.00".to_string(),
            payouts: Vec::new(),
            auto_play_active: false,
            strategy: None,
            game_history: GameHistoryData::default(),
            is_demo_mode: false,
            last_bet_timestamp: 0,
        })
    }

    async fn create_driver(geckodriver_url: &str, headless: bool) -> Result<WebDriver, String> {
        let mut caps = DesiredCapabilities::firefox();

        let mut prefs = FirefoxPreferences::new();
        prefs.set("dom.webdriver.enabled", false).map_err(|e| e.to_string())?;
        prefs.set("useAutomationExtension", false).map_err(|e| e.to_string())?;
        prefs.set("general.useragent.override", "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0").map_err(|e| e.to_string())?;
        prefs.set("intl.accept_languages", "en-US,en").map_err(|e| e.to_string())?;
        prefs.set("browser.startup.homepage", "about:blank").map_err(|e| e.to_string())?;
        prefs.set("browser.startup.page", 0).map_err(|e| e.to_string())?;
        prefs.set("browser.sessionstore.resume_from_crash", false).map_err(|e| e.to_string())?;
        prefs.set("browser.shell.checkDefaultBrowser", false).map_err(|e| e.to_string())?;
        prefs.set("browser.warnOnQuit", false).map_err(|e| e.to_string())?;
        prefs.set("browser.tabs.warnOnClose", false).map_err(|e| e.to_string())?;
        prefs.set("dom.disable_beforeunload", true).map_err(|e| e.to_string())?;
        prefs.set("privacy.popups.disable_popup_notifications", true).map_err(|e| e.to_string())?;
        prefs.set("media.volume_scale", "0.0").map_err(|e| e.to_string())?;
        prefs.set("toolkit.telemetry.enabled", false).map_err(|e| e.to_string())?;
        prefs.set("browser.newtabpage.enabled", false).map_err(|e| e.to_string())?;

        caps.set_preferences(prefs).map_err(|e| e.to_string())?;

        let mut args = vec![
            "--width=1480",
            "--height=1080",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--disable-gpu",
        ];

        if headless {
            args.push("--headless");
        }

        for arg in args {
            caps.add_arg(arg).map_err(|e| e.to_string())?;
        }

        let driver = WebDriver::new(geckodriver_url, caps)
            .await
            .map_err(|e| format!("Failed to create WebDriver: {}", e))?;

        let _ = driver.execute(r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
        "#, vec![]).await;

        Ok(driver)
    }

    pub async fn login(&mut self, phone: &str, password: &str) -> Result<String, String> {
        info!("Attempting login for phone: {}", phone);

        let login_url = format!("{}/login", self.site.url.trim_end_matches('/'));
        self.driver.goto(&login_url).await.map_err(|e| format!("Failed to navigate to login: {}", e))?;
        sleep(Duration::from_secs(3)).await;

        let phone_script = format!(r#"
            (function() {{
                var inputs = document.querySelectorAll('input');
                for (var i = 0; i < inputs.length; i++) {{
                    var inp = inputs[i];
                    var type = (inp.type || '').toLowerCase();
                    var name = (inp.name || '').toLowerCase();
                    var placeholder = (inp.placeholder || '').toLowerCase();
                    if (type === 'tel' || name.includes('phone') || placeholder.includes('phone') || placeholder.includes('07')) {{
                        inp.value = '{}';
                        inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return 'filled';
                    }}
                }}
                return 'not_found';
            }})();
        "#, phone.replace("'", "\'"));

        self.driver.execute(&phone_script, vec![]).await
            .map_err(|e| format!("Phone fill failed: {}", e))?;
        sleep(Duration::from_millis(500)).await;

        let pass_script = format!(r#"
            (function() {{
                var inputs = document.querySelectorAll('input[type="password"]');
                if (inputs.length > 0) {{
                    inputs[0].value = '{}';
                    inputs[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
                    inputs[0].dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return 'filled';
                }}
                return 'not_found';
            }})();
        "#, password.replace("'", "\'"));

        self.driver.execute(&pass_script, vec![]).await
            .map_err(|e| format!("Password fill failed: {}", e))?;
        sleep(Duration::from_millis(500)).await;

        let click_script = r#"
            (function() {
                // Strategy 1: Find button inside a form near password input
                var passwordInputs = document.querySelectorAll('input[type="password"]');
                for (var i = 0; i < passwordInputs.length; i++) {
                    var form = passwordInputs[i].closest('form');
                    if (form) {
                        var submitBtn = form.querySelector('button[type="submit"], input[type="submit"]');
                        if (submitBtn) {
                            submitBtn.click();
                            return 'clicked_form_submit';
                        }
                        // Fallback: any button in the same form
                        var formButtons = form.querySelectorAll('button');
                        for (var j = 0; j < formButtons.length; j++) {
                            var btnText = (formButtons[j].textContent || '').toLowerCase().trim();
                            if (btnText.includes('login') || btnText.includes('log in') || btnText.includes('sign in')) {
                                formButtons[j].click();
                                return 'clicked_form_button';
                            }
                        }
                    }
                }

                // Strategy 2: Look for button with specific login classes/attributes
                var loginButtons = document.querySelectorAll(
                    'button[class*="login"], button[class*="submit"], ' +
                    'button[id*="login"], button[id*="submit"], ' +
                    'a[class*="login"], a[class*="submit"]'
                );
                for (var i = 0; i < loginButtons.length; i++) {
                    loginButtons[i].click();
                    return 'clicked_login_class';
                }

                // Strategy 3: Last resort - button with exact login text, but NOT in sidebar/nav
                var allButtons = document.querySelectorAll('button');
                for (var i = 0; i < allButtons.length; i++) {
                    var rect = allButtons[i].getBoundingClientRect();
                    var btnText = (allButtons[i].textContent || '').toLowerCase().trim();
                    // Skip buttons in sidebar/nav (usually at edges)
                    if (rect.left < 100 || rect.top < 50) continue;
                    // Skip if inside nav/sidebar/header elements
                    var parent = allButtons[i].closest('nav, aside, .sidebar, .nav, header, [role="navigation"]');
                    if (parent) continue;
                    if (btnText === 'login' || btnText === 'log in' || btnText === 'sign in') {
                        allButtons[i].click();
                        return 'clicked_exact_text';
                    }
                }

                return 'not_found';
            })();
        "#;

        self.driver.execute(click_script, vec![]).await
            .map_err(|e| format!("Login click failed: {}", e))?;
        sleep(Duration::from_secs(5)).await;

        let current_url = self.driver.current_url().await
            .map_err(|e| e.to_string())?
            .to_string();

        if current_url.contains("login") {
            return Err("Login failed - still on login page".to_string());
        }

        info!("Login successful, current URL: {}", current_url);
        Ok("Login successful".to_string())
    }

    pub async fn navigate_to_aviator(&mut self) -> Result<String, String> {
        info!("Navigating to Aviator...");
        self.driver.goto(&self.site.aviator_url).await
            .map_err(|e| format!("Navigate failed: {}", e))?;

        // Wait longer for Angular app to load and iframe to appear
        sleep(Duration::from_secs(6)).await;

        // Log how many iframes we found for debugging
        let iframes = self.driver.find_all(By::Tag("iframe")).await
            .map_err(|e| e.to_string())?;
        info!("Found {} iframes on Aviator page", iframes.len());

        for (i, iframe) in iframes.iter().enumerate() {
            if let Ok(src) = iframe.attr("src").await {
                info!("Iframe {} src: {:?}", i, src);
            }
            if let Ok(id) = iframe.attr("id").await {
                info!("Iframe {} id: {:?}", i, id);
            }
            if let Ok(name) = iframe.attr("name").await {
                info!("Iframe {} name: {:?}", i, name);
            }
        }

        self.enter_aviator_iframe().await?;

        Ok("Aviator loaded".to_string())
    }

    async fn enter_aviator_iframe(&mut self) -> Result<(), String> {
        for attempt in 0..10 {
            let iframes = self.driver.find_all(By::Tag("iframe")).await
                .map_err(|e| e.to_string())?;

            info!("Attempt {}: Found {} iframes", attempt + 1, iframes.len());

            // Strategy 1: Check iframe src for aviator/spribe/casino/game
            for iframe in &iframes {
                if let Ok(src) = iframe.attr("src").await {
                    if let Some(src_str) = src {
                        let src_lower = src_str.to_lowercase();
                        if src_lower.contains("aviator") 
                            || src_lower.contains("spribe")
                            || src_lower.contains("casino")
                            || src_lower.contains("game")
                            || src_lower.contains("play") {
                            info!("Entering iframe via src match: {}", src_str);
                            iframe.clone().enter_frame().await.map_err(|e| e.to_string())?;
                            sleep(Duration::from_millis(1500)).await;
                            return Ok(());
                        }
                    }
                }
            }

            // Strategy 2: Check iframe name/id attributes
            for iframe in &iframes {
                if let Ok(id) = iframe.attr("id").await {
                    if let Some(id_str) = id {
                        let id_lower = id_str.to_lowercase();
                        if id_lower.contains("aviator") || id_lower.contains("game") {
                            info!("Entering iframe via id match: {}", id_str);
                            iframe.clone().enter_frame().await.map_err(|e| e.to_string())?;
                            sleep(Duration::from_millis(1500)).await;
                            return Ok(());
                        }
                    }
                }
                if let Ok(name) = iframe.attr("name").await {
                    if let Some(name_str) = name {
                        let name_lower = name_str.to_lowercase();
                        if name_lower.contains("aviator") || name_lower.contains("game") {
                            info!("Entering iframe via name match: {}", name_str);
                            iframe.clone().enter_frame().await.map_err(|e| e.to_string())?;
                            sleep(Duration::from_millis(1500)).await;
                            return Ok(());
                        }
                    }
                }
            }

            // Strategy 3: Try entering each iframe and check if aviator elements exist inside
            for (idx, iframe) in iframes.iter().enumerate() {
                let _ = iframe.clone().enter_frame().await;
                sleep(Duration::from_millis(800)).await;

                let check_script = r#"
                    (function() {
                        var hasAviator = document.querySelector('.payout, .cash-out-switcher, app-bet-control, .aviator') !== null;
                        var hasGameCanvas = document.querySelector('canvas') !== null;
                        var hasBody = document.body !== null;
                        return JSON.stringify({
                            hasAviator: hasAviator,
                            hasGameCanvas: hasGameCanvas,
                            hasBody: hasBody,
                            url: window.location.href
                        });
                    })();
                "#;

                if let Ok(result) = self.driver.execute(check_script, vec![]).await {
                    let json = result.json();
                    info!("Iframe {} check result: {:?}", idx, json);
                    if let Some(has_aviator) = json.get("hasAviator").and_then(|v| v.as_bool()) {
                        if has_aviator {
                            info!("Entered aviator iframe via element detection");
                            return Ok(());
                        }
                    }
                }

                let _ = self.driver.enter_default_frame().await;
                sleep(Duration::from_millis(300)).await;
            }

            // Strategy 4: Last resort - try the last/biggest iframe
            if let Some(last_iframe) = iframes.last() {
                info!("Trying last iframe as fallback");
                let _ = last_iframe.clone().enter_frame().await;
                sleep(Duration::from_millis(1500)).await;
                return Ok(());
            }

            sleep(Duration::from_secs(1)).await;
        }

        Err("Could not enter Aviator iframe after 10 attempts".to_string())
    }

    async fn exit_iframe(&mut self) {
        let _ = self.driver.enter_default_frame().await;
    }

    pub async fn get_payouts(&mut self) -> Result<Vec<String>, String> {
        self.enter_aviator_iframe().await?;

        let payout_elements = self.driver.find_all(By::ClassName("payout")).await
            .map_err(|e| e.to_string())?;

        let mut payouts = Vec::new();
        for el in payout_elements {
            if let Ok(text) = el.text().await {
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() && trimmed.contains('x') {
                    payouts.push(trimmed);
                }
            }
        }

        self.exit_iframe().await;
        self.payouts = payouts.clone();

        let _ = self.event_tx.send(BrowserEvent::PayoutChanged(payouts.clone()));
        Ok(payouts)
    }

    pub async fn get_balance(&mut self) -> Result<BalanceResponse, String> {
        self.enter_aviator_iframe().await?;

        let selectors = [
            ".balance-amount",
            "[class*='balance']",
            "[class*='amount']",
            ".header-right span",
        ];

        for selector in &selectors {
            if let Ok(el) = self.driver.find(By::Css(*selector)).await {
                if let Ok(text) = el.text().await {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        self.exit_iframe().await;
                        self.current_balance = trimmed.to_string();
                        let _ = self.event_tx.send(BrowserEvent::BalanceChanged(trimmed.to_string()));
                        return Ok(BalanceResponse {
                            success: true,
                            balance: Some(trimmed.to_string()),
                            currency: "KES".to_string(),
                            source: format!("selector: {}", selector),
                            error: None,
                        });
                    }
                }
            }
        }

        self.exit_iframe().await;

        Ok(BalanceResponse {
            success: false,
            balance: None,
            currency: "KES".to_string(),
            source: "error".to_string(),
            error: Some("Balance element not found".to_string()),
        })
    }

    pub async fn get_game_state(&mut self) -> Result<GameState, String> {
        self.enter_aviator_iframe().await?;
        sleep(Duration::from_millis(500)).await;

        let first_panel = "app-bet-control:first-of-type";

        let btn_script = format!(r#"
            (function() {{
                var panel = document.querySelector('{}');
                if (!panel) return null;
                var btn = panel.querySelector('.buttons-block .btn');
                if (!btn) return null;
                return JSON.stringify({{
                    class: btn.className || '',
                    text: btn.textContent || '',
                    disabled: btn.disabled
                }});
            }})();
        "#, first_panel);

        let btn_result = self.driver.execute(&btn_script, vec![]).await
            .map_err(|e| e.to_string())?;

        let btn_json: serde_json::Value = btn_result.json().clone();
        let btn_class = btn_json.get("class").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let btn_text = btn_json.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let is_cashout = btn_class.contains("btn-warning") || btn_class.contains("cashout");
        let is_waiting = btn_text.to_lowercase().contains("waiting");

        let cashout_amount = if is_cashout {
            let amount_script = format!(r#"
                (function() {{
                    var panel = document.querySelector('{}');
                    if (!panel) return null;
                    var btn = panel.querySelector('.buttons-block .btn');
                    if (!btn) return null;
                    var span = btn.querySelector('label.amount span, .amount span, span');
                    return span ? span.textContent.trim() : null;
                }})();
            "#, first_panel);

            let amount_result = self.driver.execute(&amount_script, vec![]).await
                .map_err(|e| e.to_string())?;
            amount_result.json().as_str().map(|s| s.to_string())
        } else {
            None
        };

        self.exit_iframe().await;

        let state = GameState {
            phase: self.game_phase.clone(),
            bet_button_class: btn_class.clone(),
            bet_button_text: btn_text.clone(),
            cashout_amount: cashout_amount.clone(),
            input_value: None,
            is_waiting,
            is_cashout_available: is_cashout,
            current_balance: self.current_balance.clone(),
            payout_count: self.payouts.len(),
            latest_payout: self.payouts.last().cloned(),
            auto_play_active: self.auto_play_active,
            current_odd: self.strategy.as_ref().map(|s| s.tuples.get(s.current_index).map(|t| t.0).unwrap_or(0.0)).unwrap_or(0.0),
            current_multiplier: self.strategy.as_ref().map(|s| s.tuples.get(s.current_index).map(|t| t.1).unwrap_or(0.0)).unwrap_or(0.0),
            cashout_target: 0.0,
            strategy_index: self.game_history.current_strategy_index,
            total_wins: self.game_history.total_wins,
            total_losses: self.game_history.total_losses,
        };

        let _ = self.event_tx.send(BrowserEvent::GameStateChanged(state.clone()));
        Ok(state)
    }

    pub async fn place_auto_bet(&mut self, amount: &str, odd: &str, multiplier: &str) -> Result<String, String> {
        self.enter_aviator_iframe().await?;

        let first_panel = "app-bet-control:first-of-type";
        let cashout_val = {
            let o: f64 = odd.parse().unwrap_or(2.0);
            let m: f64 = multiplier.parse().unwrap_or(1.5);
            (o * m * 100.0).round() / 100.0
        };

        let amount_script = format!(r#"
            (function() {{
                var panel = document.querySelector('{}');
                if (!panel) return 'no_panel';
                var input = panel.querySelector('input[inputmode="decimal"], input[type="number"]');
                if (!input) return 'no_input';
                input.value = '{}';
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return 'set';
            }})();
        "#, first_panel, amount);

        self.driver.execute(&amount_script, vec![]).await
            .map_err(|e| e.to_string())?;
        sleep(Duration::from_millis(300)).await;

        let switch_script = format!(r#"
            (function() {{
                var panel = document.querySelector('{}');
                if (!panel) return 'no_panel';
                var switcher = panel.querySelector('.cash-out-switcher .input-switch');
                if (switcher && switcher.classList.contains('off')) {{
                    switcher.click();
                }}
                var cashoutInput = panel.querySelector('.cashout-spinner input');
                if (cashoutInput) {{
                    cashoutInput.value = '{}';
                    cashoutInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                }}
                return 'done';
            }})();
        "#, first_panel, cashout_val);

        self.driver.execute(&switch_script, vec![]).await
            .map_err(|e| e.to_string())?;
        sleep(Duration::from_millis(300)).await;

        let bet_script = format!(r#"
            (function() {{
                var panel = document.querySelector('{}');
                if (!panel) return 'no_panel';
                var btn = panel.querySelector('.buttons-block .btn.btn-success, .buttons-block .btn-success');
                if (btn) {{
                    btn.click();
                    return 'clicked';
                }}
                return 'not_found';
            }})();
        "#, first_panel);

        let _result = self.driver.execute(&bet_script, vec![]).await
            .map_err(|e| e.to_string())?;

        self.exit_iframe().await;

        let _ = self.event_tx.send(BrowserEvent::BetConfirmed);
        self.last_bet_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(format!("Auto-bet placed: amount={} cashout={}", amount, cashout_val))
    }

    pub async fn click_cashout(&mut self) -> Result<String, String> {
        self.enter_aviator_iframe().await?;

        let first_panel = "app-bet-control:first-of-type";
        let cashout_script = format!(r#"
            (function() {{
                var panel = document.querySelector('{}');
                if (!panel) return 'no_panel';
                var btn = panel.querySelector('.buttons-block .btn.btn-warning, .buttons-block .btn-warning, .buttons-block .btn.cashout');
                if (btn) {{
                    btn.click();
                    return 'clicked';
                }}
                return 'not_found';
            }})();
        "#, first_panel);

        let _result = self.driver.execute(&cashout_script, vec![]).await
            .map_err(|e| e.to_string())?;

        self.exit_iframe().await;

        let _ = self.event_tx.send(BrowserEvent::CashoutConfirmed(0.0));
        Ok("Cashout clicked".to_string())
    }

    pub async fn quit(&mut self) -> Result<(), String> {
        info!("Quitting browser session {}", self.session_id);
        let _ = self.driver.clone().quit().await;
        Ok(())
    }
}
