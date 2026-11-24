use std::sync::atomic::{AtomicU8, Ordering};

use counterrush_core::PcCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLanguage {
    PtBr,
    EnUs,
    RuRu,
}

impl UiLanguage {
    pub fn from_code(code: &str) -> Self {
        match code.trim() {
            "en" | "en_US" | "en-US" | "en-EUA" => Self::EnUs,
            "ru" | "ru_RU" | "ru-RU" => Self::RuRu,
            _ => Self::PtBr,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::PtBr => "pt_BR",
            Self::EnUs => "en_US",
            Self::RuRu => "ru_RU",
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Self::PtBr => 0,
            Self::EnUs => 1,
            Self::RuRu => 2,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::EnUs,
            2 => Self::RuRu,
            _ => Self::PtBr,
        }
    }

    pub fn load_atomic(flag: &AtomicU8) -> Self {
        Self::from_u8(flag.load(Ordering::SeqCst))
    }

    pub fn store_atomic(self, flag: &AtomicU8) {
        flag.store(self.to_u8(), Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoStepStatusKind {
    Applying,
    Already,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreStepStatusKind {
    Checking,
    Reverting,
    AlreadyDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindSaveModeKind {
    NewFile,
    EnterName,
    UpdateCurrent,
    CreateNew,
    InvalidName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindUpdateVerb {
    NewFileSaved,
    BindReplaced,
    FileUpdated,
}

pub fn neutral_status_message(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "No active optimization profile. Create or detect a profile to begin.",
        UiLanguage::RuRu => {
            "Нет активного профиля оптимизации. Создайте или определите профиль для начала."
        }
        UiLanguage::PtBr => {
            "Nenhum perfil de otimização ativo. Crie ou detecte um perfil para começar."
        }
    }
}

pub fn ready_status_message(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => {
            "Ready to launch CS2 with 0.5ms polling rate and optimized CPU affinity."
        }
        UiLanguage::RuRu => {
            "Готово к запуску CS2 с частотой опроса 0.5 мс и оптимизированной привязкой ЦП."
        }
        UiLanguage::PtBr => {
            "Pronto para iniciar o CS2 com taxa de polling de 0.5ms e afinidade de CPU otimizada."
        }
    }
}

pub fn profile_required_status_message(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Create or select a profile to enable optimizations.",
        UiLanguage::RuRu => "Создайте или выберите профиль для включения оптимизаций.",
        UiLanguage::PtBr => "Crie ou selecione um perfil para ativar otimizações.",
    }
}

fn tweak_display_name(lang: UiLanguage, tweak_name: &str) -> &str {
    match (lang, tweak_name) {
        (UiLanguage::EnUs, "cpu_core_0") => "CPU Affinity (Core 0)",
        (UiLanguage::EnUs, "timer_resolution") => "Timer Resolution (0.5ms)",
        (UiLanguage::EnUs, "process_priority") => "Process Priority",
        (UiLanguage::EnUs, "empty_standby") => "RAM Cleanup (Standby)",
        (UiLanguage::EnUs, "gpu_acceleration") => "App GPU Acceleration",
        (UiLanguage::EnUs, "dpi_optimization") => "DPI & Window Optimization",
        (UiLanguage::EnUs, "memory_compression") => "Memory Compression",
        (UiLanguage::EnUs, "mmcss") => "MMCSS Scheduler",
        (UiLanguage::EnUs, "hpet") => "HPET Timer",
        (UiLanguage::EnUs, "game_bar") => "Xbox Game Bar & DVR",
        (UiLanguage::EnUs, "msi") => "MSI Interrupt Mode",
        (UiLanguage::EnUs, "power_saving") => "PCIe/USB Power Saving",
        (UiLanguage::EnUs, "render_queue") => "Render Queue",

        (UiLanguage::RuRu, "cpu_core_0") => "Привязка ЦП (Ядро 0)",
        (UiLanguage::RuRu, "timer_resolution") => "Разрешение таймера (0.5 мс)",
        (UiLanguage::RuRu, "process_priority") => "Приоритет процесса",
        (UiLanguage::RuRu, "empty_standby") => "Очистка ОЗУ (Standby)",
        (UiLanguage::RuRu, "gpu_acceleration") => "Аппаратное ускорение GPU",
        (UiLanguage::RuRu, "dpi_optimization") => "Оптимизация DPI и окна",
        (UiLanguage::RuRu, "memory_compression") => "Сжатие памяти",
        (UiLanguage::RuRu, "mmcss") => "Планировщик MMCSS",
        (UiLanguage::RuRu, "hpet") => "Таймер HPET",
        (UiLanguage::RuRu, "game_bar") => "Xbox Game Bar и DVR",
        (UiLanguage::RuRu, "msi") => "Режим прерываний MSI",
        (UiLanguage::RuRu, "power_saving") => "Энергосбережение PCIe/USB",
        (UiLanguage::RuRu, "render_queue") => "Очередь рендеринга",

        (_, "cpu_core_0") => "Afinidade de CPU (Núcleo 0)",
        (_, "timer_resolution") => "Temporizador (0.5ms)",
        (_, "process_priority") => "Prioridade de Processo",
        (_, "empty_standby") => "Limpeza de RAM (Standby)",
        (_, "gpu_acceleration") => "Aceleração de GPU em Apps",
        (_, "dpi_optimization") => "Otimização de DPI e Janela",
        (_, "memory_compression") => "Compactação de Memória",
        (_, "mmcss") => "Agendador MMCSS",
        (_, "hpet") => "Temporizador HPET",
        (_, "game_bar") => "Xbox Game Bar e DVR",
        (_, "msi") => "Modo de Interrupção MSI",
        (_, "power_saving") => "Economia de Energia PCIe/USB",
        (_, "render_queue") => "Fila de Renderização",
        (_, other) => other,
    }
}

pub fn tweak_toggled_message(lang: UiLanguage, tweak_name: &str, active: bool) -> String {
    let display_name = tweak_display_name(lang, tweak_name);
    match (lang, active) {
        (UiLanguage::EnUs, true) => format!("Module '{display_name}' enabled."),
        (UiLanguage::EnUs, false) => format!("Module '{display_name}' disabled."),
        (UiLanguage::RuRu, true) => format!("Модуль «{display_name}» включён."),
        (UiLanguage::RuRu, false) => format!("Модуль «{display_name}» отключён."),
        (UiLanguage::PtBr, true) => format!("Módulo '{display_name}' ativado."),
        (UiLanguage::PtBr, false) => format!("Módulo '{display_name}' desativado."),
    }
}

pub fn tweak_reverted_message(lang: UiLanguage, tweak_name: &str) -> String {
    let display_name = tweak_display_name(lang, tweak_name);
    match lang {
        UiLanguage::EnUs => format!("Module '{display_name}' disabled and reverted to Windows default."),
        UiLanguage::RuRu => format!("Модуль «{display_name}» отключён и возвращён к стандартным настройкам Windows."),
        UiLanguage::PtBr => format!("Módulo '{display_name}' desativado e restaurado ao padrão do Windows."),
    }
}

pub fn tweak_rollback_failed(
    lang: UiLanguage,
    tweak_name: &str,
    err: &dyn std::fmt::Display,
) -> String {
    let display_name = tweak_display_name(lang, tweak_name);
    match lang {
        UiLanguage::EnUs => format!("Could not revert '{display_name}': {err}"),
        UiLanguage::RuRu => format!("Не удалось откатить «{display_name}»: {err}"),
        UiLanguage::PtBr => format!("Não foi possível reverter '{display_name}': {err}"),
    }
}

pub fn restore_defaults_starting(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Restoring Windows default settings...",
        UiLanguage::RuRu => "Восстановление стандартных настроек Windows...",
        UiLanguage::PtBr => "Restaurando configurações padrão do Windows...",
    }
}

pub fn restore_modal_checking(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Checking applied optimizations in Windows...",
        UiLanguage::RuRu => "Проверка применённых оптимизаций в Windows...",
        UiLanguage::PtBr => "Verificando otimizações aplicadas no Windows...",
    }
}

pub fn restore_step_label(
    lang: UiLanguage,
    tweak_id: &str,
    status: RestoreStepStatusKind,
) -> &'static str {
    match (lang, tweak_id, status) {
        // PT-BR
        (UiLanguage::PtBr, "disable_gpu_acceleration", RestoreStepStatusKind::Checking) => {
            "Verificando aceleração de GPU em aplicativos..."
        }
        (UiLanguage::PtBr, "disable_gpu_acceleration", RestoreStepStatusKind::Reverting) => {
            "Reativando aceleração de GPU em aplicativos secundários..."
        }
        (UiLanguage::PtBr, "disable_gpu_acceleration", RestoreStepStatusKind::AlreadyDefault) => {
            "Aceleração de GPU em aplicativos já está no padrão."
        }
        (UiLanguage::PtBr, "apply_dpi_optimization", RestoreStepStatusKind::Checking) => {
            "Verificando configuração de DPI e tela cheia..."
        }
        (UiLanguage::PtBr, "apply_dpi_optimization", RestoreStepStatusKind::Reverting) => {
            "Removendo flags de compatibilidade DPI do CS2..."
        }
        (UiLanguage::PtBr, "apply_dpi_optimization", RestoreStepStatusKind::AlreadyDefault) => {
            "Configuração de DPI do CS2 já está no padrão."
        }
        (UiLanguage::PtBr, "disable_memory_compression", RestoreStepStatusKind::Checking) => {
            "Verificando compactação de memória do Windows..."
        }
        (UiLanguage::PtBr, "disable_memory_compression", RestoreStepStatusKind::Reverting) => {
            "Reativando compactação de memória do Windows..."
        }
        (UiLanguage::PtBr, "disable_memory_compression", RestoreStepStatusKind::AlreadyDefault) => {
            "Compactação de memória já está no padrão."
        }
        (UiLanguage::PtBr, "apply_mmcss", RestoreStepStatusKind::Checking) => {
            "Verificando agendador multimídia (MMCSS)..."
        }
        (UiLanguage::PtBr, "apply_mmcss", RestoreStepStatusKind::Reverting) => {
            "Restaurando agendador multimídia (MMCSS) ao padrão..."
        }
        (UiLanguage::PtBr, "apply_mmcss", RestoreStepStatusKind::AlreadyDefault) => {
            "Agendador multimídia (MMCSS) já está no padrão."
        }
        (UiLanguage::PtBr, "disable_hpet", RestoreStepStatusKind::Checking) => {
            "Verificando configuração HPET no BCD..."
        }
        (UiLanguage::PtBr, "disable_hpet", RestoreStepStatusKind::Reverting) => {
            "Restaurando HPET ao padrão de boot do Windows..."
        }
        (UiLanguage::PtBr, "disable_hpet", RestoreStepStatusKind::AlreadyDefault) => {
            "Configuração HPET já está no padrão."
        }
        (UiLanguage::PtBr, "disable_game_bar", RestoreStepStatusKind::Checking) => {
            "Verificando Xbox Game Bar e GameDVR..."
        }
        (UiLanguage::PtBr, "disable_game_bar", RestoreStepStatusKind::Reverting) => {
            "Restaurando Xbox Game Bar, Game Mode e GameDVR..."
        }
        (UiLanguage::PtBr, "disable_game_bar", RestoreStepStatusKind::AlreadyDefault) => {
            "Xbox Game Bar e GameDVR já estão no padrão."
        }
        (UiLanguage::PtBr, "enable_msi", RestoreStepStatusKind::Checking) => {
            "Verificando modo de interrupção MSI..."
        }
        (UiLanguage::PtBr, "enable_msi", RestoreStepStatusKind::Reverting) => {
            "Revertendo modo MSI para o padrão dos dispositivos..."
        }
        (UiLanguage::PtBr, "enable_msi", RestoreStepStatusKind::AlreadyDefault) => {
            "Modo de interrupção MSI já está no padrão."
        }
        (UiLanguage::PtBr, "disable_power_saving", RestoreStepStatusKind::Checking) => {
            "Verificando economia de energia PCIe e USB..."
        }
        (UiLanguage::PtBr, "disable_power_saving", RestoreStepStatusKind::Reverting) => {
            "Restaurando economia de energia PCIe e USB..."
        }
        (UiLanguage::PtBr, "disable_power_saving", RestoreStepStatusKind::AlreadyDefault) => {
            "Economia de energia PCIe e USB já está no padrão."
        }
        (UiLanguage::PtBr, "apply_render_queue", RestoreStepStatusKind::Checking) => {
            "Verificando fila de renderização (Render Queue)..."
        }
        (UiLanguage::PtBr, "apply_render_queue", RestoreStepStatusKind::Reverting) => {
            "Restaurando fila de renderização (Render Queue)..."
        }
        (UiLanguage::PtBr, "apply_render_queue", RestoreStepStatusKind::AlreadyDefault) => {
            "Fila de renderização já está no padrão."
        }
        (UiLanguage::PtBr, "apply_timer_resolution", RestoreStepStatusKind::Checking) => {
            "Verificando resolução do temporizador (0.5ms)..."
        }
        (UiLanguage::PtBr, "apply_timer_resolution", RestoreStepStatusKind::Reverting) => {
            "Restaurando resolução do temporizador ao padrão..."
        }
        (UiLanguage::PtBr, "apply_timer_resolution", RestoreStepStatusKind::AlreadyDefault) => {
            "Resolução do temporizador já está no padrão."
        }
        (UiLanguage::PtBr, "saving_profile", _) => {
            "Atualizando perfil e limpando otimizações..."
        }

        // EN-US
        (UiLanguage::EnUs, "disable_gpu_acceleration", RestoreStepStatusKind::Checking) => {
            "Checking app GPU acceleration..."
        }
        (UiLanguage::EnUs, "disable_gpu_acceleration", RestoreStepStatusKind::Reverting) => {
            "Re-enabling GPU acceleration in background apps..."
        }
        (UiLanguage::EnUs, "disable_gpu_acceleration", RestoreStepStatusKind::AlreadyDefault) => {
            "App GPU acceleration is already at default."
        }
        (UiLanguage::EnUs, "apply_dpi_optimization", RestoreStepStatusKind::Checking) => {
            "Checking DPI and fullscreen settings..."
        }
        (UiLanguage::EnUs, "apply_dpi_optimization", RestoreStepStatusKind::Reverting) => {
            "Removing CS2 DPI compatibility flags..."
        }
        (UiLanguage::EnUs, "apply_dpi_optimization", RestoreStepStatusKind::AlreadyDefault) => {
            "CS2 DPI configuration is already at default."
        }
        (UiLanguage::EnUs, "disable_memory_compression", RestoreStepStatusKind::Checking) => {
            "Checking Windows memory compression..."
        }
        (UiLanguage::EnUs, "disable_memory_compression", RestoreStepStatusKind::Reverting) => {
            "Re-enabling Windows memory compression..."
        }
        (UiLanguage::EnUs, "disable_memory_compression", RestoreStepStatusKind::AlreadyDefault) => {
            "Memory compression is already at default."
        }
        (UiLanguage::EnUs, "apply_mmcss", RestoreStepStatusKind::Checking) => {
            "Checking multimedia scheduler (MMCSS)..."
        }
        (UiLanguage::EnUs, "apply_mmcss", RestoreStepStatusKind::Reverting) => {
            "Restoring multimedia scheduler (MMCSS) to default..."
        }
        (UiLanguage::EnUs, "apply_mmcss", RestoreStepStatusKind::AlreadyDefault) => {
            "Multimedia scheduler (MMCSS) is already at default."
        }
        (UiLanguage::EnUs, "disable_hpet", RestoreStepStatusKind::Checking) => {
            "Checking HPET boot configuration..."
        }
        (UiLanguage::EnUs, "disable_hpet", RestoreStepStatusKind::Reverting) => {
            "Restoring HPET to Windows boot default..."
        }
        (UiLanguage::EnUs, "disable_hpet", RestoreStepStatusKind::AlreadyDefault) => {
            "HPET configuration is already at default."
        }
        (UiLanguage::EnUs, "disable_game_bar", RestoreStepStatusKind::Checking) => {
            "Checking Xbox Game Bar and GameDVR..."
        }
        (UiLanguage::EnUs, "disable_game_bar", RestoreStepStatusKind::Reverting) => {
            "Restoring Xbox Game Bar, Game Mode, and GameDVR..."
        }
        (UiLanguage::EnUs, "disable_game_bar", RestoreStepStatusKind::AlreadyDefault) => {
            "Xbox Game Bar and GameDVR are already at default."
        }
        (UiLanguage::EnUs, "enable_msi", RestoreStepStatusKind::Checking) => {
            "Checking MSI interrupt mode..."
        }
        (UiLanguage::EnUs, "enable_msi", RestoreStepStatusKind::Reverting) => {
            "Reverting MSI mode to device default..."
        }
        (UiLanguage::EnUs, "enable_msi", RestoreStepStatusKind::AlreadyDefault) => {
            "MSI interrupt mode is already at default."
        }
        (UiLanguage::EnUs, "disable_power_saving", RestoreStepStatusKind::Checking) => {
            "Checking PCIe and USB power saving..."
        }
        (UiLanguage::EnUs, "disable_power_saving", RestoreStepStatusKind::Reverting) => {
            "Restoring PCIe and USB power saving..."
        }
        (UiLanguage::EnUs, "disable_power_saving", RestoreStepStatusKind::AlreadyDefault) => {
            "PCIe and USB power saving is already at default."
        }
        (UiLanguage::EnUs, "apply_render_queue", RestoreStepStatusKind::Checking) => {
            "Checking Render Queue settings..."
        }
        (UiLanguage::EnUs, "apply_render_queue", RestoreStepStatusKind::Reverting) => {
            "Restoring Render Queue settings..."
        }
        (UiLanguage::EnUs, "apply_render_queue", RestoreStepStatusKind::AlreadyDefault) => {
            "Render Queue is already at default."
        }
        (UiLanguage::EnUs, "apply_timer_resolution", RestoreStepStatusKind::Checking) => {
            "Checking timer resolution (0.5ms)..."
        }
        (UiLanguage::EnUs, "apply_timer_resolution", RestoreStepStatusKind::Reverting) => {
            "Restoring timer resolution to Windows default..."
        }
        (UiLanguage::EnUs, "apply_timer_resolution", RestoreStepStatusKind::AlreadyDefault) => {
            "Timer resolution is already at default."
        }
        (UiLanguage::EnUs, "saving_profile", _) => {
            "Updating profile and clearing optimizations..."
        }

        // RU-RU
        (UiLanguage::RuRu, "disable_gpu_acceleration", RestoreStepStatusKind::Checking) => {
            "Проверка ускорения GPU в приложениях..."
        }
        (UiLanguage::RuRu, "disable_gpu_acceleration", RestoreStepStatusKind::Reverting) => {
            "Восстановление ускорения GPU в фоновых приложениях..."
        }
        (UiLanguage::RuRu, "disable_gpu_acceleration", RestoreStepStatusKind::AlreadyDefault) => {
            "Ускорение GPU в приложениях уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "apply_dpi_optimization", RestoreStepStatusKind::Checking) => {
            "Проверка параметров DPI и полного экрана..."
        }
        (UiLanguage::RuRu, "apply_dpi_optimization", RestoreStepStatusKind::Reverting) => {
            "Удаление флагов совместимости DPI для CS2..."
        }
        (UiLanguage::RuRu, "apply_dpi_optimization", RestoreStepStatusKind::AlreadyDefault) => {
            "Параметры DPI для CS2 уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "disable_memory_compression", RestoreStepStatusKind::Checking) => {
            "Проверка сжатия памяти Windows..."
        }
        (UiLanguage::RuRu, "disable_memory_compression", RestoreStepStatusKind::Reverting) => {
            "Повторное включение сжатия памяти Windows..."
        }
        (UiLanguage::RuRu, "disable_memory_compression", RestoreStepStatusKind::AlreadyDefault) => {
            "Сжатие памяти уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "apply_mmcss", RestoreStepStatusKind::Checking) => {
            "Проверка планировщика MMCSS..."
        }
        (UiLanguage::RuRu, "apply_mmcss", RestoreStepStatusKind::Reverting) => {
            "Восстановление планировщика MMCSS по умолчанию..."
        }
        (UiLanguage::RuRu, "apply_mmcss", RestoreStepStatusKind::AlreadyDefault) => {
            "Планировщик MMCSS уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "disable_hpet", RestoreStepStatusKind::Checking) => {
            "Проверка параметров HPET в BCD..."
        }
        (UiLanguage::RuRu, "disable_hpet", RestoreStepStatusKind::Reverting) => {
            "Восстановление HPET к параметрам загрузки Windows..."
        }
        (UiLanguage::RuRu, "disable_hpet", RestoreStepStatusKind::AlreadyDefault) => {
            "Параметры HPET уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "disable_game_bar", RestoreStepStatusKind::Checking) => {
            "Проверка Xbox Game Bar и GameDVR..."
        }
        (UiLanguage::RuRu, "disable_game_bar", RestoreStepStatusKind::Reverting) => {
            "Восстановление Xbox Game Bar, Game Mode и GameDVR..."
        }
        (UiLanguage::RuRu, "disable_game_bar", RestoreStepStatusKind::AlreadyDefault) => {
            "Xbox Game Bar и GameDVR уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "enable_msi", RestoreStepStatusKind::Checking) => {
            "Проверка режима прерываний MSI..."
        }
        (UiLanguage::RuRu, "enable_msi", RestoreStepStatusKind::Reverting) => {
            "Возврат режима прерываний MSI по умолчанию..."
        }
        (UiLanguage::RuRu, "enable_msi", RestoreStepStatusKind::AlreadyDefault) => {
            "Режим прерываний MSI уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "disable_power_saving", RestoreStepStatusKind::Checking) => {
            "Проверка энергосбережения PCIe и USB..."
        }
        (UiLanguage::RuRu, "disable_power_saving", RestoreStepStatusKind::Reverting) => {
            "Восстановление энергосбережения PCIe и USB..."
        }
        (UiLanguage::RuRu, "disable_power_saving", RestoreStepStatusKind::AlreadyDefault) => {
            "Энергосбережение PCIe и USB уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "apply_render_queue", RestoreStepStatusKind::Checking) => {
            "Проверка очереди рендеринга (Render Queue)..."
        }
        (UiLanguage::RuRu, "apply_render_queue", RestoreStepStatusKind::Reverting) => {
            "Восстановление очереди рендеринга (Render Queue)..."
        }
        (UiLanguage::RuRu, "apply_render_queue", RestoreStepStatusKind::AlreadyDefault) => {
            "Очередь рендеринга уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "apply_timer_resolution", RestoreStepStatusKind::Checking) => {
            "Проверка разрешения таймера (0.5 мс)..."
        }
        (UiLanguage::RuRu, "apply_timer_resolution", RestoreStepStatusKind::Reverting) => {
            "Восстановление разрешения таймера Windows..."
        }
        (UiLanguage::RuRu, "apply_timer_resolution", RestoreStepStatusKind::AlreadyDefault) => {
            "Разрешение таймера уже в исходном состоянии."
        }
        (UiLanguage::RuRu, "saving_profile", _) => {
            "Обновление профиля и сброс оптимизаций..."
        }

        _ => "",
    }
}

