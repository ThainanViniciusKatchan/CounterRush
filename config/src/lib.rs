use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Erro de I/O ao acessar configurações: {0}")]
    Io(#[from] std::io::Error),
    #[error("Falha ao serializar/desserializar configurações: {0}")]
    Serialization(#[from] toml::de::Error),
    #[error("Falha ao processar arquivo JSON de configurações: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub auto_backup: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: String,
    pub start_with_system: bool,
    pub optimize_on_steam_launch: bool,
    pub initial_optimization_done: bool,
    pub minimize_to_tray: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "pt_BR".to_string(),
            start_with_system: false,
            optimize_on_steam_launch: false,
            initial_optimization_done: false,
            minimize_to_tray: false,
        }
    }
}

fn write_file_atomically(path: &Path, content: &str) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_path = path.with_extension(format!("tmp.{}.{}", std::process::id(), id));

    let res = (|| -> Result<(), std::io::Error> {
        std::fs::write(&temp_path, content)?;
        #[cfg(target_os = "windows")]
        {
            if let Err(_) = std::fs::rename(&temp_path, path) {
                if path.exists() {
                    let _ = std::fs::remove_file(path);
                }
                std::fs::rename(&temp_path, path)?;
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::fs::rename(&temp_path, path)?;
        }
        Ok(())
    })();

    if let Err(e) = res {
        if temp_path.exists() {
            let _ = std::fs::remove_file(&temp_path);
        }
        return Err(e.into());
    }
    Ok(())
}

