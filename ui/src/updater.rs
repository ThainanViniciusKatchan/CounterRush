use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use counterrush_core::{
    default_update_staging_dir, DownloadedUpdate, GameMonitorService, SingleInstanceGuard,
    UpdateCheckResult, UpdateError, UpdateInfo, UpdateService,
};
use slint::ComponentHandle;

use crate::{i18n, post_ui, MainWindow};

pub const STATE_IDLE: i32 = 0;
pub const STATE_CHECKING: i32 = 1;
pub const STATE_AVAILABLE: i32 = 2;
pub const STATE_DOWNLOADING: i32 = 3;
pub const STATE_READY_TO_RESTART: i32 = 4;
pub const STATE_ERROR: i32 = 5;

#[derive(Debug, Clone)]
enum UpdateErrorContext {
    Network,
    RateLimited(Option<u64>),
    NoCompatibleAsset(String),
    ChecksumMismatch,
    RestartFailed(String),
    Other(String),
}

impl UpdateErrorContext {
    fn from_error(err: &UpdateError) -> Self {
        match err {
            UpdateError::Network(_) => Self::Network,
            UpdateError::RateLimited { retry_after_secs } => Self::RateLimited(*retry_after_secs),
            UpdateError::NoCompatibleAsset { version } => {
                Self::NoCompatibleAsset(version.clone())
            }
            UpdateError::ChecksumMismatch { .. } => Self::ChecksumMismatch,
            UpdateError::RestartFailed(msg) => Self::RestartFailed(msg.clone()),
            other => Self::Other(other.to_string()),
        }
    }