pub fn restore_defaults_done(
    lang: UiLanguage,
    reverted_count: usize,
    failed_count: usize,
) -> String {
    match (lang, reverted_count, failed_count) {
        (UiLanguage::EnUs, 0, 0) => {
            "Defaults restored: all optimizations disabled successfully.".to_string()
        }
        (UiLanguage::RuRu, 0, 0) => {
            "Настройки по умолчанию восстановлены: все оптимизации отключены.".to_string()
        }
        (UiLanguage::PtBr, 0, 0) => {
            "Padrão restaurado: todas as otimizações desativadas com sucesso.".to_string()
        }
        (UiLanguage::EnUs, rev, 0) => {
            format!("Defaults restored successfully ({rev} optimizations reverted in Windows).")
        }
        (UiLanguage::RuRu, rev, 0) => {
            format!("Стандартные настройки восстановлены ({rev} оптимизаций отменено в Windows).")
        }
        (UiLanguage::PtBr, rev, 0) => {
            format!("Padrão restaurado com sucesso ({rev} otimizações revertidas no Windows).")
        }
        (UiLanguage::EnUs, rev, fail) => {
            format!("Partial restore: {rev} reverted, {fail} could not be reverted (check permissions).")
        }
        (UiLanguage::RuRu, rev, fail) => {
            format!("Частичный откат: отменено {rev}, ошибок {fail} (проверьте права администратора).")
        }
        (UiLanguage::PtBr, rev, fail) => {
            format!("Restauração parcial: {rev} revertidas, {fail} não puderam ser revertidas (verifique permissões).")
        }
    }
}

