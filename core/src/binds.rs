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
    if cs2_path != local_path {
        write_bytes_atomically(cs2_path, data)?;
    }
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

/// Lista todos os arquivos `.cfg` presentes no diretório informado, ordenados por nome.
pub fn list_cfg_files_in_dir<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir.as_ref()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("cfg"))
                    == Some(true)
            {
                files.push(path);
            }
        }
    }
    files.sort_by_key(|p| {
        p.file_name()
            .map(|n| n.to_string_lossy().to_ascii_lowercase())
    });
    files
}

/// Lista todos os arquivos `.cfg` existentes no diretório local `config\cfg`.
pub fn list_cfg_files() -> Vec<PathBuf> {
    list_cfg_files_in_dir(resolve_local_cfg_dir())
}

fn is_exec_prefix_with_space(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() > 5
        && s.get(..5)
            .map(|p| p.eq_ignore_ascii_case("+exec"))
            .unwrap_or(false)
        && s[5..]
            .chars()
            .next()
            .map(char::is_whitespace)
            .unwrap_or(false)
}

fn consume_quoted_exec_value(launch_args: &[String], start_idx: usize) -> (String, usize) {
    let first = launch_args[start_idx].trim();
    if first.starts_with('"') && !(first.len() > 1 && first.ends_with('"')) {
        let mut combined = first.to_string();
        let mut idx = start_idx + 1;
        while idx < launch_args.len() {
            combined.push(' ');
            combined.push_str(launch_args[idx].trim());
            if launch_args[idx].trim().ends_with('"') {
                idx += 1;
                break;
            }
            idx += 1;
        }
        let cleaned = combined.trim().trim_matches('"').trim().to_string();
        (cleaned, idx)
    } else {
        let cleaned = first.trim_matches('"').trim().to_string();
        (cleaned, start_idx + 1)
    }
}