    fn format(&self, lang: i18n::UiLanguage) -> String {
        match self {
            Self::Network => i18n::update_error_network(lang).to_string(),
            Self::RateLimited(secs) => i18n::update_error_rate_limited(lang, *secs),
            Self::NoCompatibleAsset(ver) => i18n::update_error_no_asset(lang, ver),
            Self::ChecksumMismatch => i18n::update_error_checksum(lang).to_string(),
            Self::RestartFailed(msg) => i18n::update_error_restart(lang, msg),
            Self::Other(msg) => i18n::update_error_generic(lang, msg),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureStage {
    Check,
    Download,
    Restart,
}

#[derive(Default)]
struct UpdaterRuntime {
    state: i32,
    available: Option<UpdateInfo>,
    downloaded: Option<DownloadedUpdate>,
    last_error: Option<UpdateErrorContext>,
    failure_stage: Option<FailureStage>,
    checked_up_to_date: bool,
    download_progress: f32,
}

#[derive(Clone)]
pub struct UpdaterController {
    ui: slint::Weak<MainWindow>,
    runtime: Arc<Mutex<UpdaterRuntime>>,
    busy: Arc<AtomicBool>,
    active_lang: Arc<AtomicU8>,
    single_instance_guard: Arc<Mutex<Option<SingleInstanceGuard>>>,
    game_monitor: Arc<GameMonitorService>,
    current_version: String,
}

impl UpdaterController {
    pub fn trigger_check(&self, manual: bool) {
        if self.busy.swap(true, Ordering::SeqCst) {
            return;
        }

        if manual {
            if let Ok(mut rt) = self.runtime.lock() {
                rt.state = STATE_CHECKING;
                rt.last_error = None;
                rt.failure_stage = None;
                rt.checked_up_to_date = false;
            }
            let lang = i18n::UiLanguage::load_atomic(&self.active_lang);
            let checking_detail = i18n::update_checking_detail(lang).to_string();
            let checking_status = i18n::update_checking_status(lang).to_string();
            post_ui(&self.ui, move |ui| {
                ui.set_update_state(STATE_CHECKING);
                ui.set_update_status_detail(checking_detail.into());
                ui.set_status_message(checking_status.into());
            });
        }

        let ctrl = self.clone();
        std::thread::spawn(move || {
            let service = UpdateService::default();
            let check_res = service.check_for_updates(&ctrl.current_version);
            let lang = i18n::UiLanguage::load_atomic(&ctrl.active_lang);

            match check_res {
                Ok(UpdateCheckResult::UpdateAvailable(info)) => {
                    let version = info.latest_version.clone();
                    if let Ok(mut rt) = ctrl.runtime.lock() {
                        rt.state = STATE_AVAILABLE;
                        rt.available = Some(info);
                        rt.downloaded = None;
                        rt.last_error = None;
                        rt.failure_stage = None;
                        rt.checked_up_to_date = false;
                        rt.download_progress = 0.0;
                    }
                    let detail = i18n::update_available_detail(lang, &version);
                    let status_msg = i18n::update_available_status(lang, &version);
                    let ver_slint = version.clone();
                    post_ui(&ctrl.ui, move |ui| {
                        ui.set_update_state(STATE_AVAILABLE);
                        ui.set_update_available_version(ver_slint.into());
                        ui.set_update_download_progress(0.0);
                        ui.set_update_status_detail(detail.into());
                        ui.set_show_dashboard_update_banner(true);
                        ui.set_status_message(status_msg.into());
                    });
                }
                Ok(UpdateCheckResult::UpToDate { current_version }) => {
                    if let Ok(mut rt) = ctrl.runtime.lock() {
                        rt.state = STATE_IDLE;
                        rt.available = None;
                        rt.downloaded = None;
                        rt.last_error = None;
                        rt.failure_stage = None;
                        rt.checked_up_to_date = true;
                        rt.download_progress = 0.0;
                    }
                    let detail = i18n::update_up_to_date_detail(lang, &current_version);
                    let status_msg = i18n::update_up_to_date_status(lang, &current_version);
                    post_ui(&ctrl.ui, move |ui| {
                        ui.set_update_state(STATE_IDLE);
                        ui.set_update_available_version("".into());
                        ui.set_update_download_progress(0.0);
                        ui.set_show_dashboard_update_banner(false);
                        ui.set_update_status_detail(detail.into());
                        if manual {
                            ui.set_status_message(status_msg.into());
                        }
                    });
                }
                Err(err) => {
                    let err_ctx = UpdateErrorContext::from_error(&err);
                    if manual {
                        if let Ok(mut rt) = ctrl.runtime.lock() {
                            rt.state = STATE_ERROR;
                            rt.last_error = Some(err_ctx.clone());
                            rt.failure_stage = Some(FailureStage::Check);
                            rt.checked_up_to_date = false;
                        }
                        let detail = err_ctx.format(lang);
                        let status_msg = detail.clone();
                        post_ui(&ctrl.ui, move |ui| {
                            ui.set_update_state(STATE_ERROR);
                            ui.set_show_dashboard_update_banner(false);
                            ui.set_update_status_detail(detail.into());
                            ui.set_status_message(status_msg.into());
                        });
                    } else if let Ok(mut rt) = ctrl.runtime.lock() {
                        rt.state = STATE_IDLE;
                        rt.last_error = None;
                        rt.failure_stage = None;
                    }
                }
            }

            ctrl.busy.store(false, Ordering::SeqCst);
        });
    }

    pub fn trigger_download(&self) {
        let info = {
            let Ok(mut rt) = self.runtime.lock() else {
                return;
            };
            let Some(info) = rt.available.clone() else {
                return;
            };
            if self.busy.swap(true, Ordering::SeqCst) {
                return;
            }
            rt.state = STATE_DOWNLOADING;
            rt.last_error = None;
            rt.failure_stage = None;
            rt.download_progress = 0.0;
            info
        };

        let lang = i18n::UiLanguage::load_atomic(&self.active_lang);
        let detail = i18n::update_downloading_detail(lang, &info.latest_version, Some(0));
        let status_msg = i18n::update_downloading_status(lang, &info.latest_version);
        let ver_slint = info.latest_version.clone();
        post_ui(&self.ui, move |ui| {
            ui.set_update_state(STATE_DOWNLOADING);
            ui.set_update_available_version(ver_slint.into());
            ui.set_update_download_progress(0.0);
            ui.set_show_dashboard_update_banner(true);
            ui.set_update_status_detail(detail.into());
            ui.set_status_message(status_msg.into());
        });

        let ctrl = self.clone();
        std::thread::spawn(move || {
            let service = UpdateService::default();
            let staging_dir = default_update_staging_dir();
            let last_emit = Arc::new(Mutex::new((Instant::now(), -1i32)));
            let ui_for_progress = ctrl.ui.clone();
            let runtime_for_progress = Arc::clone(&ctrl.runtime);
            let lang_for_progress = Arc::clone(&ctrl.active_lang);
            let version_for_progress = info.latest_version.clone();

            let progress_cb: Arc<counterrush_core::DownloadProgressFn> =
                Arc::new(move |downloaded, total| {
                    let (ratio, pct_opt) = match total {
                        Some(tot) if tot > 0 => {
                            let r = (downloaded as f32 / tot as f32).clamp(0.0, 1.0);
                            (r, Some((r * 100.0).round() as i32))
                        }
                        _ => (-1.0, None),
                    };

                    let should_emit = if let Ok(mut guard) = last_emit.lock() {
                        let (prev_time, prev_pct) = *guard;
                        let current_pct = pct_opt.unwrap_or(-1);
                        let elapsed_ok = prev_time.elapsed() >= Duration::from_millis(80);
                        let pct_changed = current_pct != prev_pct && current_pct >= 0;
                        if elapsed_ok || pct_changed || current_pct == 100 {
                            *guard = (Instant::now(), current_pct);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !should_emit {
                        return;
                    }

                    if let Ok(mut rt) = runtime_for_progress.lock() {
                        rt.download_progress = ratio;
                    }

                    let pct_u8 = pct_opt.map(|p| p.clamp(0, 100) as u8);
                    let lang_now = i18n::UiLanguage::load_atomic(&lang_for_progress);
                    let detail_now = i18n::update_downloading_detail(
                        lang_now,
                        &version_for_progress,
                        pct_u8,
                    );
                    post_ui(&ui_for_progress, move |ui| {
                        if ui.get_update_state() == STATE_DOWNLOADING {
                            ui.set_update_download_progress(ratio);
                            ui.set_update_status_detail(detail_now.into());
                        }
                    });
                });

            let result = service.download_and_prepare(&info, &staging_dir, progress_cb);
            let lang = i18n::UiLanguage::load_atomic(&ctrl.active_lang);

            match result {
                Ok(downloaded) => {
                    let version = downloaded.version.clone();
                    if let Ok(mut rt) = ctrl.runtime.lock() {
                        rt.state = STATE_READY_TO_RESTART;
                        rt.downloaded = Some(downloaded);
                        rt.last_error = None;
                        rt.failure_stage = None;
                        rt.download_progress = 1.0;
                    }
                    let detail = i18n::update_ready_detail(lang, &version);
                    let status_msg = i18n::update_ready_status(lang, &version);
                    let ver_slint = version;
                    post_ui(&ctrl.ui, move |ui| {
                        ui.set_update_state(STATE_READY_TO_RESTART);
                        ui.set_update_available_version(ver_slint.into());
                        ui.set_update_download_progress(1.0);
                        ui.set_show_dashboard_update_banner(true);
                        ui.set_update_status_detail(detail.into());
                        ui.set_status_message(status_msg.into());
                    });
                }
                Err(err) => {
                    let err_ctx = UpdateErrorContext::from_error(&err);
                    if let Ok(mut rt) = ctrl.runtime.lock() {
                        rt.state = STATE_ERROR;
                        rt.last_error = Some(err_ctx.clone());
                        rt.failure_stage = Some(FailureStage::Download);
                        rt.download_progress = 0.0;
                    }
                    let detail = err_ctx.format(lang);
                    let status_msg = detail.clone();
                    post_ui(&ctrl.ui, move |ui| {
                        ui.set_update_state(STATE_ERROR);
                        ui.set_update_download_progress(0.0);
                        ui.set_show_dashboard_update_banner(true);
                        ui.set_update_status_detail(detail.into());
                        ui.set_status_message(status_msg.into());
                    });
                }
            }

            ctrl.busy.store(false, Ordering::SeqCst);
        });
    }

    pub fn trigger_restart(&self) {
        let downloaded = {
            let Ok(rt) = self.runtime.lock() else {
                return;
            };
            let Some(downloaded) = rt.downloaded.clone() else {
                return;
            };
            if self.busy.swap(true, Ordering::SeqCst) {
                return;
            }
            downloaded
        };

        let lang = i18n::UiLanguage::load_atomic(&self.active_lang);
        let restarting_msg = i18n::update_restarting_status(lang).to_string();
        post_ui(&self.ui, move |ui| {
            ui.set_status_message(restarting_msg.into());
        });

        let ctrl = self.clone();
        std::thread::spawn(move || {
            let service = UpdateService::default();
            let guard_ref = Arc::clone(&ctrl.single_instance_guard);
            let monitor_ref = Arc::clone(&ctrl.game_monitor);

            let res = service.apply_and_restart(&downloaded, || {
                monitor_ref.stop();
                if let Ok(mut guard) = guard_ref.lock() {
                    let _ = guard.take();
                }
            });

            match res {
                Ok(()) => {
                    let _ = slint::invoke_from_event_loop(|| {
                        let _ = slint::quit_event_loop();
                    });
                    std::thread::sleep(Duration::from_millis(300));
                    std::process::exit(0);
                }
                Err(err) => {
                    let err_ctx = UpdateErrorContext::from_error(&err);
                    if let Ok(mut rt) = ctrl.runtime.lock() {
                        rt.state = STATE_ERROR;
                        rt.last_error = Some(err_ctx.clone());
                        rt.failure_stage = Some(FailureStage::Restart);
                    }
                    let lang_now = i18n::UiLanguage::load_atomic(&ctrl.active_lang);
                    let detail = err_ctx.format(lang_now);
                    let status_msg = detail.clone();
                    post_ui(&ctrl.ui, move |ui| {
                        ui.set_update_state(STATE_ERROR);
                        ui.set_show_dashboard_update_banner(true);
                        ui.set_update_status_detail(detail.into());
                        ui.set_status_message(status_msg.into());
                    });
                    ctrl.busy.store(false, Ordering::SeqCst);
                }
            }
        });
    }

    pub fn refresh_for_language(&self, ui: &MainWindow, lang: i18n::UiLanguage) {
        let Ok(rt) = self.runtime.lock() else {
            return;
        };
        let detail = match rt.state {
            STATE_CHECKING => i18n::update_checking_detail(lang).to_string(),
            STATE_AVAILABLE => rt
                .available
                .as_ref()
                .map(|i| i18n::update_available_detail(lang, &i.latest_version))
                .unwrap_or_default(),
            STATE_DOWNLOADING => {
                let ver = rt
                    .available
                    .as_ref()
                    .map(|i| i.latest_version.as_str())
                    .unwrap_or("");
                let pct = if rt.download_progress >= 0.0 {
                    Some((rt.download_progress * 100.0).round() as u8)
                } else {
                    None
                };
                i18n::update_downloading_detail(lang, ver, pct)
            }
            STATE_READY_TO_RESTART => rt
                .downloaded
                .as_ref()
                .map(|d| i18n::update_ready_detail(lang, &d.version))
                .unwrap_or_default(),
            STATE_ERROR => rt
                .last_error
                .as_ref()
                .map(|e| e.format(lang))
                .unwrap_or_default(),
            _ => {
                if rt.checked_up_to_date {
                    i18n::update_up_to_date_detail(lang, &self.current_version)
                } else {
                    String::new()
                }
            }
        };
        ui.set_update_status_detail(detail.into());
    }
}

pub fn wire_updater(
    main_window: &MainWindow,
    active_lang: Arc<AtomicU8>,
    single_instance_guard: Arc<Mutex<Option<SingleInstanceGuard>>>,
    game_monitor: Arc<GameMonitorService>,
) -> UpdaterController {
    let controller = UpdaterController {
        ui: main_window.as_weak(),
        runtime: Arc::new(Mutex::new(UpdaterRuntime::default())),
        busy: Arc::new(AtomicBool::new(false)),
        active_lang,
        single_instance_guard,
        game_monitor,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let ctrl_click = controller.clone();
    main_window.on_update_action_clicked(move || {
        let (state, stage) = {
            let Ok(rt) = ctrl_click.runtime.lock() else {
                return;
            };
            (rt.state, rt.failure_stage)
        };

        match state {
            STATE_IDLE => ctrl_click.trigger_check(true),
            STATE_AVAILABLE => ctrl_click.trigger_download(),
            STATE_READY_TO_RESTART => ctrl_click.trigger_restart(),
            STATE_ERROR => match stage {
                Some(FailureStage::Download) => ctrl_click.trigger_download(),
                Some(FailureStage::Restart) => ctrl_click.trigger_restart(),
                _ => ctrl_click.trigger_check(true),
            },
            _ => {}
        }
    });

    // Verificação automática silenciosa sempre que o software inicia.
    controller.trigger_check(false);

    controller
}