pub fn auto_optimize_initial_step(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Starting calibration...",
        UiLanguage::RuRu => "Запуск калибровки...",
        UiLanguage::PtBr => "Iniciando calibração...",
    }
}

pub fn auto_optimize_detecting_hw(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Detecting hardware and system settings...",
        UiLanguage::RuRu => "Определение оборудования и параметров системы...",
        UiLanguage::PtBr => "Detectando hardware e configurações do sistema...",
    }
}

pub fn auto_optimize_analyzing_hw(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Analyzing hardware and existing optimizations...",
        UiLanguage::RuRu => "Анализ оборудования и существующих оптимизаций...",
        UiLanguage::PtBr => "Analisando hardware e otimizações existentes...",
    }
}

pub fn auto_optimize_err_generate(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Error generating optimization profile: {err}"),
        UiLanguage::RuRu => format!("Ошибка создания профиля оптимизации: {err}"),
        UiLanguage::PtBr => format!("Erro ao gerar perfil de otimização: {err}"),
    }
}

pub fn auto_optimize_err_save(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Error saving AutoDetect.json: {err}"),
        UiLanguage::RuRu => format!("Ошибка сохранения AutoDetect.json: {err}"),
        UiLanguage::PtBr => format!("Erro ao salvar AutoDetect.json: {err}"),
    }
}

pub fn auto_optimize_cancelled(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Automatic calibration cancelled by user.",
        UiLanguage::RuRu => "Автоматическая калибровка отменена пользователем.",
        UiLanguage::PtBr => "Calibração automática cancelada pelo usuário.",
    }
}

pub fn auto_optimize_cancelled_reverted(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Automatic calibration cancelled by user. Changes reverted.",
        UiLanguage::RuRu => {
            "Автоматическая калибровка отменена пользователем. Изменения отменены."
        }
        UiLanguage::PtBr => "Calibração automática cancelada pelo usuário. Alterações revertidas.",
    }
}