impl AppSettings {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let trimmed = content.trim_start_matches('\u{feff}').trim();
        if trimmed.is_empty() {
            let default_settings = Self::default();
            let _ = default_settings.save_to_file(path.as_ref());
            return Ok(default_settings);
        }
        let settings = serde_json::from_str(trimmed)?;
        Ok(settings)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(self)?;
        write_file_atomically(path.as_ref(), &json)
    }

    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if path.exists() {
            Self::load_from_file(path)
        } else {
            let default_settings = Self::default();
            default_settings.save_to_file(path)?;
            Ok(default_settings)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomOptimizationConfig {
    pub launch_args: Vec<String>,
    pub empty_standby_list: bool,
    pub cs2_priority: bool,
    pub cs2_affinity: bool,
    pub disable_gpu_acceleration: bool,
    pub apply_dpi_optimization: bool,
    pub disable_memory_compression: bool,
    pub apply_timer_resolution: bool,
    pub apply_mmcss: bool,
    pub disable_hpet: bool,
    pub disable_game_bar: bool,
    pub enable_msi: bool,
    pub disable_power_saving: bool,
    pub apply_render_queue: bool,
}

impl Default for CustomOptimizationConfig {
    fn default() -> Self {
        Self {
            launch_args: Vec::new(),
            empty_standby_list: true,
            cs2_priority: true,
            cs2_affinity: true,
            disable_gpu_acceleration: false,
            apply_dpi_optimization: true,
            disable_memory_compression: false,
            apply_timer_resolution: true,
            apply_mmcss: true,
            disable_hpet: false,
            disable_game_bar: true,
            enable_msi: false,
            disable_power_saving: true,
            apply_render_queue: true,
        }
    }
}

impl CustomOptimizationConfig {
    pub fn all_disabled() -> Self {
        Self {
            launch_args: Vec::new(),
            empty_standby_list: false,
            cs2_priority: false,
            cs2_affinity: false,
            disable_gpu_acceleration: false,
            apply_dpi_optimization: false,
            disable_memory_compression: false,
            apply_timer_resolution: false,
            apply_mmcss: false,
            disable_hpet: false,
            disable_game_bar: false,
            enable_msi: false,
            disable_power_saving: false,
            apply_render_queue: false,
        }
    }

    pub fn default_with_launch_args(args: Vec<String>) -> Self {
        Self {
            launch_args: args,
            ..Default::default()
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let trimmed = content.trim_start_matches('\u{feff}').trim();
        if trimmed.is_empty() {
            let default_config = Self::default();
            let _ = default_config.save_to_file(path.as_ref());
            return Ok(default_config);
        }
        let config = serde_json::from_str(trimmed)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(self)?;
        write_file_atomically(path.as_ref(), &json)
    }

    pub fn load_or_create<P: AsRef<Path>>(
        path: P,
        default_args: Vec<String>,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if path.exists() {
            Self::load_from_file(path)
        } else {
            let default_config = Self::default_with_launch_args(default_args);
            default_config.save_to_file(path)?;
            Ok(default_config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_optimization_config_all_disabled() {
        let config = CustomOptimizationConfig::all_disabled();
        assert!(config.launch_args.is_empty());
        assert!(!config.empty_standby_list);
        assert!(!config.cs2_priority);
        assert!(!config.cs2_affinity);
        assert!(!config.disable_gpu_acceleration);
        assert!(!config.apply_dpi_optimization);
        assert!(!config.disable_memory_compression);
        assert!(!config.apply_timer_resolution);
        assert!(!config.apply_mmcss);
        assert!(!config.disable_hpet);
        assert!(!config.disable_game_bar);
        assert!(!config.enable_msi);
        assert!(!config.disable_power_saving);
        assert!(!config.apply_render_queue);
    }

    #[test]
    fn test_json_roundtrip_integrity() {
        let original = CustomOptimizationConfig {
            launch_args: vec!["-novid".to_string(), "+fps_max 0".to_string()],
            empty_standby_list: true,
            cs2_priority: false,
            cs2_affinity: true,
            disable_gpu_acceleration: false,
            apply_dpi_optimization: true,
            disable_memory_compression: false,
            apply_timer_resolution: true,
            apply_mmcss: false,
            disable_hpet: true,
            disable_game_bar: false,
            enable_msi: true,
            disable_power_saving: true,
            apply_render_queue: false,
        };

        let json = serde_json::to_string_pretty(&original)
            .expect("Falha ao serializar CustomOptimizationConfig");
        let deserialized: CustomOptimizationConfig =
            serde_json::from_str(&json).expect("Falha ao desserializar CustomOptimizationConfig");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_save_and_load_file() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let target_path = temp_dir.path().join("profiles").join("test_preset.json");

        let original = CustomOptimizationConfig {
            launch_args: vec!["-high".to_string(), "-threads 8".to_string()],
            apply_timer_resolution: true,
            disable_hpet: true,
            ..CustomOptimizationConfig::all_disabled()
        };

        original
            .save_to_file(&target_path)
            .expect("Falha ao salvar no arquivo temporário");
        assert!(target_path.exists());

        let loaded = CustomOptimizationConfig::load_from_file(&target_path)
            .expect("Falha ao recarregar arquivo");
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_load_with_bom() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let target_path = temp_dir.path().join("bom_config.json");

        let json_with_bom = "\u{feff}{\"launch_args\":[\"-novid\"],\"apply_timer_resolution\":true}";
        std::fs::write(&target_path, json_with_bom).expect("Falha ao escrever JSON com BOM");

        let loaded = CustomOptimizationConfig::load_from_file(&target_path)
            .expect("Falha ao carregar arquivo com BOM");
        assert_eq!(loaded.launch_args, vec!["-novid".to_string()]);
        assert!(loaded.apply_timer_resolution);
    }

    #[test]
    fn test_malformed_json_handling() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let target_path = temp_dir.path().join("malformed.json");

        std::fs::write(&target_path, "{ \"broken_json\": [").expect("Falha ao escrever JSON inválido");

        let result = CustomOptimizationConfig::load_from_file(&target_path);
        match result {
            Err(ConfigError::Json(_)) => {}
            other => panic!("Esperava ConfigError::Json, mas obteve: {:?}", other),
        }
    }

    #[test]
    fn test_serde_default_omitted_fields() {
        let partial_json = r#"{"launch_args": ["-tickrate 128"]}"#;
        let loaded: CustomOptimizationConfig =
            serde_json::from_str(partial_json).expect("Falha ao desserializar JSON com campos omitidos");

        assert_eq!(loaded.launch_args, vec!["-tickrate 128".to_string()]);
        let default_config = CustomOptimizationConfig::default();
        assert_eq!(loaded.apply_timer_resolution, default_config.apply_timer_resolution);
        assert_eq!(loaded.cs2_priority, default_config.cs2_priority);
        assert_eq!(loaded.disable_game_bar, default_config.disable_game_bar);
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "pt_BR");
        assert!(!settings.start_with_system);
        assert!(!settings.optimize_on_steam_launch);
        assert!(!settings.initial_optimization_done);
        assert!(!settings.minimize_to_tray);
    }

    #[test]
    fn test_app_settings_roundtrip() {
        let original = AppSettings {
            language: "en_US".to_string(),
            start_with_system: true,
            optimize_on_steam_launch: true,
            initial_optimization_done: true,
            minimize_to_tray: true,
        };

        let json = serde_json::to_string_pretty(&original).expect("Falha ao serializar AppSettings");
        let deserialized: AppSettings =
            serde_json::from_str(&json).expect("Falha ao desserializar AppSettings");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_app_settings_backwards_compatible_with_missing_initial_optimization() {
        let json_legacy = r#"{
            "language": "pt_BR",
            "start_with_system": true,
            "optimize_on_steam_launch": false
        }"#;
        let settings: AppSettings = serde_json::from_str(json_legacy).expect("Falha ao desserializar");
        assert_eq!(settings.language, "pt_BR");
        assert!(settings.start_with_system);
        assert!(!settings.optimize_on_steam_launch);
        assert!(!settings.initial_optimization_done);
        assert!(!settings.minimize_to_tray);
    }

    #[test]
    fn test_app_settings_backwards_compatible_with_missing_minimize_to_tray() {
        let json_legacy = r#"{
            "language": "pt_BR",
            "start_with_system": true,
            "optimize_on_steam_launch": false,
            "initial_optimization_done": true
        }"#;
        let settings: AppSettings = serde_json::from_str(json_legacy).expect("Falha ao desserializar");
        assert_eq!(settings.language, "pt_BR");
        assert!(settings.start_with_system);
        assert!(!settings.optimize_on_steam_launch);
        assert!(settings.initial_optimization_done);
        assert!(!settings.minimize_to_tray);
    }

    #[test]
    fn test_app_settings_save_and_load_file() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir");
        let path = temp_dir.path().join("config").join("settings.json");

        let settings = AppSettings {
            language: "ru".to_string(),
            start_with_system: false,
            optimize_on_steam_launch: true,
            initial_optimization_done: true,
            minimize_to_tray: false,
        };

        settings.save_to_file(&path).expect("Falha ao salvar settings.json");
        assert!(path.exists());

        let loaded = AppSettings::load_from_file(&path).expect("Falha ao carregar settings.json");
        assert_eq!(settings, loaded);
    }

    #[test]
    fn test_app_settings_load_or_create() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir");
        let path = temp_dir.path().join("config").join("new_settings.json");

        assert!(!path.exists());
        let created = AppSettings::load_or_create(&path).expect("Falha no load_or_create");
        assert!(path.exists());
        assert_eq!(created, AppSettings::default());
    }

    #[test]
    fn test_concurrent_save_and_load_atomic() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir");
        let path = Arc::new(temp_dir.path().join("concurrent_settings.json"));

        let initial = AppSettings::default();
        initial.save_to_file(&*path).expect("Falha ao salvar inicial");

        let mut handles = Vec::new();
        for tid in 0..6 {
            let p = path.clone();
            handles.push(thread::spawn(move || {
                for i in 0..20 {
                    if tid % 2 == 0 {
                        let mut s = AppSettings::default();
                        s.minimize_to_tray = (i % 2) == 0;
                        s.language = format!("lang_{}_{}", tid, i);
                        let _ = s.save_to_file(&*p);
                    } else {
                        if let Ok(loaded) = AppSettings::load_from_file(&*p) {
                            assert!(!loaded.language.is_empty());
                        }
                    }
                }
            }));
        }

        for h in handles {
            h.join().expect("Thread concorrente falhou");
        }

        let final_loaded = AppSettings::load_from_file(&*path).expect("Arquivo final deve ser válido");
        assert!(!final_loaded.language.is_empty());
    }
}
