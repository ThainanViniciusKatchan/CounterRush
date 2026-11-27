use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use self_update::update::{Release, ReleaseAsset, Releases};
use self_update::version::bump_is_greater;
use sha2::{Digest, Sha256, Sha512};
use thiserror::Error;

pub const GITHUB_REPO_OWNER: &str = "ThainanViniciusKatchan";
pub const GITHUB_REPO_NAME: &str = "CounterRush";
pub const UPDATE_BIN_NAME: &str = "counterrush-ui";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAssetKind {
    MsiInstaller,
    PortableExecutable,
    PortableArchive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_name: String,
    pub asset_name: String,
    pub download_url: String,
    pub digest: Option<String>,
    pub release_notes_url: Option<String>,
    pub asset_kind: UpdateAssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheckResult {
    UpToDate { current_version: String },
    UpdateAvailable(UpdateInfo),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedUpdate {
    pub version: String,
    pub staged_path: PathBuf,
    pub asset_kind: UpdateAssetKind,
    pub already_replaced_in_place: bool,
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Erro de rede ao consultar ou baixar atualização: {0}")]
    Network(String),
    #[error("Limite de requisições da API do GitHub atingido.")]
    RateLimited { retry_after_secs: Option<u64> },
    #[error("Nenhum instalador compatível encontrado para a versão {version}.")]
    NoCompatibleAsset { version: String },
    #[error("Falha de integridade SHA no pacote baixado (esperado {expected}, obtido {computed}).")]
    ChecksumMismatch { expected: String, computed: String },
    #[error("Nome de arquivo de atualização inseguro ou inválido: '{0}'")]
    InvalidAssetName(String),
    #[error("Erro de I/O durante a atualização: {0}")]
    Io(#[from] std::io::Error),
    #[error("Erro no motor de atualização: {0}")]
    SelfUpdate(String),
    #[error("Falha ao reiniciar o aplicativo após a atualização: {0}")]
    RestartFailed(String),
}

pub type DownloadProgressFn = dyn Fn(u64, Option<u64>) + Send + Sync;

pub trait ReleaseCatalog: Send + Sync {
    fn fetch_releases(&self, current_version: &str) -> Result<Releases, UpdateError>;
}

pub trait UpdateDownloader: Send + Sync {
    fn download_asset(
        &self,
        info: &UpdateInfo,
        dest_dir: &Path,
        on_progress: Arc<DownloadProgressFn>,
    ) -> Result<PathBuf, UpdateError>;
}

#[derive(Debug, Clone)]
pub struct GitHubReleaseCatalog {
    owner: String,
    repo: String,
    bin_name: String,
}

impl Default for GitHubReleaseCatalog {
    fn default() -> Self {
        Self {
            owner: GITHUB_REPO_OWNER.to_string(),
            repo: GITHUB_REPO_NAME.to_string(),
            bin_name: UPDATE_BIN_NAME.to_string(),
        }
    }
}

impl GitHubReleaseCatalog {
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        bin_name: impl Into<String>,
    ) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            bin_name: bin_name.into(),
        }
    }
}