pub fn auto_optimize_step_label(
    lang: UiLanguage,
    tweak_id: &str,
    status: AutoStepStatusKind,
) -> &'static str {
    match (lang, tweak_id, status) {
        // PT-BR
        (UiLanguage::PtBr, "apply_timer_resolution", AutoStepStatusKind::Applying) => {
            "Otimizando resolução do temporizador (0.5ms)..."
        }
        (UiLanguage::PtBr, "apply_timer_resolution", AutoStepStatusKind::Already) => {
            "Resolução do temporizador (0.5ms) já configurada — preservada."
        }
        (UiLanguage::PtBr, "apply_timer_resolution", AutoStepStatusKind::Skipped) => {
            "Resolução do temporizador mantida padrão."
        }
        (UiLanguage::PtBr, "apply_mmcss", AutoStepStatusKind::Applying) => {
            "Configurando agendador multimídia e rede (MMCSS)..."
        }
        (UiLanguage::PtBr, "apply_mmcss", AutoStepStatusKind::Already) => {
            "Agendador multimídia (MMCSS) já otimizado — preservado."
        }
        (UiLanguage::PtBr, "apply_mmcss", AutoStepStatusKind::Skipped) => {
            "Agendador multimídia (MMCSS) ignorado."
        }
        (UiLanguage::PtBr, "apply_dpi_optimization", AutoStepStatusKind::Applying) => {
            "Ajustando DPI e compatibilidade de tela inteira CS2..."
        }
        (UiLanguage::PtBr, "apply_dpi_optimization", AutoStepStatusKind::Already) => {
            "Otimização de DPI já configurada — preservada."
        }
        (UiLanguage::PtBr, "apply_dpi_optimization", AutoStepStatusKind::Skipped) => {
            "Otimização de DPI ignorada."
        }
        (UiLanguage::PtBr, "disable_gpu_acceleration", AutoStepStatusKind::Applying) => {
            "Desativando aceleração de GPU em aplicativos secundários..."
        }
        (UiLanguage::PtBr, "disable_gpu_acceleration", AutoStepStatusKind::Already) => {
            "Aceleração de GPU em apps já desativada — preservada."
        }
        (UiLanguage::PtBr, "disable_gpu_acceleration", AutoStepStatusKind::Skipped) => {
            "Aceleração de GPU em apps mantida para perfil Potente."
        }
        (UiLanguage::PtBr, "disable_memory_compression", AutoStepStatusKind::Applying) => {
            "Desativando compactação de memória do Windows..."
        }
        (UiLanguage::PtBr, "disable_memory_compression", AutoStepStatusKind::Already) => {
            "Compactação de memória já desativada — preservada."
        }
        (UiLanguage::PtBr, "disable_memory_compression", AutoStepStatusKind::Skipped) => {
            "Compactação de memória mantida (RAM <= 16 GB)."
        }
        (UiLanguage::PtBr, "empty_standby_list", AutoStepStatusKind::Applying) => {
            "Liberando cache de memória RAM em Standby..."
        }
        (UiLanguage::PtBr, "empty_standby_list", AutoStepStatusKind::Already) => {
            "Standby List já verificada."
        }
        (UiLanguage::PtBr, "empty_standby_list", AutoStepStatusKind::Skipped) => {
            "Limpeza da Standby List ignorada."
        }
        (UiLanguage::PtBr, "disable_hpet", AutoStepStatusKind::Applying) => {
            "Desativando HPET no BCD do Windows..."
        }
        (UiLanguage::PtBr, "disable_hpet", AutoStepStatusKind::Already) => {
            "HPET já desativado no sistema — preservado."
        }
        (UiLanguage::PtBr, "disable_hpet", AutoStepStatusKind::Skipped) => {
            "Configuração HPET mantida."
        }
        (UiLanguage::PtBr, "disable_game_bar", AutoStepStatusKind::Applying) => {
            "Desativando Xbox Game Bar e capturas GameDVR..."
        }
        (UiLanguage::PtBr, "disable_game_bar", AutoStepStatusKind::Already) => {
            "Xbox Game Bar / GameDVR já desativados — preservados."
        }
        (UiLanguage::PtBr, "disable_game_bar", AutoStepStatusKind::Skipped) => {
            "Xbox Game Bar mantido."
        }
        (UiLanguage::PtBr, "enable_msi", AutoStepStatusKind::Applying) => {
            "Ativando Modo MSI para GPU e periféricos..."
        }
        (UiLanguage::PtBr, "enable_msi", AutoStepStatusKind::Already) => {
            "Modo MSI já habilitado nos dispositivos — preservado."
        }
        (UiLanguage::PtBr, "enable_msi", AutoStepStatusKind::Skipped) => "Modo MSI ignorado.",
        (UiLanguage::PtBr, "disable_power_saving", AutoStepStatusKind::Applying) => {
            "Desativando suspensão de energia PCIe e USB..."
        }
        (UiLanguage::PtBr, "disable_power_saving", AutoStepStatusKind::Already) => {
            "Economia de energia PCIe/USB já desativada — preservada."
        }
        (UiLanguage::PtBr, "disable_power_saving", AutoStepStatusKind::Skipped) => {
            "Economia de energia PCIe/USB mantida."
        }
        (UiLanguage::PtBr, "apply_render_queue", AutoStepStatusKind::Applying) => {
            "Configurando fila de renderização (Render Queue)..."
        }
        (UiLanguage::PtBr, "apply_render_queue", AutoStepStatusKind::Already) => {
            "Fila de renderização já configurada — preservada."
        }
        (UiLanguage::PtBr, "apply_render_queue", AutoStepStatusKind::Skipped) => {
            "Fila de renderização não aplicável ao adaptador atual."
        }

        // EN-US
        (UiLanguage::EnUs, "apply_timer_resolution", AutoStepStatusKind::Applying) => {
            "Optimizing timer resolution (0.5ms)..."
        }
        (UiLanguage::EnUs, "apply_timer_resolution", AutoStepStatusKind::Already) => {
            "Timer resolution (0.5ms) already configured — preserved."
        }
        (UiLanguage::EnUs, "apply_timer_resolution", AutoStepStatusKind::Skipped) => {
            "Timer resolution kept at default."
        }
        (UiLanguage::EnUs, "apply_mmcss", AutoStepStatusKind::Applying) => {
            "Configuring multimedia and network scheduler (MMCSS)..."
        }
        (UiLanguage::EnUs, "apply_mmcss", AutoStepStatusKind::Already) => {
            "Multimedia scheduler (MMCSS) already optimized — preserved."
        }
        (UiLanguage::EnUs, "apply_mmcss", AutoStepStatusKind::Skipped) => {
            "Multimedia scheduler (MMCSS) skipped."
        }
        (UiLanguage::EnUs, "apply_dpi_optimization", AutoStepStatusKind::Applying) => {
            "Adjusting DPI and CS2 fullscreen compatibility..."
        }
        (UiLanguage::EnUs, "apply_dpi_optimization", AutoStepStatusKind::Already) => {
            "DPI optimization already configured — preserved."
        }
        (UiLanguage::EnUs, "apply_dpi_optimization", AutoStepStatusKind::Skipped) => {
            "DPI optimization skipped."
        }
        (UiLanguage::EnUs, "disable_gpu_acceleration", AutoStepStatusKind::Applying) => {
            "Disabling GPU acceleration in background applications..."
        }
        (UiLanguage::EnUs, "disable_gpu_acceleration", AutoStepStatusKind::Already) => {
            "App GPU acceleration already disabled — preserved."
        }
        (UiLanguage::EnUs, "disable_gpu_acceleration", AutoStepStatusKind::Skipped) => {
            "App GPU acceleration kept for High-End profile."
        }
        (UiLanguage::EnUs, "disable_memory_compression", AutoStepStatusKind::Applying) => {
            "Disabling Windows memory compression..."
        }
        (UiLanguage::EnUs, "disable_memory_compression", AutoStepStatusKind::Already) => {
            "Memory compression already disabled — preserved."
        }
        (UiLanguage::EnUs, "disable_memory_compression", AutoStepStatusKind::Skipped) => {
            "Memory compression kept (RAM <= 16 GB)."
        }
        (UiLanguage::EnUs, "empty_standby_list", AutoStepStatusKind::Applying) => {
            "Flushing RAM Standby cache..."
        }
        (UiLanguage::EnUs, "empty_standby_list", AutoStepStatusKind::Already) => {
            "Standby List already checked."
        }
        (UiLanguage::EnUs, "empty_standby_list", AutoStepStatusKind::Skipped) => {
            "Standby List cleanup skipped."
        }
        (UiLanguage::EnUs, "disable_hpet", AutoStepStatusKind::Applying) => {
            "Disabling HPET in Windows BCD..."
        }
        (UiLanguage::EnUs, "disable_hpet", AutoStepStatusKind::Already) => {
            "HPET already disabled on system — preserved."
        }
        (UiLanguage::EnUs, "disable_hpet", AutoStepStatusKind::Skipped) => {
            "HPET setting kept."
        }
        (UiLanguage::EnUs, "disable_game_bar", AutoStepStatusKind::Applying) => {
            "Disabling Xbox Game Bar and GameDVR captures..."
        }
        (UiLanguage::EnUs, "disable_game_bar", AutoStepStatusKind::Already) => {
            "Xbox Game Bar / GameDVR already disabled — preserved."
        }
        (UiLanguage::EnUs, "disable_game_bar", AutoStepStatusKind::Skipped) => {
            "Xbox Game Bar kept."
        }
        (UiLanguage::EnUs, "enable_msi", AutoStepStatusKind::Applying) => {
            "Enabling MSI Mode for GPU and peripherals..."
        }
        (UiLanguage::EnUs, "enable_msi", AutoStepStatusKind::Already) => {
            "MSI Mode already enabled on devices — preserved."
        }
        (UiLanguage::EnUs, "enable_msi", AutoStepStatusKind::Skipped) => "MSI Mode skipped.",
        (UiLanguage::EnUs, "disable_power_saving", AutoStepStatusKind::Applying) => {
            "Disabling PCIe and USB power saving..."
        }
        (UiLanguage::EnUs, "disable_power_saving", AutoStepStatusKind::Already) => {
            "PCIe/USB power saving already disabled — preserved."
        }
        (UiLanguage::EnUs, "disable_power_saving", AutoStepStatusKind::Skipped) => {
            "PCIe/USB power saving kept."
        }
        (UiLanguage::EnUs, "apply_render_queue", AutoStepStatusKind::Applying) => {
            "Configuring Render Queue..."
        }
        (UiLanguage::EnUs, "apply_render_queue", AutoStepStatusKind::Already) => {
            "Render Queue already configured — preserved."
        }
        (UiLanguage::EnUs, "apply_render_queue", AutoStepStatusKind::Skipped) => {
            "Render Queue not applicable to current adapter."
        }

        // RU-RU
        (UiLanguage::RuRu, "apply_timer_resolution", AutoStepStatusKind::Applying) => {
            "Оптимизация разрешения таймера (0.5 мс)..."
        }
        (UiLanguage::RuRu, "apply_timer_resolution", AutoStepStatusKind::Already) => {
            "Разрешение таймера (0.5 мс) уже настроено — сохранено."
        }
        (UiLanguage::RuRu, "apply_timer_resolution", AutoStepStatusKind::Skipped) => {
            "Разрешение таймера оставлено по умолчанию."
        }
        (UiLanguage::RuRu, "apply_mmcss", AutoStepStatusKind::Applying) => {
            "Настройка планировщика мультимедиа и сети (MMCSS)..."
        }
        (UiLanguage::RuRu, "apply_mmcss", AutoStepStatusKind::Already) => {
            "Планировщик MMCSS уже оптимизирован — сохранён."
        }
        (UiLanguage::RuRu, "apply_mmcss", AutoStepStatusKind::Skipped) => {
            "Планировщик MMCSS пропущен."
        }
        (UiLanguage::RuRu, "apply_dpi_optimization", AutoStepStatusKind::Applying) => {
            "Настройка DPI и полноэкранной совместимости CS2..."
        }
        (UiLanguage::RuRu, "apply_dpi_optimization", AutoStepStatusKind::Already) => {
            "Оптимизация DPI уже настроена — сохранена."
        }
        (UiLanguage::RuRu, "apply_dpi_optimization", AutoStepStatusKind::Skipped) => {
            "Оптимизация DPI пропущена."
        }
        (UiLanguage::RuRu, "disable_gpu_acceleration", AutoStepStatusKind::Applying) => {
            "Отключение аппаратного ускорения GPU в фоновых приложениях..."
        }
        (UiLanguage::RuRu, "disable_gpu_acceleration", AutoStepStatusKind::Already) => {
            "Ускорение GPU в приложениях уже отключено — сохранено."
        }
        (UiLanguage::RuRu, "disable_gpu_acceleration", AutoStepStatusKind::Skipped) => {
            "Ускорение GPU в приложениях сохранено для мощного ПК."
        }
        (UiLanguage::RuRu, "disable_memory_compression", AutoStepStatusKind::Applying) => {
            "Отключение сжатия памяти Windows..."
        }
        (UiLanguage::RuRu, "disable_memory_compression", AutoStepStatusKind::Already) => {
            "Сжатие памяти уже отключено — сохранено."
        }
        (UiLanguage::RuRu, "disable_memory_compression", AutoStepStatusKind::Skipped) => {
            "Сжатие памяти сохранено (ОЗУ <= 16 ГБ)."
        }
        (UiLanguage::RuRu, "empty_standby_list", AutoStepStatusKind::Applying) => {
            "Очистка кэша оперативной памяти (Standby)..."
        }
        (UiLanguage::RuRu, "empty_standby_list", AutoStepStatusKind::Already) => {
            "Список Standby уже проверен."
        }
        (UiLanguage::RuRu, "empty_standby_list", AutoStepStatusKind::Skipped) => {
            "Очистка Standby List пропущена."
        }
        (UiLanguage::RuRu, "disable_hpet", AutoStepStatusKind::Applying) => {
            "Отключение HPET в конфигурации загрузки Windows (BCD)..."
        }
        (UiLanguage::RuRu, "disable_hpet", AutoStepStatusKind::Already) => {
            "HPET уже отключён в системе — сохранён."
        }
        (UiLanguage::RuRu, "disable_hpet", AutoStepStatusKind::Skipped) => {
            "Настройка HPET сохранена."
        }
        (UiLanguage::RuRu, "disable_game_bar", AutoStepStatusKind::Applying) => {
            "Отключение Xbox Game Bar и записи GameDVR..."
        }
        (UiLanguage::RuRu, "disable_game_bar", AutoStepStatusKind::Already) => {
            "Xbox Game Bar / GameDVR уже отключены — сохранены."
        }
        (UiLanguage::RuRu, "disable_game_bar", AutoStepStatusKind::Skipped) => {
            "Xbox Game Bar сохранён."
        }
        (UiLanguage::RuRu, "enable_msi", AutoStepStatusKind::Applying) => {
            "Включение режима MSI для видеокарты и периферии..."
        }
        (UiLanguage::RuRu, "enable_msi", AutoStepStatusKind::Already) => {
            "Режим MSI уже включён на устройствах — сохранён."
        }
        (UiLanguage::RuRu, "enable_msi", AutoStepStatusKind::Skipped) => {
            "Режим MSI пропущен."
        }
        (UiLanguage::RuRu, "disable_power_saving", AutoStepStatusKind::Applying) => {
            "Отключение энергосбережения PCIe и USB..."
        }
        (UiLanguage::RuRu, "disable_power_saving", AutoStepStatusKind::Already) => {
            "Энергосбережение PCIe/USB уже отключено — сохранено."
        }
        (UiLanguage::RuRu, "disable_power_saving", AutoStepStatusKind::Skipped) => {
            "Энергосбережение PCIe/USB сохранено."
        }
        (UiLanguage::RuRu, "apply_render_queue", AutoStepStatusKind::Applying) => {
            "Настройка очереди рендеринга (Render Queue)..."
        }
        (UiLanguage::RuRu, "apply_render_queue", AutoStepStatusKind::Already) => {
            "Очередь рендеринга уже настроена — сохранена."
        }
        (UiLanguage::RuRu, "apply_render_queue", AutoStepStatusKind::Skipped) => {
            "Очередь рендеринга не применима к текущему адаптеру."
        }

        _ => "",
    }
}