/// Extrai o arquivo `.cfg` configurado via `+exec` nos argumentos de inicialização.
pub fn extract_exec_cfg_from_launch_args(launch_args: &[String]) -> Option<String> {
    let mut found = None;
    let mut i = 0;

    while i < launch_args.len() {
        let trimmed = launch_args[i].trim();
        if trimmed.eq_ignore_ascii_case("+exec") {
            if let Some(next) = launch_args.get(i + 1) {
                let next_trimmed = next.trim();
                if !next_trimmed.is_empty()
                    && !next_trimmed.starts_with('-')
                    && !next_trimmed.starts_with('+')
                {
                    let (raw_val, next_i) = consume_quoted_exec_value(launch_args, i + 1);
                    if !raw_val.is_empty() {
                        if let Ok(normalized) = validate_cfg_file_name(&raw_val) {
                            found = Some(normalized);
                        }
                    }
                    i = next_i;
                    continue;
                }
            }
            i += 1;
        } else if is_exec_prefix_with_space(trimmed) {
            let rest = trimmed[5..].trim().trim_matches('"').trim();
            if !rest.is_empty() {
                if let Ok(normalized) = validate_cfg_file_name(rest) {
                    found = Some(normalized);
                }
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    found
}

/// Atualiza ou remove `+exec <nomedoarquivo.cfg>` nos parâmetros de inicialização sem duplicar entradas.
pub fn set_exec_cfg_in_launch_args(
    launch_args: &[String],
    cfg_file: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    let mut result = Vec::with_capacity(launch_args.len() + 2);
    let mut i = 0;

    while i < launch_args.len() {
        let trimmed = launch_args[i].trim();
        if trimmed.eq_ignore_ascii_case("+exec") {
            i += 1;
            if let Some(next) = launch_args.get(i) {
                let next_trimmed = next.trim();
                if !next_trimmed.starts_with('-') && !next_trimmed.starts_with('+') {
                    let (_, next_i) = consume_quoted_exec_value(launch_args, i);
                    i = next_i;
                }
            }
        } else if is_exec_prefix_with_space(trimmed) {
            i += 1;
        } else {
            result.push(launch_args[i].clone());
            i += 1;
        }
    }

    if let Some(raw) = cfg_file {
        if !raw.trim().is_empty() {
            let normalized = validate_cfg_file_name(raw)?;
            result.push("+exec".to_string());
            if normalized.chars().any(char::is_whitespace) {
                result.push(format!("\"{normalized}\""));
            } else {
                result.push(normalized);
            }
        }
    }

    Ok(result)
}

/// Lê um arquivo `.cfg` e extrai todas as linhas de bind ativas no formato `(tecla, comando)`.
pub fn read_binds_from_cfg_file(path: &Path) -> Result<Vec<(String, String)>, CoreError> {
    let content = std::fs::read_to_string(path)?;
    let mut binds = Vec::new();
    for line in content.lines() {
        if let Some(pair) = parse_bind_line(line) {
            binds.push(pair);
        }
    }
    Ok(binds)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuiBindSaveReport {
    pub outcome: BindOperationOutcome,
    pub synced_to_cs2_global: bool,
    pub cs2_sync_warning: Option<String>,
}

/// Determina se o salvamento na interface deve atualizar o arquivo existente selecionado (`BindFlag::Default`)
/// ou criar um novo arquivo preservando o original (`BindFlag::NewConfig`).
pub fn determine_bind_save_mode(
    selected_existing_file: &str,
    file_name_input: &str,
) -> Result<(String, BindFlag), CoreError> {
    let target_file = validate_cfg_file_name(file_name_input)?;
    let selected_trimmed = selected_existing_file.trim();

    if selected_trimmed.is_empty() {
        return Ok((target_file, BindFlag::NewConfig));
    }

    match validate_cfg_file_name(selected_trimmed) {
        Ok(selected_normalized) if selected_normalized.eq_ignore_ascii_case(&target_file) => {
            Ok((target_file, BindFlag::Default))
        }
        _ => Ok((target_file, BindFlag::NewConfig)),
    }
}

fn restore_snapshot(path: &Path, prev_bytes: Option<Vec<u8>>) {
    match prev_bytes {
        Some(bytes) => {
            let _ = write_bytes_atomically(path, &bytes);
        }
        None => {
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

/// Grava a bind nos diretórios informados com fallback automático para gravação local
/// quando o diretório global do CS2 estiver inacessível ou exigir elevação.
pub fn save_cs2_bind_with_local_fallback<S, F>(
    dirs: &BindDirectories,
    nome_config: &str,
    tecla: &str,
    comandos: &[S],
    flag: BindFlag,
    mut confirm_conflict: F,
) -> Result<GuiBindSaveReport, CoreError>
where
    S: AsRef<str>,
    F: FnMut(&BindConflict) -> Result<bool, CoreError>,
{
    let file_name = validate_cfg_file_name(nome_config)?;
    let local_path = dirs.local_cfg_dir.join(&file_name);
    let backup_path = dirs.backup_dir.join(&file_name);

    let prev_local_bytes = std::fs::read(&local_path).ok();
    let prev_backup_bytes = std::fs::read(&backup_path).ok();
    let mut cached_confirmation: Option<bool> = None;

    let first_attempt = manage_cs2_bind_with_dirs(
        dirs,
        nome_config,
        Some(tecla),
        comandos,
        Some(flag),
        |conflict| {
            let accepted = confirm_conflict(conflict)?;
            cached_confirmation = Some(accepted);
            Ok(accepted)
        },
    );

    match first_attempt {
        Ok(outcome) => Ok(GuiBindSaveReport {
            outcome,
            synced_to_cs2_global: true,
            cs2_sync_warning: None,
        }),
        Err(CoreError::Io(ref io_err))
            if matches!(
                io_err.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::NotFound
            ) && dirs.local_cfg_dir != dirs.cs2_cfg_dir =>
        {
            restore_snapshot(&local_path, prev_local_bytes);
            restore_snapshot(&backup_path, prev_backup_bytes);

            let warning = format!(
                "Salvo em config\\cfg (sem acesso ao diretório global do CS2: {io_err})"
            );
            let local_only_dirs = BindDirectories::with_paths(
                &dirs.local_cfg_dir,
                &dirs.backup_dir,
                &dirs.local_cfg_dir,
            );
            let outcome = manage_cs2_bind_with_dirs(
                &local_only_dirs,
                nome_config,
                Some(tecla),
                comandos,
                Some(flag),
                |conflict| match cached_confirmation {
                    Some(accepted) => Ok(accepted),
                    None => confirm_conflict(conflict),
                },
            )?;
            Ok(GuiBindSaveReport {
                outcome,
                synced_to_cs2_global: false,
                cs2_sync_warning: Some(warning),
            })
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_cfg_files_in_dir_filters_and_sorts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        std::fs::create_dir_all(root.join("bkp")).unwrap();
        std::fs::write(root.join("bkp").join("autoexec.cfg"), "bind \"k\" \"kill\"\n").unwrap();
        std::fs::write(root.join("zeta.cfg"), "bind \"z\" \"slot9\"\n").unwrap();
        std::fs::write(root.join("Alpha.CFG"), "bind \"a\" \"+left\"\n").unwrap();
        std::fs::write(root.join("notes.txt"), "ignorar\n").unwrap();

        let listed = list_cfg_files_in_dir(root);
        let names: Vec<String> = listed
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
            .collect();

        assert_eq!(names, vec!["Alpha.CFG".to_string(), "zeta.cfg".to_string()]);
    }

    #[test]
    fn test_read_binds_from_cfg_file_parses_active_lines() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cfg = temp.path().join("autoexec.cfg");
        std::fs::write(
            &cfg,
            "// comentario\nbind \"f4\" \"buy ak47;buy vesthelm\"\nbind \"v\" \"noclip\"\n",
        )
        .unwrap();

        let binds = read_binds_from_cfg_file(&cfg).expect("deve ler binds");
        assert_eq!(
            binds,
            vec![
                ("f4".to_string(), "buy ak47;buy vesthelm".to_string()),
                ("v".to_string(), "noclip".to_string()),
            ]
        );
    }

    #[test]
    fn test_save_cs2_bind_with_local_fallback_works_isolated() {
        let temp = tempfile::tempdir().expect("tempdir");
        let local_dir = temp.path().join("local_cfg");
        let bkp_dir = local_dir.join("bkp");
        let cs2_dir = temp.path().join("cs2_cfg");

        let dirs = BindDirectories::with_paths(&local_dir, &bkp_dir, &cs2_dir);
        let report = save_cs2_bind_with_local_fallback(
            &dirs,
            "teste",
            "mwheeldown",
            &["+jump"],
            BindFlag::NewConfig,
            |_| Ok(true),
        )
        .expect("gravacao deve ter sucesso");

        assert!(report.synced_to_cs2_global);
        assert!(report.cs2_sync_warning.is_none());
        assert_eq!(
            std::fs::read_to_string(local_dir.join("teste.cfg")).unwrap(),
            "bind \"mwheeldown\" \"+jump\"\n"
        );
        assert_eq!(
            std::fs::read_to_string(cs2_dir.join("teste.cfg")).unwrap(),
            "bind \"mwheeldown\" \"+jump\"\n"
        );
    }

    #[test]
    fn test_determine_bind_save_mode_and_new_file_without_selection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let local_dir = temp.path().join("local_cfg");
        let bkp_dir = local_dir.join("bkp");
        let cs2_dir = temp.path().join("cs2_cfg");
        let dirs = BindDirectories::with_paths(&local_dir, &bkp_dir, &cs2_dir);

        let (file_name, flag) =
            determine_bind_save_mode("", "novo_treino").expect("nome válido sem seleção");
        assert_eq!(file_name, "novo_treino.cfg");
        assert_eq!(flag, BindFlag::NewConfig);

        let report = save_cs2_bind_with_local_fallback(
            &dirs,
            &file_name,
            "v",
            &["noclip", "sv_rethrow_last_grenade"],
            flag,
            |_| Ok(true),
        )
        .expect("gravação de novo arquivo deve funcionar");

        assert!(matches!(report.outcome, BindOperationOutcome::Created { .. }));
        assert_eq!(
            std::fs::read_to_string(local_dir.join("novo_treino.cfg")).unwrap(),
            "bind \"v\" \"noclip;sv_rethrow_last_grenade\"\n"
        );
    }

    #[test]
    fn test_determine_bind_save_mode_updates_selected_file_preserving_existing_binds() {
        let temp = tempfile::tempdir().expect("tempdir");
        let local_dir = temp.path().join("local_cfg");
        let bkp_dir = local_dir.join("bkp");
        let cs2_dir = temp.path().join("cs2_cfg");
        std::fs::create_dir_all(&local_dir).unwrap();
        std::fs::create_dir_all(&cs2_dir).unwrap();
        let initial_content = "bind \"f4\" \"buy ak47;buy vesthelm\"\n";
        std::fs::write(local_dir.join("autoexec.cfg"), initial_content).unwrap();

        let (file_same, flag_same) =
            determine_bind_save_mode("autoexec.cfg", "autoexec.cfg").unwrap();
        assert_eq!(file_same, "autoexec.cfg");
        assert_eq!(flag_same, BindFlag::Default);

        let (file_no_ext, flag_no_ext) =
            determine_bind_save_mode("autoexec.cfg", "autoexec").unwrap();
        assert_eq!(file_no_ext, "autoexec.cfg");
        assert_eq!(flag_no_ext, BindFlag::Default);

        let dirs = BindDirectories::with_paths(&local_dir, &bkp_dir, &cs2_dir);
        let report = save_cs2_bind_with_local_fallback(
            &dirs,
            &file_no_ext,
            "mwheeldown",
            &["+jump"],
            flag_no_ext,
            |_| Ok(false),
        )
        .expect("atualização sem conflito deve funcionar");

        assert!(matches!(
            report.outcome,
            BindOperationOutcome::Updated {
                replaced_existing_key: false,
                ..
            }
        ));
        let updated = std::fs::read_to_string(local_dir.join("autoexec.cfg")).unwrap();
        assert!(updated.contains("bind \"f4\" \"buy ak47;buy vesthelm\""));
        assert!(updated.contains("bind \"mwheeldown\" \"+jump\""));
    }

    #[test]
    fn test_renaming_selected_file_creates_new_and_preserves_original_byte_for_byte() {
        let temp = tempfile::tempdir().expect("tempdir");
        let local_dir = temp.path().join("local_cfg");
        let bkp_dir = local_dir.join("bkp");
        let cs2_dir = temp.path().join("cs2_cfg");
        std::fs::create_dir_all(&local_dir).unwrap();
        std::fs::create_dir_all(&cs2_dir).unwrap();
        let original_bytes = b"// comentario preservado\r\nbind \"f4\" \"buy ak47;buy vesthelm\"\r\n";
        std::fs::write(local_dir.join("autoexec.cfg"), original_bytes).unwrap();

        let (target_file, flag) =
            determine_bind_save_mode("autoexec.cfg", "autoexec_custom.cfg").unwrap();
        assert_eq!(target_file, "autoexec_custom.cfg");
        assert_eq!(flag, BindFlag::NewConfig);

        let dirs = BindDirectories::with_paths(&local_dir, &bkp_dir, &cs2_dir);
        let report = save_cs2_bind_with_local_fallback(
            &dirs,
            &target_file,
            "c",
            &["slot8"],
            flag,
            |_| Ok(true),
        )
        .expect("criação do novo arquivo deve funcionar");

        assert!(matches!(report.outcome, BindOperationOutcome::Created { .. }));
        assert_eq!(
            std::fs::read(local_dir.join("autoexec.cfg")).unwrap(),
            original_bytes,
            "O arquivo original deve permanecer 100% intacto byte a byte"
        );
        assert_eq!(
            std::fs::read_to_string(local_dir.join("autoexec_custom.cfg")).unwrap(),
            "bind \"c\" \"slot8\"\n"
        );
    }

    #[test]
    fn test_conflict_detection_cancels_or_overwrites_with_backup() {
        let temp = tempfile::tempdir().expect("tempdir");
        let local_dir = temp.path().join("local_cfg");
        let bkp_dir = local_dir.join("bkp");
        let cs2_dir = temp.path().join("cs2_cfg");
        std::fs::create_dir_all(&local_dir).unwrap();
        std::fs::create_dir_all(&cs2_dir).unwrap();
        let original = "bind \"f4\" \"buy ak47;buy vesthelm\"\nbind \"v\" \"noclip\"\n";
        std::fs::write(local_dir.join("autoexec.cfg"), original).unwrap();

        let dirs = BindDirectories::with_paths(&local_dir, &bkp_dir, &cs2_dir);
        let (_, flag) = determine_bind_save_mode("autoexec.cfg", "autoexec.cfg").unwrap();

        let cancelled = save_cs2_bind_with_local_fallback(
            &dirs,
            "autoexec.cfg",
            "f4",
            &["slot8"],
            flag,
            |_| Ok(false),
        )
        .expect("recusa deve retornar CancelledByUser");
        assert!(matches!(
            cancelled.outcome,
            BindOperationOutcome::CancelledByUser(_)
        ));
        assert_eq!(
            std::fs::read_to_string(local_dir.join("autoexec.cfg")).unwrap(),
            original
        );

        let overwritten = save_cs2_bind_with_local_fallback(
            &dirs,
            "autoexec.cfg",
            "f4",
            &["slot8"],
            flag,
            |_| Ok(true),
        )
        .expect("confirmação deve substituir com backup");
        assert!(matches!(
            overwritten.outcome,
            BindOperationOutcome::Updated {
                replaced_existing_key: true,
                ..
            }
        ));
        assert_eq!(
            std::fs::read_to_string(bkp_dir.join("autoexec.cfg")).unwrap(),
            original
        );
        assert_eq!(
            std::fs::read_to_string(local_dir.join("autoexec.cfg")).unwrap(),
            "bind \"f4\" \"slot8\"\nbind \"v\" \"noclip\"\n"
        );
    }

    #[test]
    fn test_set_and_extract_exec_cfg_basic() {
        let initial = vec!["-fullscreen".to_string(), "-nojoy".to_string()];
        let updated = set_exec_cfg_in_launch_args(&initial, Some("autoexec.cfg"))
            .expect("deve adicionar +exec");
        assert_eq!(
            updated,
            vec!["-fullscreen", "-nojoy", "+exec", "autoexec.cfg"]
        );
        assert_eq!(
            extract_exec_cfg_from_launch_args(&updated).as_deref(),
            Some("autoexec.cfg")
        );
    }

    #[test]
    fn test_set_exec_cfg_replaces_without_duplicating() {
        let initial = vec![
            "-console".to_string(),
            "+exec old.cfg".to_string(),
            "+fps_max".to_string(),
            "240".to_string(),
            "+EXEC".to_string(),
            "autoexec.cfg".to_string(),
        ];
        let updated = set_exec_cfg_in_launch_args(&initial, Some("meuexec"))
            .expect("deve substituir sem duplicar");
        assert_eq!(
            updated,
            vec!["-console", "+fps_max", "240", "+exec", "meuexec.cfg"]
        );
        assert_eq!(
            extract_exec_cfg_from_launch_args(&updated).as_deref(),
            Some("meuexec.cfg")
        );
    }

    #[test]
    fn test_set_exec_cfg_none_removes_and_preserves_other_args() {
        let initial = vec![
            "-fullscreen".to_string(),
            "+exec".to_string(),
            "-nojoy".to_string(),
            "+exec".to_string(),
            "autoexec.cfg".to_string(),
            "+fps_max".to_string(),
            "240".to_string(),
        ];
        let cleared = set_exec_cfg_in_launch_args(&initial, None).expect("deve remover +exec");
        assert_eq!(cleared, vec!["-fullscreen", "-nojoy", "+fps_max", "240"]);
        assert_eq!(extract_exec_cfg_from_launch_args(&cleared), None);

        let cleared_empty =
            set_exec_cfg_in_launch_args(&cleared, Some("   ")).expect("string vazia limpa");
        assert_eq!(cleared_empty, cleared);
    }

    #[test]
    fn test_set_exec_cfg_handles_spaces_and_rejects_invalid_names() {
        let updated = set_exec_cfg_in_launch_args(&[], Some("minha bind.cfg"))
            .expect("nome com espaço deve ser aceito entre aspas");
        assert_eq!(updated, vec!["+exec", "\"minha bind.cfg\""]);

        let joined_tokens: Vec<String> = updated
            .join(" ")
            .split_whitespace()
            .map(str::to_string)
            .collect();
        assert_eq!(
            extract_exec_cfg_from_launch_args(&joined_tokens).as_deref(),
            Some("minha bind.cfg")
        );

        let replaced = set_exec_cfg_in_launch_args(&joined_tokens, Some("limpo.cfg"))
            .expect("deve substituir token com aspas dividido");
        assert_eq!(replaced, vec!["+exec", "limpo.cfg"]);

        assert!(set_exec_cfg_in_launch_args(&[], Some("../evil.cfg")).is_err());
    }
}