impl ReleaseCatalog for GitHubReleaseCatalog {
    fn fetch_releases(&self, current_version: &str) -> Result<Releases, UpdateError> {
        let updater = self_update::backends::github::Update::configure()
            .repo_owner(&self.owner)
            .repo_name(&self.repo)
            .bin_name(&self.bin_name)
            .current_version(current_version)
            .no_confirm(true)
            .show_output(false)
            .show_download_progress(false)
            .timeout(Duration::from_secs(15))
            .retries(1)
            .build()
            .map_err(map_self_update_error)?;

        match updater.get_latest_release() {
            Ok(releases) => Ok(releases),
            Err(self_update::Error::SemVer(_)) => match updater.get_newer_releases() {
                Ok(releases) => Ok(releases),
                Err(self_update::Error::NotFound { .. })
                | Err(self_update::Error::NoReleaseFound { .. }) => {
                    Ok(Releases::from_releases(Vec::new(), current_version))
                }
                Err(err) => Err(map_self_update_error(err)),
            },
            Err(self_update::Error::NotFound { .. })
            | Err(self_update::Error::NoReleaseFound { .. }) => {
                Ok(Releases::from_releases(Vec::new(), current_version))
            }
            Err(err) => Err(map_self_update_error(err)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SelfUpdateDownloader;

impl UpdateDownloader for SelfUpdateDownloader {
    fn download_asset(
        &self,
        info: &UpdateInfo,
        dest_dir: &Path,
        on_progress: Arc<DownloadProgressFn>,
    ) -> Result<PathBuf, UpdateError> {
        if !is_safe_asset_name(&info.asset_name) {
            return Err(UpdateError::InvalidAssetName(info.asset_name.clone()));
        }

        std::fs::create_dir_all(dest_dir)?;

        let final_path = dest_dir.join(&info.asset_name);
        let part_path = dest_dir.join(format!("{}.part", info.asset_name));
        if part_path.exists() {
            let _ = std::fs::remove_file(&part_path);
        }

        let download_res = (|| -> Result<(), UpdateError> {
            let mut part_file = File::create(&part_path)?;
            let mut download = self_update::Download::from_url(&info.download_url);
            download
                .request_header(
                    self_update::http::header::ACCEPT,
                    "application/octet-stream",
                )
                .request_header(
                    self_update::http::header::USER_AGENT,
                    format!("CounterRush/{}", info.current_version),
                )
                .show_download_progress(false)
                .timeout(Duration::from_secs(120));

            download.progress_callback(move |downloaded, total| {
                on_progress(downloaded, total);
            });

            download
                .download_to(&mut part_file)
                .map_err(map_self_update_error)?;
            part_file.sync_all()?;
            Ok(())
        })();

        if let Err(err) = download_res {
            let _ = std::fs::remove_file(&part_path);
            return Err(err);
        }

        let file_len = std::fs::metadata(&part_path)?.len();
        if file_len == 0 {
            let _ = std::fs::remove_file(&part_path);
            return Err(UpdateError::Network(
                "O arquivo baixado está vazio.".to_string(),
            ));
        }

        if let Some(expected_digest) = info.digest.as_deref() {
            if let Err(err) = verify_file_digest(&part_path, expected_digest) {
                let _ = std::fs::remove_file(&part_path);
                return Err(err);
            }
        }

        if final_path.exists() {
            let _ = std::fs::remove_file(&final_path);
        }
        std::fs::rename(&part_path, &final_path)?;

        Ok(final_path)
    }
}

pub struct UpdateService<C = GitHubReleaseCatalog, D = SelfUpdateDownloader> {
    catalog: C,
    downloader: D,
}

impl Default for UpdateService<GitHubReleaseCatalog, SelfUpdateDownloader> {
    fn default() -> Self {
        Self::new(GitHubReleaseCatalog::default(), SelfUpdateDownloader)
    }
}

impl<C, D> UpdateService<C, D>
where
    C: ReleaseCatalog,
    D: UpdateDownloader,
{
    pub fn new(catalog: C, downloader: D) -> Self {
        Self {
            catalog,
            downloader,
        }
    }

    pub fn check_for_updates(&self, current_version: &str) -> Result<UpdateCheckResult, UpdateError> {
        let releases = self.catalog.fetch_releases(current_version)?;
        let update_available = releases
            .is_update_available()
            .map_err(map_self_update_error)?;
        if !update_available {
            return Ok(UpdateCheckResult::UpToDate {
                current_version: current_version.to_string(),
            });
        }

        let newest = releases
            .all()
            .iter()
            .filter(|r| bump_is_greater(current_version, r.version()).unwrap_or(false))
            .max_by(|a, b| {
                if bump_is_greater(b.version(), a.version()).unwrap_or(false) {
                    std::cmp::Ordering::Less
                } else if bump_is_greater(a.version(), b.version()).unwrap_or(false) {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .or_else(|| releases.latest())
            .ok_or_else(|| {
                UpdateError::SelfUpdate("Nenhuma release mais nova encontrada.".to_string())
            })?;

        let prefer_msi = windows_api::is_installed_via_msi();
        let (asset, asset_kind) = select_compatible_asset(newest, prefer_msi).ok_or_else(|| {
            UpdateError::NoCompatibleAsset {
                version: newest.version().to_string(),
            }
        })?;

        Ok(UpdateCheckResult::UpdateAvailable(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: newest.version().to_string(),
            release_name: newest.name().to_string(),
            asset_name: asset.name().to_string(),
            download_url: asset.download_url().to_string(),
            digest: asset.digest().map(str::to_string),
            release_notes_url: newest.release_notes_url().map(str::to_string),
            asset_kind,
        }))
    }

    pub fn download_and_prepare(
        &self,
        info: &UpdateInfo,
        staging_root: &Path,
        on_progress: Arc<DownloadProgressFn>,
    ) -> Result<DownloadedUpdate, UpdateError> {
        let version_dir = staging_root.join(sanitize_version_dir(&info.latest_version));
        let downloaded_path = self.downloader.download_asset(info, &version_dir, on_progress)?;

        match info.asset_kind {
            UpdateAssetKind::MsiInstaller => Ok(DownloadedUpdate {
                version: info.latest_version.clone(),
                staged_path: downloaded_path,
                asset_kind: UpdateAssetKind::MsiInstaller,
                already_replaced_in_place: false,
            }),
            UpdateAssetKind::PortableExecutable => {
                self_replace::self_replace(&downloaded_path).map_err(|e| {
                    UpdateError::SelfUpdate(format!(
                        "Falha ao substituir executável atual: {e}"
                    ))
                })?;
                Ok(DownloadedUpdate {
                    version: info.latest_version.clone(),
                    staged_path: downloaded_path,
                    asset_kind: UpdateAssetKind::PortableExecutable,
                    already_replaced_in_place: true,
                })
            }
            UpdateAssetKind::PortableArchive => {
                let extract_dir = version_dir.join("extracted");
                if extract_dir.exists() {
                    let _ = std::fs::remove_dir_all(&extract_dir);
                }
                std::fs::create_dir_all(&extract_dir)?;

                self_update::Extract::from_source(&downloaded_path)
                    .extract_into(&extract_dir)
                    .map_err(map_self_update_error)?;

                let new_exe = find_extracted_executable(&extract_dir).ok_or_else(|| {
                    UpdateError::NoCompatibleAsset {
                        version: info.latest_version.clone(),
                    }
                })?;

                self_replace::self_replace(&new_exe).map_err(|e| {
                    UpdateError::SelfUpdate(format!(
                        "Falha ao substituir executável extraído: {e}"
                    ))
                })?;

                Ok(DownloadedUpdate {
                    version: info.latest_version.clone(),
                    staged_path: new_exe,
                    asset_kind: UpdateAssetKind::PortableArchive,
                    already_replaced_in_place: true,
                })
            }
        }
    }

    pub fn apply_and_restart<F>(
        &self,
        downloaded: &DownloadedUpdate,
        before_restart: F,
    ) -> Result<(), UpdateError>
    where
        F: FnOnce(),
    {
        if downloaded.asset_kind == UpdateAssetKind::MsiInstaller
            && !downloaded.already_replaced_in_place
        {
            let current_exe = std::env::current_exe()?;
            let relaunch_exe = windows_api::resolve_post_msi_relaunch_exe(&current_exe);
            windows_api::spawn_msi_upgrade_and_relaunch(
                &downloaded.staged_path,
                std::process::id(),
                &relaunch_exe,
            )
            .map_err(|e| UpdateError::RestartFailed(e.to_string()))?;
            before_restart();
            return Ok(());
        }

        before_restart();
        self_update::restart::restart().map_err(|e| UpdateError::RestartFailed(e.to_string()))?;
        Ok(())
    }
}

pub fn default_update_staging_dir() -> PathBuf {
    std::env::temp_dir().join("CounterRush").join("updates")
}

pub fn is_safe_asset_name(name: &str) -> bool {
    if name.is_empty()
        || name.chars().any(char::is_control)
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return false;
    }
    let mut components = Path::new(name).components();
    matches!((components.next(), components.next()), (Some(Component::Normal(_)), None))
}

fn sanitize_version_dir(version: &str) -> String {
    let cleaned: String = version
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "release".to_string()
    } else {
        cleaned
    }
}

fn is_incompatible_platform_asset(lower: &str) -> bool {
    const NON_WINDOWS_MARKERS: [&str; 9] = [
        "linux", "darwin", "macos", "apple", "aarch64", "arm64", "i686", "appimage", "deb",
    ];
    if NON_WINDOWS_MARKERS.iter().any(|m| lower.contains(m)) {
        return true;
    }
    if lower.contains("x86") && !lower.contains("x86_64") && !lower.contains("x64") {
        return true;
    }
    const METADATA_SUFFIXES: [&str; 6] = [".sha256", ".sha512", ".sig", ".minisig", ".txt", ".md"];
    METADATA_SUFFIXES.iter().any(|ext| lower.ends_with(ext))
}

fn classify_windows_asset(name: &str) -> Option<UpdateAssetKind> {
    if !is_safe_asset_name(name) {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    if is_incompatible_platform_asset(&lower) {
        return None;
    }
    if lower.ends_with(".msi") {
        Some(UpdateAssetKind::MsiInstaller)
    } else if lower.ends_with(".exe") {
        Some(UpdateAssetKind::PortableExecutable)
    } else if lower.ends_with(".zip") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        Some(UpdateAssetKind::PortableArchive)
    } else {
        None
    }
}

pub fn select_compatible_asset(
    release: &Release,
    prefer_msi: bool,
) -> Option<(ReleaseAsset, UpdateAssetKind)> {
    let mut msi: Option<ReleaseAsset> = None;
    let mut archive: Option<ReleaseAsset> = None;
    let mut exe: Option<ReleaseAsset> = None;

    for asset in release.assets() {
        let Some(kind) = classify_windows_asset(asset.name()) else {
            continue;
        };
        let is_preferred_name = asset.name().to_ascii_lowercase().contains("counterrush");
        match kind {
            UpdateAssetKind::MsiInstaller => {
                if msi.is_none() || is_preferred_name {
                    msi = Some(asset.clone());
                }
            }
            UpdateAssetKind::PortableArchive => {
                if archive.is_none() || is_preferred_name {
                    archive = Some(asset.clone());
                }
            }
            UpdateAssetKind::PortableExecutable => {
                if exe.is_none() || is_preferred_name {
                    exe = Some(asset.clone());
                }
            }
        }
    }

    if prefer_msi {
        msi.map(|a| (a, UpdateAssetKind::MsiInstaller))
            .or_else(|| archive.map(|a| (a, UpdateAssetKind::PortableArchive)))
            .or_else(|| exe.map(|a| (a, UpdateAssetKind::PortableExecutable)))
    } else {
        archive
            .map(|a| (a, UpdateAssetKind::PortableArchive))
            .or_else(|| exe.map(|a| (a, UpdateAssetKind::PortableExecutable)))
            .or_else(|| msi.map(|a| (a, UpdateAssetKind::MsiInstaller)))
    }
}

pub fn verify_file_digest(path: &Path, expected_digest: &str) -> Result<(), UpdateError> {
    let trimmed = expected_digest.trim();
    let (algo, expected_hex) = if let Some((alg, hex)) = trimmed.split_once(':') {
        (alg.trim().to_ascii_lowercase(), hex.trim().to_ascii_lowercase())
    } else if trimmed.len() == 64 {
        ("sha256".to_string(), trimmed.to_ascii_lowercase())
    } else if trimmed.len() == 128 {
        ("sha512".to_string(), trimmed.to_ascii_lowercase())
    } else {
        return Err(UpdateError::SelfUpdate(format!(
            "Formato de digest não suportado: '{trimmed}'"
        )));
    };

    let mut file = File::open(path)?;
    let mut buf = [0u8; 8192];

    let computed_hex = match algo.as_str() {
        "sha256" => {
            let mut hasher = Sha256::new();
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            hex_encode(&hasher.finalize())
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            hex_encode(&hasher.finalize())
        }
        other => {
            return Err(UpdateError::SelfUpdate(format!(
                "Algoritmo de digest não suportado: '{other}'"
            )));
        }
    };

    if computed_hex != expected_hex {
        return Err(UpdateError::ChecksumMismatch {
            expected: expected_hex,
            computed: computed_hex,
        });
    }

    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn find_extracted_executable(root: &Path) -> Option<PathBuf> {
    let mut fallback: Option<PathBuf> = None;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
            {
                let is_target = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.eq_ignore_ascii_case("counterrush-ui.exe"));
                if is_target {
                    return Some(path);
                }
                if fallback.is_none() {
                    fallback = Some(path);
                }
            }
        }
    }

    fallback
}

fn map_self_update_error(err: self_update::Error) -> UpdateError {
    match &err {
        self_update::Error::RateLimited { .. } => UpdateError::RateLimited {
            retry_after_secs: err.rate_limit_delay().map(|d| d.as_secs()),
        },
        self_update::Error::Transport(_) => UpdateError::Network(err.to_string()),
        self_update::Error::NoReleaseFound { .. } => UpdateError::NoCompatibleAsset {
            version: "latest".to_string(),
        },
        self_update::Error::InvalidAssetName { name, .. } => {
            UpdateError::InvalidAssetName(name.clone())
        }
        self_update::Error::ChecksumMismatch {
            expected, computed, ..
        } => UpdateError::ChecksumMismatch {
            expected: expected.clone(),
            computed: computed.clone(),
        },
        self_update::Error::Io(_) => UpdateError::Network(err.to_string()),
        _ => UpdateError::SelfUpdate(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockCatalog {
        releases: Releases,
    }

    impl ReleaseCatalog for MockCatalog {
        fn fetch_releases(&self, _current_version: &str) -> Result<Releases, UpdateError> {
            Ok(self.releases.clone())
        }
    }

    struct MockDownloader {
        payload: Vec<u8>,
        recorded_dir: Mutex<Option<PathBuf>>,
    }

    impl UpdateDownloader for MockDownloader {
        fn download_asset(
            &self,
            info: &UpdateInfo,
            dest_dir: &Path,
            on_progress: Arc<DownloadProgressFn>,
        ) -> Result<PathBuf, UpdateError> {
            std::fs::create_dir_all(dest_dir)?;
            let target = dest_dir.join(&info.asset_name);
            std::fs::write(&target, &self.payload)?;
            let total = self.payload.len() as u64;
            on_progress(total, Some(total));
            if let Ok(mut guard) = self.recorded_dir.lock() {
                *guard = Some(dest_dir.to_path_buf());
            }
            Ok(target)
        }
    }

    fn build_release(version: &str, assets: Vec<ReleaseAsset>) -> Release {
        let mut builder = Release::builder();
        builder
            .name(format!("CounterRush {version}"))
            .version(version)
            .release_notes_url(format!(
                "https://github.com/ThainanViniciusKatchan/CounterRush/releases/tag/{version}"
            ))
            .assets(assets);
        builder.build().expect("Release de teste deve ser válida")
    }

    #[test]
    fn test_check_for_updates_detects_newer_msi_release_via_releases_struct() {
        let release = build_release(
            "0.25.0",
            vec![ReleaseAsset::new(
                "counterrush-ui-0.25.0-x86_64.msi",
                "https://api.github.com/repos/test/assets/1",
            )
            .with_digest("sha256:abcdef")],
        );
        let catalog = MockCatalog {
            releases: Releases::from_releases(vec![release], "0.24.0"),
        };
        let service = UpdateService::new(
            catalog,
            MockDownloader {
                payload: vec![],
                recorded_dir: Mutex::new(None),
            },
        );

        let result = service.check_for_updates("0.24.0").unwrap();
        match result {
            UpdateCheckResult::UpdateAvailable(info) => {
                assert_eq!(info.latest_version, "0.25.0");
                assert_eq!(info.asset_name, "counterrush-ui-0.25.0-x86_64.msi");
                assert_eq!(info.asset_kind, UpdateAssetKind::MsiInstaller);
                assert_eq!(info.digest.as_deref(), Some("sha256:abcdef"));
            }
            other => panic!("Esperava UpdateAvailable, obteve {other:?}"),
        }
    }

    #[test]
    fn test_check_for_updates_returns_up_to_date_for_same_older_or_empty_releases() {
        for versions in [vec!["0.24.0"], vec!["0.22.1"], vec![]] {
            let releases: Vec<Release> = versions
                .into_iter()
                .map(|v| {
                    build_release(
                        v,
                        vec![ReleaseAsset::new(
                            format!("counterrush-ui-{v}-x86_64.msi"),
                            "https://example.invalid/a.msi",
                        )],
                    )
                })
                .collect();
            let catalog = MockCatalog {
                releases: Releases::from_releases(releases, "0.24.0"),
            };
            let service = UpdateService::new(
                catalog,
                MockDownloader {
                    payload: vec![],
                    recorded_dir: Mutex::new(None),
                },
            );

            assert_eq!(
                service.check_for_updates("0.24.0").unwrap(),
                UpdateCheckResult::UpToDate {
                    current_version: "0.24.0".to_string()
                }
            );
        }
    }

    #[test]
    fn test_select_compatible_asset_respects_preference_and_filters_unsafe_or_other_os() {
        let release = build_release(
            "0.25.0",
            vec![
                ReleaseAsset::new("../evil.msi", "https://example.invalid/evil"),
                ReleaseAsset::new(
                    "counterrush-ui-0.25.0-x86_64-unknown-linux-gnu.tar.gz",
                    "https://example.invalid/linux",
                ),
                ReleaseAsset::new(
                    "counterrush-ui-0.25.0-x86_64.msi",
                    "https://example.invalid/msi",
                ),
                ReleaseAsset::new(
                    "counterrush-ui-0.25.0-x86_64.zip",
                    "https://example.invalid/zip",
                ),
            ],
        );

        let (msi_asset, msi_kind) = select_compatible_asset(&release, true).unwrap();
        assert_eq!(msi_kind, UpdateAssetKind::MsiInstaller);
        assert_eq!(msi_asset.name(), "counterrush-ui-0.25.0-x86_64.msi");

        let (zip_asset, zip_kind) = select_compatible_asset(&release, false).unwrap();
        assert_eq!(zip_kind, UpdateAssetKind::PortableArchive);
        assert_eq!(zip_asset.name(), "counterrush-ui-0.25.0-x86_64.zip");

        let linux_only = build_release(
            "0.25.0",
            vec![ReleaseAsset::new(
                "counterrush-ui-0.25.0-linux.tar.gz",
                "https://example.invalid/linux",
            )],
        );
        assert!(select_compatible_asset(&linux_only, true).is_none());
    }

    #[test]
    fn test_verify_file_digest_sha256_and_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("payload.bin");
        std::fs::write(&file_path, b"hello").unwrap();

        let hello_sha256 =
            "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        verify_file_digest(&file_path, hello_sha256).expect("SHA-256 válido deve passar");

        let err = verify_file_digest(&file_path, &format!("sha256:{}", "00".repeat(32)))
            .unwrap_err();
        assert!(matches!(err, UpdateError::ChecksumMismatch { .. }));
    }

    #[test]
    fn test_download_and_prepare_msi_keeps_staged_installer() {
        let tmp = tempfile::tempdir().unwrap();
        let info = UpdateInfo {
            current_version: "0.24.0".to_string(),
            latest_version: "0.25.0".to_string(),
            release_name: "0.25.0".to_string(),
            asset_name: "counterrush-ui-0.25.0-x86_64.msi".to_string(),
            download_url: "https://example.invalid/asset".to_string(),
            digest: None,
            release_notes_url: None,
            asset_kind: UpdateAssetKind::MsiInstaller,
        };

        let service = UpdateService::new(
            MockCatalog {
                releases: Releases::from_releases(vec![], "0.24.0"),
            },
            MockDownloader {
                payload: b"fake-msi-bytes".to_vec(),
                recorded_dir: Mutex::new(None),
            },
        );

        let progress_calls = Arc::new(Mutex::new(Vec::new()));
        let calls_ref = Arc::clone(&progress_calls);
        let downloaded = service
            .download_and_prepare(
                &info,
                tmp.path(),
                Arc::new(move |dl, total| {
                    calls_ref.lock().unwrap().push((dl, total));
                }),
            )
            .expect("Download mock deve concluir");

        assert_eq!(downloaded.version, "0.25.0");
        assert_eq!(downloaded.asset_kind, UpdateAssetKind::MsiInstaller);
        assert!(!downloaded.already_replaced_in_place);
        assert!(downloaded.staged_path.exists());
        assert_eq!(
            progress_calls.lock().unwrap().as_slice(),
            &[(14, Some(14))]
        );
    }

    #[test]
    fn test_apply_and_restart_missing_msi_does_not_invoke_before_restart() {
        let tmp = tempfile::tempdir().unwrap();
        let missing_msi = tmp.path().join("missing-installer.msi");
        let downloaded = DownloadedUpdate {
            version: "0.24.5".to_string(),
            staged_path: missing_msi,
            asset_kind: UpdateAssetKind::MsiInstaller,
            already_replaced_in_place: false,
        };

        let service = UpdateService::new(
            MockCatalog {
                releases: Releases::from_releases(vec![], "0.24.4"),
            },
            MockDownloader {
                payload: vec![],
                recorded_dir: Mutex::new(None),
            },
        );

        let callback_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_ref = Arc::clone(&callback_called);
        let err = service
            .apply_and_restart(&downloaded, move || {
                callback_ref.store(true, std::sync::atomic::Ordering::SeqCst);
            })
            .unwrap_err();

        assert!(matches!(err, UpdateError::RestartFailed(_)));
        assert!(
            !callback_called.load(std::sync::atomic::Ordering::SeqCst),
            "before_restart não deve ser chamado quando o spawn do instalador MSI falha"
        );
    }
}