pub fn auto_optimize_completed_step(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "System optimized successfully!",
        UiLanguage::RuRu => "Система успешно оптимизирована!",
        UiLanguage::PtBr => "Sistema otimizado com sucesso!",
    }
}

pub fn category_display_name(lang: UiLanguage, category: PcCategory) -> &'static str {
    match (lang, category) {
        (UiLanguage::EnUs, PcCategory::Economico) => "Budget",
        (UiLanguage::EnUs, PcCategory::Intermediario) => "Mid-Range",
        (UiLanguage::EnUs, PcCategory::Potente) => "High-End",
        (UiLanguage::RuRu, PcCategory::Economico) => "Бюджетный",
        (UiLanguage::RuRu, PcCategory::Intermediario) => "Средний",
        (UiLanguage::RuRu, PcCategory::Potente) => "Мощный",
        (UiLanguage::PtBr, PcCategory::Economico) => "Econômico",
        (UiLanguage::PtBr, PcCategory::Intermediario) => "Intermediário",
        (UiLanguage::PtBr, PcCategory::Potente) => "Potente",
    }
}

pub fn auto_optimize_completed_status(
    lang: UiLanguage,
    category: PcCategory,
    gpu_model: &str,
    applied_count: usize,
    already_count: usize,
    blocked_count: usize,
) -> String {
    let cat_label = category_display_name(lang, category);
    match (lang, blocked_count == 0) {
        (UiLanguage::EnUs, true) => format!(
            "Hardware detected ({cat_label} • {gpu_model}). AutoDetect saved ({applied_count} applied now, {already_count} already optimized)."
        ),
        (UiLanguage::EnUs, false) => format!(
            "Hardware detected ({cat_label} • {gpu_model}). AutoDetect saved ({applied_count} applied now, {already_count} already optimized, {blocked_count} not applicable)."
        ),
        (UiLanguage::RuRu, true) => format!(
            "Оборудование определено ({cat_label} • {gpu_model}). AutoDetect сохранён (применено: {applied_count}, уже оптимизировано: {already_count})."
        ),
        (UiLanguage::RuRu, false) => format!(
            "Оборудование определено ({cat_label} • {gpu_model}). AutoDetect сохранён (применено: {applied_count}, уже оптимизировано: {already_count}, неприменимо: {blocked_count})."
        ),
        (UiLanguage::PtBr, true) => format!(
            "Hardware detectado ({cat_label} • {gpu_model}). AutoDetect salvo ({applied_count} aplicadas agora, {already_count} já estavam otimizadas)."
        ),
        (UiLanguage::PtBr, false) => format!(
            "Hardware detectado ({cat_label} • {gpu_model}). AutoDetect salvo ({applied_count} aplicadas agora, {already_count} já otimizadas, {blocked_count} não aplicáveis ao hardware)."
        ),
    }
}

pub fn configured_checking_profile(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Checking profile and hardware compatibility...",
        UiLanguage::RuRu => "Проверка профиля и совместимости оборудования...",
        UiLanguage::PtBr => "Verificando perfil e compatibilidade do hardware...",
    }
}

pub fn configured_step_applying(lang: UiLanguage, tweak_id: &str) -> &'static str {
    if tweak_id == "disable_memory_compression" && lang == UiLanguage::PtBr {
        return "Calibrando compactação de memória do Windows...";
    }
    if tweak_id == "disable_hpet" && lang == UiLanguage::PtBr {
        return "Desativando HPET (temporizador de alta precisão)...";
    }
    auto_optimize_step_label(lang, tweak_id, AutoStepStatusKind::Applying)
}

pub fn configured_step_already(lang: UiLanguage, tweak_id: &str) -> &'static str {
    auto_optimize_step_label(lang, tweak_id, AutoStepStatusKind::Already)
}

pub fn launch_initial_step(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Starting optimizations for CS2...",
        UiLanguage::RuRu => "Запуск оптимизаций для CS2...",
        UiLanguage::PtBr => "Iniciando otimizações para o CS2...",
    }
}

pub fn launch_status_starting(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Optimizing system and launching CS2...",
        UiLanguage::RuRu => "Оптимизация системы и запуск CS2...",
        UiLanguage::PtBr => "Otimizando sistema e iniciando CS2...",
    }
}

pub fn launch_flow_initial_step(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Starting calibration and applying optimizations...",
        UiLanguage::RuRu => "Запуск калибровки и применения оптимизаций...",
        UiLanguage::PtBr => "Iniciando calibração e aplicação de otimizações...",
    }
}

pub fn launch_cancelled_reverted(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Launch cancelled by user. Changes reverted.",
        UiLanguage::RuRu => "Запуск отменён пользователем. Изменения отменены.",
        UiLanguage::PtBr => "Inicialização cancelada pelo usuário. Alterações revertidas.",
    }
}

pub fn launch_config_error(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Configuration error: {err}"),
        UiLanguage::RuRu => format!("Ошибка конфигурации: {err}"),
        UiLanguage::PtBr => format!("Erro na configuração: {err}"),
    }
}

