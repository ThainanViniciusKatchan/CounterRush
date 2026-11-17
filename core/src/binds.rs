use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::{current_exe_dir, is_workspace_cwd, join_dir, CoreError};

pub const DEFAULT_CS2_CFG_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\csgo\cfg";

const RESERVED_WINDOWS_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BindFlag {
    #[default]
    Default,
    NewConfig,
    Restore,
}

impl BindFlag {
    pub fn from_str_flag(flag: &str) -> Result<Self, CoreError> {
        match flag.trim() {
            "--new_config" => Ok(Self::NewConfig),
            "--restore" => Ok(Self::Restore),
            other => Err(invalid_input(format!(
                "Flag desconhecida '{other}'. Flags suportadas: --new_config, --restore."
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindDirectories {
    pub local_cfg_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub cs2_cfg_dir: PathBuf,
}

impl Default for BindDirectories {
    fn default() -> Self {
        let local_cfg_dir = resolve_local_cfg_dir();
        let backup_dir = local_cfg_dir.join("bkp");
        Self {
            local_cfg_dir,
            backup_dir,
            cs2_cfg_dir: resolve_cs2_cfg_dir(),
        }
    }
}

impl BindDirectories {
    pub fn with_paths<P1: Into<PathBuf>, P2: Into<PathBuf>, P3: Into<PathBuf>>(
        local_cfg_dir: P1,
        backup_dir: P2,
        cs2_cfg_dir: P3,
    ) -> Self {
        Self {
            local_cfg_dir: local_cfg_dir.into(),
            backup_dir: backup_dir.into(),
            cs2_cfg_dir: cs2_cfg_dir.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindConflict {
    pub file_path: PathBuf,
    pub key: String,
    pub existing_command: String,
    pub new_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindOperationOutcome {
    Created {
        file_name: String,
        bind_line: String,
        local_path: PathBuf,
        cs2_path: PathBuf,
        backup_path: Option<PathBuf>,
    },
    Updated {
        file_name: String,
        bind_line: String,
        replaced_existing_key: bool,
        local_path: PathBuf,
        cs2_path: PathBuf,
        backup_path: Option<PathBuf>,
    },
    Restored {
        file_name: String,
        backup_path: PathBuf,
        local_path: PathBuf,
        cs2_path: PathBuf,
    },
    CancelledByUser(BindConflict),
}

fn invalid_input(message: impl Into<String>) -> CoreError {
    CoreError::Io(io::Error::new(io::ErrorKind::InvalidInput, message.into()))
}

pub(crate) fn resolve_local_cfg_dir_with(exe_dir: Option<&Path>, cwd: &Path) -> PathBuf {
    if let Some(dir) = exe_dir {
        let candidate = join_dir(dir, &["config", "cfg"]);
        if candidate.is_dir() {
            return candidate;
        }
    }

    if is_workspace_cwd(cwd) {
        let direct = join_dir(cwd, &["config", "cfg"]);
        if direct.is_dir() || join_dir(cwd, &["config"]).is_dir() {
            return direct;
        }
    }

    if let Some(dir) = exe_dir {
        return join_dir(dir, &["config", "cfg"]);
    }

    join_dir(cwd, &["config", "cfg"])
}

/// Resolve o diretório local `config\cfg` do software.
pub fn resolve_local_cfg_dir() -> PathBuf {
    resolve_local_cfg_dir_with(current_exe_dir().as_deref(), Path::new("."))
}

/// Resolve o diretório de backup automático `config\cfg\bkp`.
pub fn resolve_cfg_backup_dir() -> PathBuf {
    resolve_local_cfg_dir().join("bkp")
}

/// Retorna o caminho global padrão de arquivos `.cfg` do Counter-Strike 2.
pub fn resolve_cs2_cfg_dir() -> PathBuf {
    PathBuf::from(DEFAULT_CS2_CFG_DIR)
}

/// Valida e normaliza `nome_config`, assegurando extensão `.cfg` e proteção contra caracteres inválidos.
pub fn validate_cfg_file_name(nome_config: &str) -> Result<String, CoreError> {
    let trimmed = nome_config.trim();
    if trimmed.is_empty() {
        return Err(invalid_input("O nome do arquivo de configuração não pode ser vazio."));
    }

    let base_name = if trimmed.len() >= 4 && trimmed[trimmed.len() - 4..].eq_ignore_ascii_case(".cfg")
    {
        trimmed[..trimmed.len() - 4].trim()
    } else {
        trimmed
    };

    if base_name.is_empty() {
        return Err(invalid_input("O nome do arquivo de configuração não pode ser vazio."));
    }

    if base_name == "." || base_name == ".." || base_name.contains("..") {
        return Err(invalid_input(
            "O nome do arquivo de configuração não pode conter referências de caminho ('..').",
        ));
    }

    if base_name.ends_with('.') || base_name.ends_with(' ') {
        return Err(invalid_input(
            "O nome do arquivo de configuração não pode terminar com ponto ou espaço.",
        ));
    }

    if base_name.chars().any(|c| {
        c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    }) {
        return Err(invalid_input(
            "O nome do arquivo contém caracteres proibidos pelo sistema operacional.",
        ));
    }

    let upper_stem = base_name
        .split('.')
        .next()
        .unwrap_or(base_name)
        .to_ascii_uppercase();
    if RESERVED_WINDOWS_NAMES.contains(&upper_stem.as_str()) {
        return Err(invalid_input(
            "Nome de arquivo reservado pelo sistema operacional.",
        ));
    }

    Ok(format!("{base_name}.cfg"))
}

/// Valida o identificador da tecla de bind do CS2.
pub fn validate_bind_key(tecla: &str) -> Result<String, CoreError> {
    let trimmed = tecla.trim();
    if trimmed.is_empty() {
        return Err(invalid_input("A tecla da bind não pode ser vazia."));
    }
    if trimmed.contains('"') || trimmed.contains('\r') || trimmed.contains('\n') || trimmed.contains('\0') {
        return Err(invalid_input(
            "A tecla informada contém caracteres inválidos para sintaxe .cfg.",
        ));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(invalid_input(
            "O nome da tecla não deve conter espaços em branco.",
        ));
    }
    Ok(trimmed.to_string())
}

/// Valida e concatena múltiplos comandos de CS2 separando-os com `;`.
pub fn format_bind_commands<S: AsRef<str>>(comandos: &[S]) -> Result<String, CoreError> {
    let mut parts = Vec::new();
    for raw in comandos {
        let raw_str = raw.as_ref();
        if raw_str.contains('"') || raw_str.contains('\r') || raw_str.contains('\n') || raw_str.contains('\0') {
            return Err(invalid_input(
                "Os comandos não podem conter aspas duplas, quebras de linha ou caractere nulo.",
            ));
        }
        for segment in raw_str.split(';') {
            let clean = segment.trim();
            if !clean.is_empty() {
                parts.push(clean.to_string());
            }
        }
    }

    if parts.is_empty() {
        return Err(invalid_input(
            "Pelo menos um comando válido do CS2 deve ser informado.",
        ));
    }

    Ok(parts.join(";"))
}

/// Formata uma linha de bind na sintaxe plana oficial do CS2 (`bind "<tecla>" "<comando>"`).
pub fn format_bind_line(tecla: &str, comando_unificado: &str) -> String {
    format!("bind \"{tecla}\" \"{comando_unificado}\"")
}

fn tokenize_cfg_line(line: &str) -> Vec<String> {
    let clean = line.trim_start_matches('\u{feff}').trim_start();
    if clean.is_empty() || clean.starts_with("//") {
        return Vec::new();
    }

    let mut tokens = Vec::new();
    let mut chars = clean.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == ';' {
            break;
        }
        if ch == '/' {
            let mut lookahead = chars.clone();
            lookahead.next();
            if lookahead.peek() == Some(&'/') {
                break;
            }
        }

        if ch == '"' {
            chars.next();
            let mut token = String::new();
            while let Some(c) = chars.next() {
                if c == '"' {
                    break;
                }
                token.push(c);
            }
            tokens.push(token);
        } else {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == ';' || c == '"' {
                    break;
                }
                if c == '/' {
                    let mut lookahead = chars.clone();
                    lookahead.next();
                    if lookahead.peek() == Some(&'/') {
                        break;
                    }
                }
                token.push(c);
                chars.next();
            }
            tokens.push(token);
        }
    }

    tokens
}

/// Extrai `(tecla, comando)` caso a linha represente uma declaração `bind` ativa.
pub fn parse_bind_line(line: &str) -> Option<(String, String)> {
    let tokens = tokenize_cfg_line(line);
    if tokens.len() < 3 || !tokens[0].eq_ignore_ascii_case("bind") {
        return None;
    }
    let key = tokens[1].clone();
    let command = if tokens.len() == 3 {
        tokens[2].clone()
    } else {
        tokens[2..].join(" ")
    };
    Some((key, command))
}

/// Verifica se o conteúdo de um `.cfg` já contém um `bind` ativo para `tecla`.
pub fn find_key_conflict_in_content(content: &str, tecla: &str) -> Option<String> {
    for line in content.lines() {
        if let Some((existing_key, existing_cmd)) = parse_bind_line(line) {
            if existing_key.eq_ignore_ascii_case(tecla) {
                return Some(existing_cmd);
            }
        }
    }
    None
}

fn upsert_bind_in_content(content: &str, tecla: &str, new_bind_line: &str) -> (String, bool) {
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let clean_content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let mut result_lines: Vec<String> = Vec::new();
    let mut replaced = false;

    for raw_line in clean_content.lines() {
        if let Some((existing_key, _)) = parse_bind_line(raw_line) {
            if existing_key.eq_ignore_ascii_case(tecla) {
                if !replaced {
                    result_lines.push(new_bind_line.to_string());
                    replaced = true;
                }
                continue;
            }
        }
        result_lines.push(raw_line.to_string());
    }

    if !replaced {
        result_lines.push(new_bind_line.to_string());
    }

    let mut out = result_lines.join(line_ending);
    if !out.ends_with(line_ending) {
        out.push_str(line_ending);
    }
    (out, replaced)
}

fn write_bytes_atomically(path: &Path, data: &[u8]) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut tmp_os: OsString = path.as_os_str().to_os_string();
    tmp_os.push(format!(".tmp.{}.{}", std::process::id(), id));
    let temp_path = PathBuf::from(tmp_os);

    let res = (|| -> Result<(), io::Error> {
        std::fs::write(&temp_path, data)?;
        #[cfg(target_os = "windows")]
        {
            if std::fs::rename(&temp_path, path).is_err() {
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

    if let Err(err) = res {
        if temp_path.exists() {
            let _ = std::fs::remove_file(&temp_path);
        }
        return Err(CoreError::Io(err));
    }
    Ok(())
}

fn save_latest_backup(
    source_path: &Path,
    backup_dir: &Path,
    file_name: &str,
) -> Result<PathBuf, CoreError> {
    let bytes = std::fs::read(source_path)?;
    let backup_path = backup_dir.join(file_name);
    write_bytes_atomically(&backup_path, &bytes)?;
    Ok(backup_path)
}

/// Solicita confirmação interativa no terminal quando uma tecla já possui bind ativo.
pub fn prompt_user_bind_conflict(conflict: &BindConflict) -> Result<bool, CoreError> {
    print!(
        "Conflito de tecla detectado em '{}': a tecla '{}' já possui bind (\"{}\"). Deseja substituir por \"{}\"? [s/N]: ",
        conflict.file_path.display(),
        conflict.key,
        conflict.existing_command,
        conflict.new_command
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let normalized = input.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "s" | "sim" | "y" | "yes"))
}

fn check_target_conflict<F>(
    target_paths: &[&Path],
    tecla: &str,
    new_command: &str,
    confirm_conflict: &mut F,
) -> Result<(bool, Option<BindConflict>), CoreError>
where
    F: FnMut(&BindConflict) -> Result<bool, CoreError>,
{
    for path in target_paths {
        if !path.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(path)?;
        if let Some(existing_command) = find_key_conflict_in_content(&content, tecla) {
            let conflict = BindConflict {
                file_path: (*path).to_path_buf(),
                key: tecla.to_string(),
                existing_command,
                new_command: new_command.to_string(),
            };
            if !confirm_conflict(&conflict)? {
                return Ok((true, Some(conflict)));
            }
            return Ok((true, None));
        }
    }
    Ok((false, None))
}

fn write_cfg_to_targets(local_path: &Path, cs2_path: &Path, data: &[u8]) -> Result<(), CoreError> {
    write_bytes_atomically(local_path, data)?;
    write_bytes_atomically(cs2_path, data)?;
    Ok(())
}

/// Restaura o backup mais recente do arquivo `.cfg` a partir de `config\cfg\bkp`.
pub fn restore_cs2_cfg_backup_in_dirs(
    dirs: &BindDirectories,
    nome_config: &str,
) -> Result<BindOperationOutcome, CoreError> {
    let file_name = validate_cfg_file_name(nome_config)?;
    let backup_path = dirs.backup_dir.join(&file_name);
    if !backup_path.is_file() {
        return Err(CoreError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Backup para '{}' não encontrado em '{}'.",
                file_name,
                backup_path.display()
            ),
        )));
    }

    let bytes = std::fs::read(&backup_path)?;
    let local_path = dirs.local_cfg_dir.join(&file_name);
    let cs2_path = dirs.cs2_cfg_dir.join(&file_name);

    write_cfg_to_targets(&local_path, &cs2_path, &bytes)?;

    Ok(BindOperationOutcome::Restored {
        file_name,
        backup_path,
        local_path,
        cs2_path,
    })
}

/// Restaura o backup mais recente do arquivo informado para `config\cfg` e o diretório do CS2.
pub fn restore_cs2_cfg_backup(nome_config: &str) -> Result<BindOperationOutcome, CoreError> {
    restore_cs2_cfg_backup_in_dirs(&BindDirectories::default(), nome_config)
}

/// Gerencia binds `.cfg` do CS2 aplicando alterações em `config\cfg` e no diretório global do jogo,
/// com injeção de diretórios e callback de confirmação de conflito.
pub fn manage_cs2_bind_with_dirs<S, F>(
    dirs: &BindDirectories,
    nome_config: &str,
    tecla: Option<&str>,
    comandos: &[S],
    flag: Option<BindFlag>,
    mut confirm_conflict: F,
) -> Result<BindOperationOutcome, CoreError>
where
    S: AsRef<str>,
    F: FnMut(&BindConflict) -> Result<bool, CoreError>,
{
    let mode = flag.unwrap_or(BindFlag::Default);
    if mode == BindFlag::Restore {
        return restore_cs2_cfg_backup_in_dirs(dirs, nome_config);
    }

    let file_name = validate_cfg_file_name(nome_config)?;
    let raw_key = tecla.ok_or_else(|| {
        invalid_input("O parâmetro 'tecla' é obrigatório quando --restore não é utilizado.")
    })?;
    let clean_key = validate_bind_key(raw_key)?;
    let unified_command = format_bind_commands(comandos)?;
    let bind_line = format_bind_line(&clean_key, &unified_command);

    let local_path = dirs.local_cfg_dir.join(&file_name);
    let cs2_path = dirs.cs2_cfg_dir.join(&file_name);

    let local_exists = local_path.is_file();
    let cs2_exists = cs2_path.is_file();
    let existing_source = if local_exists {
        Some(local_path.as_path())
    } else if cs2_exists {
        Some(cs2_path.as_path())
    } else {
        None
    };

    let (had_key_conflict, cancelled_conflict) = check_target_conflict(
        &[local_path.as_path(), cs2_path.as_path()],
        &clean_key,
        &unified_command,
        &mut confirm_conflict,
    )?;
    if let Some(conflict) = cancelled_conflict {
        return Ok(BindOperationOutcome::CancelledByUser(conflict));
    }

    let backup_path = match existing_source {
        Some(src) => Some(save_latest_backup(src, &dirs.backup_dir, &file_name)?),
        None => None,
    };

    match (mode, existing_source) {
        (BindFlag::Default, Some(src)) => {
            let current_content = std::fs::read_to_string(src)?;
            let (updated_content, replaced_existing_key) =
                upsert_bind_in_content(&current_content, &clean_key, &bind_line);

            write_cfg_to_targets(&local_path, &cs2_path, updated_content.as_bytes())?;

            Ok(BindOperationOutcome::Updated {
                file_name,
                bind_line,
                replaced_existing_key,
                local_path,
                cs2_path,
                backup_path,
            })
        }
        (BindFlag::NewConfig, Some(_)) => {
            let new_content = format!("{bind_line}\n");
            write_cfg_to_targets(&local_path, &cs2_path, new_content.as_bytes())?;

            Ok(BindOperationOutcome::Updated {
                file_name,
                bind_line,
                replaced_existing_key: had_key_conflict,
                local_path,
                cs2_path,
                backup_path,
            })
        }
        (_, None) => {
            let new_content = format!("{bind_line}\n");
            write_cfg_to_targets(&local_path, &cs2_path, new_content.as_bytes())?;

            Ok(BindOperationOutcome::Created {
                file_name,
                bind_line,
                local_path,
                cs2_path,
                backup_path: None,
            })
        }
        (BindFlag::Restore, _) => unreachable!(),
    }
}

/// Gerencia binds de Counter-Strike 2 nos diretórios padrão (`config\cfg` e CS2 global),
/// solicitando confirmação interativa em caso de conflito de teclas.
pub fn manage_cs2_bind<S: AsRef<str>>(
    nome_config: &str,
    tecla: Option<&str>,
    comandos: &[S],
    flag: Option<BindFlag>,
) -> Result<BindOperationOutcome, CoreError> {
    manage_cs2_bind_with_dirs(
        &BindDirectories::default(),
        nome_config,
        tecla,
        comandos,
        flag,
        prompt_user_bind_conflict,
    )
}

/// Interpreta argumentos no formato CLI (`nome_config`, `tecla`, `comandos...` e flags
/// `--new_config` / `--restore`) e executa o gerenciamento de binds.
pub fn manage_cs2_bind_from_args_with_dirs<S, F>(
    dirs: &BindDirectories,
    args: &[S],
    confirm_conflict: F,
) -> Result<BindOperationOutcome, CoreError>
where
    S: AsRef<str>,
    F: FnMut(&BindConflict) -> Result<bool, CoreError>,
{
    let mut selected_flag: Option<BindFlag> = None;
    let mut positional: Vec<&str> = Vec::new();

    for raw in args {
        let token = raw.as_ref();
        if token.starts_with("--") {
            let parsed = BindFlag::from_str_flag(token)?;
            if let Some(prev) = selected_flag {
                if prev != parsed {
                    return Err(invalid_input(
                        "As flags --new_config e --restore não podem ser usadas juntas.",
                    ));
                }
            }
            selected_flag = Some(parsed);
        } else {
            positional.push(token);
        }
    }

    let nome_config = positional.first().copied().ok_or_else(|| {
        invalid_input("O parâmetro obrigatório 'nome_config' não foi informado.")
    })?;

    if selected_flag == Some(BindFlag::Restore) {
        return manage_cs2_bind_with_dirs::<&str, _>(
            dirs,
            nome_config,
            None,
            &[],
            Some(BindFlag::Restore),
            confirm_conflict,
        );
    }

    let tecla = positional.get(1).copied().ok_or_else(|| {
        invalid_input("O parâmetro obrigatório 'tecla' não foi informado.")
    })?;

    if positional.len() < 3 {
        return Err(invalid_input(
            "O parâmetro obrigatório 'comandos' não foi informado.",
        ));
    }

    manage_cs2_bind_with_dirs(
        dirs,
        nome_config,
        Some(tecla),
        &positional[2..],
        selected_flag,
        confirm_conflict,
    )
}

/// Interpreta argumentos no formato CLI usando os diretórios padrão do sistema e prompt interativo.
pub fn manage_cs2_bind_from_args<S: AsRef<str>>(
    args: &[S],
) -> Result<BindOperationOutcome, CoreError> {
    manage_cs2_bind_from_args_with_dirs(
        &BindDirectories::default(),
        args,
        prompt_user_bind_conflict,
    )
}