pub fn launch_download_title(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "GAME DOWNLOADING OR UPDATING",
        UiLanguage::RuRu => "ИГРА В ПРОЦЕССЕ ЗАГРУЗКИ ИЛИ ОБНОВЛЕНИЯ",
        UiLanguage::PtBr => "JOGO EM PROCESSO DE DOWNLOAD OU ATUALIZAÇÃO",
    }
}

pub fn launch_download_subtitle(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Waiting for completion to continue",
        UiLanguage::RuRu => "Ожидание завершения для продолжения",
        UiLanguage::PtBr => "Aguardando finalização para continuar",
    }
}

pub fn launch_starting_steam(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Launching Counter-Strike 2 via Steam...",
        UiLanguage::RuRu => "Запуск Counter-Strike 2 через Steam...",
        UiLanguage::PtBr => "Inicializando Counter-Strike 2 pela Steam...",
    }
}

pub fn launch_failed(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Failed to launch CS2: {err}"),
        UiLanguage::RuRu => format!("Не удалось запустить CS2: {err}"),
        UiLanguage::PtBr => format!("Falha ao iniciar CS2: {err}"),
    }
}

pub fn launch_success(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "CS2 OPTIMIZED SUCCESSFULLY",
        UiLanguage::RuRu => "CS2 УСПЕШНО ОПТИМИЗИРОВАН",
        UiLanguage::PtBr => "CS2 OTIMIZADO COM SUCESSO",
    }
}

pub fn steam_auto_detected_status(lang: UiLanguage, pid: u32) -> String {
    match lang {
        UiLanguage::EnUs => format!("CS2 detected via Steam (PID: {pid}). Applying optimizations..."),
        UiLanguage::RuRu => format!("CS2 обнаружен через Steam (PID: {pid}). Применение оптимизаций..."),
        UiLanguage::PtBr => format!("CS2 detectado via Steam (PID: {pid}). Aplicando otimizações..."),
    }
}

pub fn steam_auto_detected_step(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "CS2 detected via Steam. Starting optimizations...",
        UiLanguage::RuRu => "CS2 обнаружен через Steam. Запуск оптимизаций...",
        UiLanguage::PtBr => "CS2 detectado via Steam. Iniciando otimizações...",
    }
}

pub fn steam_auto_no_profile(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "No optimization profile selected for Steam Auto-Optimization.",
        UiLanguage::RuRu => "Не выбран профиль оптимизации для автооптимизации Steam.",
        UiLanguage::PtBr => "Nenhum perfil de otimização selecionado para auto-otimização Steam.",
    }
}

pub fn steam_auto_cancelled(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Steam Auto-Optimization cancelled by user. Changes reverted.",
        UiLanguage::RuRu => "Автооптимизация Steam отменена пользователем. Изменения отменены.",
        UiLanguage::PtBr => "Auto-otimização Steam cancelada pelo usuário. Alterações revertidas.",
    }
}

pub fn steam_auto_applying_process(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Applying priority and affinity to CS2 process...",
        UiLanguage::RuRu => "Применение приоритета и привязки к процессу CS2...",
        UiLanguage::PtBr => "Aplicando prioridade e afinidade no processo CS2...",
    }
}

pub fn steam_auto_done_status(lang: UiLanguage, profile_label: &str) -> String {
    match lang {
        UiLanguage::EnUs => {
            format!("CS2 optimized with profile '{profile_label}'! Waiting for game focus...")
        }
        UiLanguage::RuRu => {
            format!("CS2 оптимизирован с профилем '{profile_label}'! Ожидание фокуса игры...")
        }
        UiLanguage::PtBr => {
            format!("CS2 otimizado com o perfil '{profile_label}'! Aguardando foco do jogo...")
        }
    }
}

pub fn steam_auto_closed_status(lang: UiLanguage, pid: u32) -> String {
    match lang {
        UiLanguage::EnUs => format!("CS2 closed (PID: {pid}). Monitoring active."),
        UiLanguage::RuRu => format!("CS2 закрыт (PID: {pid}). Мониторинг активен."),
        UiLanguage::PtBr => format!("CS2 encerrado (PID: {pid}). Monitoramento ativo."),
    }
}

pub fn preset_selected_ok(lang: UiLanguage, preset_name: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Profile '{preset_name}' selected and loaded successfully."),
        UiLanguage::RuRu => format!("Профиль «{preset_name}» выбран и успешно загружен."),
        UiLanguage::PtBr => format!("Perfil '{preset_name}' selecionado e carregado com sucesso."),
    }
}

pub fn preset_load_err(
    lang: UiLanguage,
    preset_name: &str,
    err: &dyn std::fmt::Display,
) -> String {
    match lang {
        UiLanguage::EnUs => format!("Error loading profile '{preset_name}': {err}"),
        UiLanguage::RuRu => format!("Ошибка загрузки профиля «{preset_name}»: {err}"),
        UiLanguage::PtBr => format!("Erro ao carregar perfil '{preset_name}': {err}"),
    }
}

pub fn profile_validation_error(lang: UiLanguage, raw_err: &str) -> String {
    if lang == UiLanguage::PtBr {
        return raw_err.to_string();
    }
    if raw_err.contains("não pode ser vazio") {
        return match lang {
            UiLanguage::EnUs => "Profile name cannot be empty.".to_string(),
            UiLanguage::RuRu => "Имя профиля не может быть пустым.".to_string(),
            UiLanguage::PtBr => raw_err.to_string(),
        };
    }
    if raw_err.contains("caracteres proibidos") {
        return match lang {
            UiLanguage::EnUs => "The name contains characters prohibited by the operating system.".to_string(),
            UiLanguage::RuRu => "Имя содержит символы, запрещённые операционной системой.".to_string(),
            UiLanguage::PtBr => raw_err.to_string(),
        };
    }
    if raw_err.contains("reservado") {
        return match lang {
            UiLanguage::EnUs => "File name reserved by the operating system.".to_string(),
            UiLanguage::RuRu => "Имя файла зарезервировано операционной системой.".to_string(),
            UiLanguage::PtBr => raw_err.to_string(),
        };
    }
    raw_err.to_string()
}

pub fn profile_created_ok(lang: UiLanguage, name: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Profile '{name}' created and selected successfully!"),
        UiLanguage::RuRu => format!("Профиль «{name}» успешно создан и выбран!"),
        UiLanguage::PtBr => format!("Perfil '{name}' criado e selecionado com sucesso!"),
    }
}

pub fn profile_save_err(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Failed to save file: {err}"),
        UiLanguage::RuRu => format!("Не удалось сохранить файл: {err}"),
        UiLanguage::PtBr => format!("Falha ao salvar arquivo: {err}"),
    }
}

pub fn launch_args_saved(lang: UiLanguage, file_stem: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Launch parameters saved to profile '{file_stem}'."),
        UiLanguage::RuRu => format!("Параметры запуска сохранены в профиль «{file_stem}»."),
        UiLanguage::PtBr => {
            format!("Parâmetros de inicialização salvos no perfil '{file_stem}'.")
        }
    }
}

pub fn launch_args_save_err(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Failed to save parameters: {err}"),
        UiLanguage::RuRu => format!("Не удалось сохранить параметры: {err}"),
        UiLanguage::PtBr => format!("Falha ao salvar parâmetros: {err}"),
    }
}

pub fn profile_deleted_empty(lang: UiLanguage, name: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Profile '{name}' deleted successfully. No profiles remaining."),
        UiLanguage::RuRu => format!("Профиль «{name}» удалён. Других профилей нет."),
        UiLanguage::PtBr => {
            format!("Perfil '{name}' excluído com sucesso. Nenhum perfil restante.")
        }
    }
}

pub fn profile_deleted_switched(lang: UiLanguage, name: &str, next: &str) -> String {
    match lang {
        UiLanguage::EnUs => {
            format!("Profile '{name}' deleted. Active profile switched to '{next}'.")
        }
        UiLanguage::RuRu => {
            format!("Профиль «{name}» удалён. Активный профиль переключён на «{next}».")
        }
        UiLanguage::PtBr => {
            format!("Perfil '{name}' excluído. Perfil ativo alternado para '{next}'.")
        }
    }
}

pub fn profile_deleted_ok(lang: UiLanguage, name: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Profile '{name}' deleted successfully."),
        UiLanguage::RuRu => format!("Профиль «{name}» успешно удалён."),
        UiLanguage::PtBr => format!("Perfil '{name}' excluído com sucesso."),
    }
}

pub fn profile_delete_err(
    lang: UiLanguage,
    name: &str,
    err: &dyn std::fmt::Display,
) -> String {
    match lang {
        UiLanguage::EnUs => format!("Error deleting profile '{name}': {err}"),
        UiLanguage::RuRu => format!("Ошибка удаления профиля «{name}»: {err}"),
        UiLanguage::PtBr => format!("Erro ao excluir perfil '{name}': {err}"),
    }
}

pub fn card_action_status(lang: UiLanguage, module_id: &str) -> &'static str {
    match (lang, module_id) {
        (UiLanguage::EnUs, "custom_optimization") => "Opening module: Custom Optimization",
        (UiLanguage::EnUs, "pro_cfg") => "Opening module: Pro .cfg Catalog",
        (UiLanguage::EnUs, "binds") => "Opening module: Bind Creator & Manager",
        (UiLanguage::EnUs, "workshop_maps") => "Opening module: CS2 Workshop Maps",
        (UiLanguage::EnUs, "benchmark") => "Opening module: Benchmark & Performance Analysis",
        (UiLanguage::EnUs, "about") => "Opening module: About CounterRush",
        (UiLanguage::EnUs, _) => "Module selected",

        (UiLanguage::RuRu, "custom_optimization") => "Открытие модуля: Пользовательская оптимизация",
        (UiLanguage::RuRu, "pro_cfg") => "Открытие модуля: Каталог про .cfg",
        (UiLanguage::RuRu, "binds") => "Открытие модуля: Генератор и менеджер биндов",
        (UiLanguage::RuRu, "workshop_maps") => "Открытие модуля: Карты мастерской CS2",
        (UiLanguage::RuRu, "benchmark") => "Открытие модуля: Бенчмарк и анализ производительности",
        (UiLanguage::RuRu, "about") => "Открытие модуля: О CounterRush",
        (UiLanguage::RuRu, _) => "Модуль выбран",

        (_, "custom_optimization") => "Acessando módulo: Otimização Personalizada",
        (_, "pro_cfg") => "Acessando módulo: Catálogo de .cfg Profissionais",
        (_, "binds") => "Acessando módulo: Criador e Gerenciador de Binds",
        (_, "workshop_maps") => "Acessando módulo: Mapas da Oficina CS2",
        (_, "benchmark") => "Acessando módulo: Benchmark e Análise de Desempenho",
        (_, "about") => "Acessando módulo: Sobre o CounterRush",
        _ => "Módulo selecionado",
    }
}

pub fn language_changed_status(lang: UiLanguage, code: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Language changed to '{code}'."),
        UiLanguage::RuRu => format!("Язык изменен на '{code}'."),
        UiLanguage::PtBr => format!("Idioma alterado para '{code}'."),
    }
}

pub fn share_project_text(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "CounterRush - Performance and Latency Optimizer for Counter-Strike 2: https://github.com/ThainanViniciusKatchan/CounterRush",
        UiLanguage::RuRu => "CounterRush — оптимизатор производительности и задержки для Counter-Strike 2: https://github.com/ThainanViniciusKatchan/CounterRush",
        UiLanguage::PtBr => "CounterRush - Otimizador de Desempenho e Latência para Counter-Strike 2: https://github.com/ThainanViniciusKatchan/CounterRush",
    }
}

pub fn clipboard_err_prefix(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Error accessing clipboard",
        UiLanguage::RuRu => "Ошибка доступа к буферу обмена",
        UiLanguage::PtBr => "Erro ao acessar clipboard",
    }
}

pub fn clipboard_copied_ok(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Official CounterRush link copied to clipboard!",
        UiLanguage::RuRu => "Официальная ссылка CounterRush скопирована в буфер обмена!",
        UiLanguage::PtBr => "Link oficial do CounterRush copiado para a área de transferência!",
    }
}

pub fn open_url_err_prefix(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Error opening link",
        UiLanguage::RuRu => "Ошибка при открытии ссылки",
        UiLanguage::PtBr => "Erro ao abrir link",
    }
}

pub fn startup_toggled_status(
    lang: UiLanguage,
    enabled: bool,
    prev_steam_auto: bool,
) -> &'static str {
    match (lang, enabled, prev_steam_auto) {
        (UiLanguage::EnUs, true, _) => "Start with Windows enabled (Highest privileges).",
        (UiLanguage::EnUs, false, true) => {
            "Start with Windows and Steam Auto-Optimization disabled."
        }
        (UiLanguage::EnUs, false, false) => "Start with Windows disabled.",
        (UiLanguage::RuRu, true, _) => "Автозапуск с Windows включен (Наивысшие права).",
        (UiLanguage::RuRu, false, true) => {
            "Автозапуск с Windows и автооптимизация Steam отключены."
        }
        (UiLanguage::RuRu, false, false) => "Автозапуск с Windows отключен.",
        (UiLanguage::PtBr, true, _) => {
            "Inicialização com o Windows ativada com privilégios máximos."
        }
        (UiLanguage::PtBr, false, true) => {
            "Inicialização com o Windows e Auto-Otimização Steam desativadas."
        }
        (UiLanguage::PtBr, false, false) => "Inicialização com o Windows desativada.",
    }
}

pub fn startup_toggled_err_prefix(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Failed to configure Windows startup",
        UiLanguage::RuRu => "Ошибка настройки автозапуска Windows",
        UiLanguage::PtBr => "Falha ao configurar inicialização com o Windows",
    }
}

pub fn steam_auto_toggled_status(lang: UiLanguage, enabled: bool) -> &'static str {
    match (lang, enabled) {
        (UiLanguage::EnUs, true) => "Steam Auto-Optimization and Startup enabled.",
        (UiLanguage::EnUs, false) => "Steam Auto-Optimization disabled.",
        (UiLanguage::RuRu, true) => "Автооптимизация Steam и автозапуск включены.",
        (UiLanguage::RuRu, false) => "Автооптимизация Steam отключена.",
        (UiLanguage::PtBr, true) => {
            "Auto-Otimização Steam e Inicialização com o Windows ativadas."
        }
        (UiLanguage::PtBr, false) => "Auto-Otimização Steam desativada.",
    }
}

pub fn steam_auto_toggled_err_prefix(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Failed to configure Steam Auto-Optimization",
        UiLanguage::RuRu => "Ошибка настройки автооптимизации Steam",
        UiLanguage::PtBr => "Falha ao configurar Auto-Otimização Steam",
    }
}

// =============================================================================
// Mensagens dinâmicas da página Gerenciador de Binds (ui/src/binds.rs)
// =============================================================================

pub fn binds_default_status(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => {
            "Select or type a .cfg file, capture a key, and check the desired commands."
        }
        UiLanguage::RuRu => {
            "Выберите или введите файл .cfg, назначьте клавишу и отметьте нужные команды."
        }
        UiLanguage::PtBr => {
            "Selecione ou digite um arquivo .cfg, capture uma tecla e marque os comandos desejados."
        }
    }
}

pub fn binds_save_mode_badge(lang: UiLanguage, mode: BindSaveModeKind) -> &'static str {
    match (lang, mode) {
        (UiLanguage::EnUs, BindSaveModeKind::NewFile) => "NEW FILE",
        (UiLanguage::EnUs, BindSaveModeKind::EnterName) => "ENTER NAME",
        (UiLanguage::EnUs, BindSaveModeKind::UpdateCurrent) => "UPDATE CURRENT",
        (UiLanguage::EnUs, BindSaveModeKind::CreateNew) => "CREATE NEW",
        (UiLanguage::EnUs, BindSaveModeKind::InvalidName) => "INVALID NAME",

        (UiLanguage::RuRu, BindSaveModeKind::NewFile) => "НОВЫЙ ФАЙЛ",
        (UiLanguage::RuRu, BindSaveModeKind::EnterName) => "ВВЕДИТЕ ИМЯ",
        (UiLanguage::RuRu, BindSaveModeKind::UpdateCurrent) => "ОБНОВИТЬ ТЕКУЩИЙ",
        (UiLanguage::RuRu, BindSaveModeKind::CreateNew) => "СОЗДАТЬ НОВЫЙ",
        (UiLanguage::RuRu, BindSaveModeKind::InvalidName) => "НЕВЕРНОЕ ИМЯ",

        (UiLanguage::PtBr, BindSaveModeKind::NewFile) => "NOVO ARQUIVO",
        (UiLanguage::PtBr, BindSaveModeKind::EnterName) => "DIGITE O NOME",
        (UiLanguage::PtBr, BindSaveModeKind::UpdateCurrent) => "ATUALIZAR ATUAL",
        (UiLanguage::PtBr, BindSaveModeKind::CreateNew) => "CRIAR NOVO",
        (UiLanguage::PtBr, BindSaveModeKind::InvalidName) => "NOME INVÁLIDO",
    }
}

pub fn binds_err_missing_file(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Enter a .cfg file name or select an existing file.",
        UiLanguage::RuRu => "Введите имя файла .cfg или выберите существующий файл.",
        UiLanguage::PtBr => {
            "Informe um nome para o arquivo .cfg ou selecione um arquivo existente."
        }
    }
}

pub fn binds_err_missing_key(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Click the key field and press the desired key before saving.",
        UiLanguage::RuRu => "Нажмите на поле клавиши и задайте нужную клавишу перед сохранением.",
        UiLanguage::PtBr => {
            "Clique no campo de tecla e pressione a tecla desejada antes de salvar."
        }
    }
}

pub fn binds_err_no_commands(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Check at least one command in the list below to save.",
        UiLanguage::RuRu => "Отметьте хотя бы одну команду в списке ниже для сохранения.",
        UiLanguage::PtBr => "Marque ao menos um comando na listagem abaixo para salvar.",
    }
}

pub fn binds_err_invalid_file(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    let raw = err.to_string();
    if lang == UiLanguage::PtBr {
        return format!("Nome de arquivo inválido: {raw}");
    }
    let translated = if raw.contains("não pode ser vazio") {
        match lang {
            UiLanguage::EnUs => "Configuration file name cannot be empty.".to_string(),
            UiLanguage::RuRu => "Имя файла конфигурации не может быть пустым.".to_string(),
            UiLanguage::PtBr => raw,
        }
    } else if raw.contains("referências de caminho") {
        match lang {
            UiLanguage::EnUs => {
                "Configuration file name cannot contain path references ('..').".to_string()
            }
            UiLanguage::RuRu => {
                "Имя файла конфигурации не может содержать ссылки на путь ('..').".to_string()
            }
            UiLanguage::PtBr => raw,
        }
    } else if raw.contains("terminar com ponto ou espaço") {
        match lang {
            UiLanguage::EnUs => {
                "Configuration file name cannot end with a dot or space.".to_string()
            }
            UiLanguage::RuRu => {
                "Имя файла конфигурации не может заканчиваться точкой или пробелом.".to_string()
            }
            UiLanguage::PtBr => raw,
        }
    } else if raw.contains("caracteres proibidos") {
        match lang {
            UiLanguage::EnUs => {
                "File name contains characters prohibited by the operating system.".to_string()
            }
            UiLanguage::RuRu => {
                "Имя файла содержит символы, запрещённые операционной системой.".to_string()
            }
            UiLanguage::PtBr => raw,
        }
    } else if raw.contains("reservado") {
        match lang {
            UiLanguage::EnUs => "File name reserved by the operating system.".to_string(),
            UiLanguage::RuRu => "Имя файла зарезервировано операционной системой.".to_string(),
            UiLanguage::PtBr => raw,
        }
    } else {
        raw
    };

    match lang {
        UiLanguage::EnUs => format!("Invalid file name: {translated}"),
        UiLanguage::RuRu => format!("Неверное имя файла: {translated}"),
        UiLanguage::PtBr => format!("Nome de arquivo inválido: {translated}"),
    }
}

pub fn binds_err_invalid_commands(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Error in checked commands: {err}"),
        UiLanguage::RuRu => format!("Ошибка в отмеченных командах: {err}"),
        UiLanguage::PtBr => format!("Erro nos comandos marcados: {err}"),
    }
}

pub fn binds_conflict_message(
    lang: UiLanguage,
    key: &str,
    existing_cmd: &str,
    file_name: &str,
    unified_cmd: &str,
) -> String {
    match lang {
        UiLanguage::EnUs => format!(
            "Key '{key}' is already bound to \"{existing_cmd}\" in '{file_name}'. Replace with \"{unified_cmd}\"?"
        ),
        UiLanguage::RuRu => format!(
            "Клавиша '{key}' уже назначена на \"{existing_cmd}\" в '{file_name}'. Заменить на \"{unified_cmd}\"?"
        ),
        UiLanguage::PtBr => format!(
            "A tecla '{key}' já possui a bind \"{existing_cmd}\" em '{file_name}'. Substituir por \"{unified_cmd}\"?"
        ),
    }
}

pub fn binds_conflict_status(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Confirm replacing the conflicting key above.",
        UiLanguage::RuRu => "Подтвердите замену конфликтующей клавиши выше.",
        UiLanguage::PtBr => "Confirme a substituição da tecla conflitante acima.",
    }
}

fn translate_cs2_sync_warning(lang: UiLanguage, warn: &str) -> String {
    if lang == UiLanguage::PtBr {
        return warn.to_string();
    }
    const PREFIX: &str = "Salvo em config\\cfg (sem acesso ao diretório global do CS2: ";
    if let Some(rest) = warn.strip_prefix(PREFIX) {
        let detail = rest.strip_suffix(')').unwrap_or(rest);
        return match lang {
            UiLanguage::EnUs => {
                format!("Saved to config\\cfg (no access to CS2 global folder: {detail})")
            }
            UiLanguage::RuRu => {
                format!("Сохранено в config\\cfg (нет доступа к общей папке CS2: {detail})")
            }
            UiLanguage::PtBr => warn.to_string(),
        };
    }
    warn.to_string()
}

pub fn binds_saved_created(
    lang: UiLanguage,
    file_name: &str,
    bind_line: &str,
    cs2_warning: Option<&str>,
) -> String {
    match (lang, cs2_warning) {
        (UiLanguage::EnUs, None) => {
            format!("New file '{file_name}' created with {bind_line} (local + CS2).")
        }
        (UiLanguage::EnUs, Some(warn)) => {
            let w = translate_cs2_sync_warning(lang, warn);
            format!("New file '{file_name}' created with {bind_line}. {w}")
        }
        (UiLanguage::RuRu, None) => {
            format!("Создан новый файл '{file_name}': {bind_line} (локально + CS2).")
        }
        (UiLanguage::RuRu, Some(warn)) => {
            let w = translate_cs2_sync_warning(lang, warn);
            format!("Создан новый файл '{file_name}': {bind_line}. {w}")
        }
        (UiLanguage::PtBr, None) => {
            format!("Novo arquivo '{file_name}' criado com {bind_line} (local + CS2).")
        }
        (UiLanguage::PtBr, Some(warn)) => {
            format!("Novo arquivo '{file_name}' criado com {bind_line}. {warn}")
        }
    }
}

pub fn binds_saved_updated(
    lang: UiLanguage,
    verb: BindUpdateVerb,
    file_name: &str,
    bind_line: &str,
    cs2_warning: Option<&str>,
) -> String {
    let verb_str = match (lang, verb) {
        (UiLanguage::EnUs, BindUpdateVerb::NewFileSaved) => "New file saved",
        (UiLanguage::EnUs, BindUpdateVerb::BindReplaced) => "Bind replaced",
        (UiLanguage::EnUs, BindUpdateVerb::FileUpdated) => "File updated",

        (UiLanguage::RuRu, BindUpdateVerb::NewFileSaved) => "Новый файл сохранён",
        (UiLanguage::RuRu, BindUpdateVerb::BindReplaced) => "Бинд заменён",
        (UiLanguage::RuRu, BindUpdateVerb::FileUpdated) => "Файл обновлён",

        (UiLanguage::PtBr, BindUpdateVerb::NewFileSaved) => "Novo arquivo salvo",
        (UiLanguage::PtBr, BindUpdateVerb::BindReplaced) => "Bind substituída",
        (UiLanguage::PtBr, BindUpdateVerb::FileUpdated) => "Arquivo atualizado",
    };

    match (lang, cs2_warning) {
        (UiLanguage::EnUs, None) => format!("{verb_str} in '{file_name}': {bind_line} (local + CS2)."),
        (UiLanguage::EnUs, Some(warn)) => {
            let w = translate_cs2_sync_warning(lang, warn);
            format!("{verb_str} in '{file_name}': {bind_line}. {w}")
        }
        (UiLanguage::RuRu, None) => {
            format!("{verb_str} в '{file_name}': {bind_line} (локально + CS2).")
        }
        (UiLanguage::RuRu, Some(warn)) => {
            let w = translate_cs2_sync_warning(lang, warn);
            format!("{verb_str} в '{file_name}': {bind_line}. {w}")
        }
        (UiLanguage::PtBr, None) => format!("{verb_str} em '{file_name}': {bind_line} (local + CS2)."),
        (UiLanguage::PtBr, Some(warn)) => format!("{verb_str} em '{file_name}': {bind_line}. {warn}"),
    }
}

pub fn binds_saved_cancelled(lang: UiLanguage, key: &str, path: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Operation cancelled for key '{key}' in '{path}'."),
        UiLanguage::RuRu => format!("Операция отменена для клавиши '{key}' в '{path}'."),
        UiLanguage::PtBr => format!("Operação cancelada para tecla '{key}' em '{path}'."),
    }
}

pub fn binds_saved_restored(lang: UiLanguage, file_name: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!("Backup of '{file_name}' restored."),
        UiLanguage::RuRu => format!("Резервная копия '{file_name}' восстановлена."),
        UiLanguage::PtBr => format!("Backup de '{file_name}' restaurado."),
    }
}

pub fn binds_save_err(lang: UiLanguage, err: &dyn std::fmt::Display) -> String {
    match lang {
        UiLanguage::EnUs => format!("Error saving bind: {err}"),
        UiLanguage::RuRu => format!("Ошибка сохранения бинда: {err}"),
        UiLanguage::PtBr => format!("Erro ao salvar bind: {err}"),
    }
}

pub fn binds_selection_cleared(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => {
            "Selection cleared. Type a name in the file field to create a new .cfg."
        }
        UiLanguage::RuRu => {
            "Выбор сброшен. Введите имя в поле файла, чтобы создать новый .cfg."
        }
        UiLanguage::PtBr => {
            "Seleção limpa. Digite um nome no campo de arquivo para criar um novo .cfg."
        }
    }
}

pub fn binds_file_loaded(lang: UiLanguage, selected: &str) -> String {
    match lang {
        UiLanguage::EnUs => format!(
            "File '{selected}' loaded. Keep the name to update or change it to create a new file."
        ),
        UiLanguage::RuRu => format!(
            "Файл '{selected}' загружен. Оставьте имя для обновления или измените его, чтобы создать новый."
        ),
        UiLanguage::PtBr => format!(
            "Arquivo '{selected}' carregado. Mantenha o nome para atualizar ou altere para criar um novo arquivo."
        ),
    }
}

pub fn binds_key_removed(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Key cleared. Click the field to capture another.",
        UiLanguage::RuRu => "Клавиша сброшена. Нажмите на поле, чтобы задать другую.",
        UiLanguage::PtBr => "Tecla removida. Clique no campo para capturar outra.",
    }
}

pub fn binds_key_captured(lang: UiLanguage, captured: &str) -> String {
    match lang {
        UiLanguage::EnUs => {
            format!("Key '{captured}' bound. Check the commands and click Save Bind.")
        }
        UiLanguage::RuRu => {
            format!("Клавиша '{captured}' выбрана. Отметьте команды и нажмите «Сохранить бинд».")
        }
        UiLanguage::PtBr => {
            format!("Tecla '{captured}' associada. Marque os comandos e clique em Salvar Bind.")
        }
    }
}

pub fn binds_commands_cleared(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Command selection cleared.",
        UiLanguage::RuRu => "Выбор команд очищен.",
        UiLanguage::PtBr => "Seleção de comandos limpa.",
    }
}

pub fn binds_overwrite_cancelled(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::EnUs => "Replacement cancelled. No changes were written to disk.",
        UiLanguage::RuRu => "Замена отменена. Изменения не записаны в файл.",
        UiLanguage::PtBr => "Substituição cancelada. Nenhuma alteração foi gravada no arquivo.",
    }
}
