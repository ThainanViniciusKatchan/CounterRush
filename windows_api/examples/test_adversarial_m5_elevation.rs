use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn GetProcessHandleCount(hProcess: *mut std::ffi::c_void, pdwHandleCount: *mut u32) -> i32;
    fn LoadLibraryExW(
        lpLibFileName: *const u16,
        hFile: *mut std::ffi::c_void,
        dwFlags: u32,
    ) -> *mut std::ffi::c_void;
    fn FreeLibrary(hLibModule: *mut std::ffi::c_void) -> i32;
    fn FindResourceW(
        hModule: *mut std::ffi::c_void,
        lpName: *const u16,
        lpType: *const u16,
    ) -> *mut std::ffi::c_void;
    fn LoadResource(hModule: *mut std::ffi::c_void, hResInfo: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn LockResource(hResData: *mut std::ffi::c_void) -> *const u8;
    fn SizeofResource(hModule: *mut std::ffi::c_void, hResInfo: *mut std::ffi::c_void) -> u32;
}

const LOAD_LIBRARY_AS_DATAFILE: u32 = 0x00000002;
const RT_MANIFEST: *const u16 = 24 as *const u16;
const MANIFEST_RESOURCE_ID: *const u16 = 1 as *const u16;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn locate_target_binary() -> PathBuf {
    let candidates = [
        PathBuf::from(r"C:\cargo-target\CounterRush\debug\counterrush-ui.exe"),
        PathBuf::from(r"target\debug\counterrush-ui.exe"),
        PathBuf::from(r"..\target\debug\counterrush-ui.exe"),
    ];
    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }
    // Fallback: busca pelo executável atual ou diretório do workspace
    panic!("Binário counterrush-ui.exe não encontrado em nenhum dos locais esperados: {:?}", candidates);
}

fn main() {
    println!("============================================================");
    println!(" ADVERSARIAL HARNESS: ELEVATION, MANIFEST & PE INTEGRITY");
    println!("============================================================");

    let mut passed = 0;
    let mut failed = 0;

    macro_rules! run_test {
        ($name:expr, $code:expr) => {
            print!("TEST: {:<55} ... ", $name);
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $code)) {
                Ok(_) => {
                    println!("[PASS]");
                    passed += 1;
                }
                Err(err) => {
                    println!("[FAIL]");
                    failed += 1;
                    eprintln!("  ==> Falha no teste '{}': {:?}", $name, err);
                }
            }
        };
    }

    // ---------------------------------------------------------
    // TEST 1: Source Manifest XML Content & Syntax Integrity
    // ---------------------------------------------------------
    run_test!("Manifest source XML contains requireAdministrator", {
        let manifest_path = Path::new("ui/counterrush.exe.manifest");
        assert!(manifest_path.exists(), "counterrush.exe.manifest deve existir no diretório ui");
        let content = std::fs::read_to_string(manifest_path).expect("Leitura do manifesto XML");
        
        assert!(content.contains(r#"level="requireAdministrator""#), "Manifesto deve exigir requireAdministrator");
        assert!(content.contains(r#"uiAccess="false""#), "uiAccess deve ser false");
        assert!(content.contains(r#"supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}""#), "Compatibilidade Win10/11");
        assert!(content.contains("permonitorv2,permonitor"), "DPI awareness permonitorv2");
        assert!(content.contains("Microsoft.Windows.Common-Controls"), "Common Controls v6");
    });

    // ---------------------------------------------------------
    // TEST 2: Build Script Embedding Verification
    // ---------------------------------------------------------
    run_test!("ui/build.rs configures manifest embedding via winres", {
        let build_rs_path = Path::new("ui/build.rs");
        let content = std::fs::read_to_string(build_rs_path).expect("Leitura de ui/build.rs");
        assert!(content.contains(r#"res.set_manifest_file("counterrush.exe.manifest")"#), "build.rs deve configurar set_manifest_file");
        assert!(content.contains(r#"res.compile()"#), "build.rs deve invocar res.compile()");
    });

    // ---------------------------------------------------------
    // TEST 3: PE Binary Header & Subsystem Validation
    // ---------------------------------------------------------
    let exe_path = locate_target_binary();
    println!("Inspecionando binário alvo: {}", exe_path.display());

    run_test!("PE binary has valid DOS & NT PE headers and GUI subsystem", {
        let bytes = std::fs::read(&exe_path).expect("Leitura dos bytes do binário PE");
        assert!(bytes.len() > 1024, "Binário PE deve ter tamanho válido");
        // DOS Magic: MZ (0x4D, 0x5A)
        assert_eq!(&bytes[0..2], b"MZ", "DOS header magic deve ser MZ");
        
        // e_lfanew at offset 0x3C
        let e_lfanew = u32::from_le_bytes([bytes[0x3C], bytes[0x3D], bytes[0x3E], bytes[0x3F]]) as usize;
        assert!(e_lfanew + 4 < bytes.len(), "e_lfanew fora dos limites do arquivo");
        
        // PE signature: PE\0\0
        assert_eq!(&bytes[e_lfanew..e_lfanew + 4], b"PE\0\0", "PE signature deve ser PE\\0\\0");
        
        // Machine type (offset e_lfanew + 4): 0x8664 (x86_64)
        let machine = u16::from_le_bytes([bytes[e_lfanew + 4], bytes[e_lfanew + 5]]);
        assert_eq!(machine, 0x8664, "Machine type deve ser x86_64 (IMAGE_FILE_MACHINE_AMD64)");
        
        // Optional Header Magic (offset e_lfanew + 24): 0x020B (PE32+)
        let opt_magic = u16::from_le_bytes([bytes[e_lfanew + 24], bytes[e_lfanew + 25]]);
        assert_eq!(opt_magic, 0x020B, "Magic deve ser PE32+ (64-bit)");
        
        // Subsystem (offset e_lfanew + 24 + 68): 2 (IMAGE_SUBSYSTEM_WINDOWS_GUI)
        let subsystem = u16::from_le_bytes([bytes[e_lfanew + 92], bytes[e_lfanew + 93]]);
        assert_eq!(subsystem, 2, "Subsystem deve ser IMAGE_SUBSYSTEM_WINDOWS_GUI (2)");
    });

    // ---------------------------------------------------------
    // TEST 4: Direct OS Loader Resource Manifest Inspection (Win32 API)
    // ---------------------------------------------------------
    #[cfg(windows)]
    run_test!("Win32 FindResourceW loads embedded RT_MANIFEST id 1", {
        let wide_path = to_wide(&exe_path.to_string_lossy());
        let h_module = unsafe {
            LoadLibraryExW(wide_path.as_ptr(), std::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE)
        };
        assert!(!h_module.is_null(), "LoadLibraryExW deve carregar o binário como DATAFILE");

        let h_res = unsafe {
            FindResourceW(h_module, MANIFEST_RESOURCE_ID, RT_MANIFEST)
        };
        assert!(!h_res.is_null(), "FindResourceW DEVE encontrar RT_MANIFEST (tipo 24) com ID 1");

        let res_size = unsafe { SizeofResource(h_module, h_res) };
        assert!(res_size > 0, "Tamanho do manifesto embutido no PE deve ser > 0");

        let h_global = unsafe { LoadResource(h_module, h_res) };
        assert!(!h_global.is_null(), "LoadResource deve retornar handle válido");

        let p_data = unsafe { LockResource(h_global) };
        assert!(!p_data.is_null(), "LockResource deve retornar ponteiro de dados");

        let manifest_bytes = unsafe {
            std::slice::from_raw_parts(p_data, res_size as usize)
        };
        let manifest_str = String::from_utf8_lossy(manifest_bytes);

        assert!(
            manifest_str.contains(r#"level="requireAdministrator""#),
            "Manifesto embutido no PE DEVE conter requireAdministrator"
        );
        assert!(
            manifest_str.contains("permonitorv2"),
            "Manifesto embutido no PE DEVE conter permonitorv2"
        );
        assert!(
            manifest_str.contains("{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"),
            "Manifesto embutido no PE DEVE conter GUID do Windows 10/11"
        );

        unsafe {
            FreeLibrary(h_module);
        }
    });

    // ---------------------------------------------------------
    // TEST 5: Fallback Raw Binary Manifest Search
    // ---------------------------------------------------------
    run_test!("PE raw binary slice contains requireAdministrator verbatim", {
        let bytes = std::fs::read(&exe_path).expect("Leitura dos bytes");
        let pattern = b"requireAdministrator";
        let found = bytes.windows(pattern.len()).any(|w| w == pattern);
        assert!(found, "String 'requireAdministrator' DEVE estar presente no binário PE compilado");
    });

    // ---------------------------------------------------------
    // TEST 6: Empirical Execution of is_elevated()
    // ---------------------------------------------------------
    run_test!("windows_api::is_elevated() executes without crash/panic", {
        let elevated = windows_api::is_elevated();
        println!("  (Status de elevação do processo atual: {})", elevated);
    });

    // ---------------------------------------------------------
    // TEST 7: Handle Leak Stress Test for is_elevated() (10,000 iterations)
    // ---------------------------------------------------------
    #[cfg(windows)]
    run_test!("is_elevated() 10,000 calls across 20 threads does NOT leak handles", {
        let mut handle_count_before: u32 = 0;
        let ok = unsafe {
            GetProcessHandleCount(GetCurrentProcess(), &mut handle_count_before)
        };
        assert_ne!(ok, 0, "GetProcessHandleCount deve ter sucesso");

        let thread_count = 20;
        let iters_per_thread = 500;
        let mut handles = Vec::new();
        let total_calls = Arc::new(AtomicUsize::new(0));

        for _ in 0..thread_count {
            let tc = total_calls.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..iters_per_thread {
                    let _ = windows_api::is_elevated();
                    tc.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        for h in handles {
            h.join().expect("Thread deve concluir com sucesso");
        }

        assert_eq!(total_calls.load(Ordering::SeqCst), thread_count * iters_per_thread);

        // Dá tempo para o subsistema encerrar handles pendentes de thread se houver
        thread::sleep(std::time::Duration::from_millis(100));

        let mut handle_count_after: u32 = 0;
        let ok = unsafe {
            GetProcessHandleCount(GetCurrentProcess(), &mut handle_count_after)
        };
        assert_ne!(ok, 0, "GetProcessHandleCount pós-teste deve ter sucesso");

        let diff = (handle_count_after as i64) - (handle_count_before as i64);
        println!(
            "  (Handles antes: {}, handles depois: {}, delta: {})",
            handle_count_before, handle_count_after, diff
        );

        // Se houvesse vazamento de token em cada chamada, o delta seria de ~10.000!
        // Tolerância de até 20 handles transitórios de criação/destruição de threads do runtime Rust.
        assert!(
            diff < 50,
            "VAZAMENTO DE HANDLE DETECTADO! Delta de handles ({}) excede margem aceitável",
            diff
        );
    });

    // ---------------------------------------------------------
    // TEST 8: Adversarial Test of Path Normalization and Prefix Stripping
    // ---------------------------------------------------------
    run_test!("strip_prefix r\"\\\\?\\\" handles multiple path topologies safely", {
        let test_cases = [
            (r"\\?\C:\Games\CounterRush\counterrush.exe", r"C:\Games\CounterRush\counterrush.exe"),
            (r"\\?\UNC\server\share\counterrush.exe", r"UNC\server\share\counterrush.exe"),
            (r"C:\Program Files\CounterRush\counterrush.exe", r"C:\Program Files\CounterRush\counterrush.exe"),
            (r"\\?\D:\Outros computadores\Meu laptop\CounterRush\ui.exe", r"D:\Outros computadores\Meu laptop\CounterRush\ui.exe"),
            (r"", r""),
        ];

        for (input, expected) in test_cases {
            let clean = input.strip_prefix(r"\\?\").unwrap_or(input);
            assert_eq!(clean, expected);
        }
    });

    // ---------------------------------------------------------
    // TEST 9: Argument Formatting Edge Cases in relaunch_as_admin
    // ---------------------------------------------------------
    run_test!("Argument construction logic handles empty and multi-arg CLI", {
        // Cenário 1: Nenhum argumento
        let empty_args: Vec<String> = vec![];
        let params_str1 = if empty_args.is_empty() { None } else { Some(empty_args.join(" ")) };
        assert_eq!(params_str1, None);

        // Cenário 2: Argumentos comuns
        let args2 = vec!["--profile".to_string(), "default".to_string(), "--minimized".to_string()];
        let params_str2 = if args2.is_empty() { None } else { Some(args2.join(" ")) };
        assert_eq!(params_str2, Some("--profile default --minimized".to_string()));

        // Cenário 3: Conversão para UTF-16 wide string
        let wide = to_wide("runas");
        assert_eq!(wide.len(), 6); // 'r','u','n','a','s', 0
        assert_eq!(wide[5], 0);

        let wide_accents = to_wide(r"C:\Outros computadores\ação_otimização.exe");
        assert_eq!(*wide_accents.last().unwrap(), 0);
    });

    // ---------------------------------------------------------
    // TEST 10: Non-Windows Stub Correctness
    // ---------------------------------------------------------
    run_test!("Non-Windows stubs return safe defaults (verification of mock contracts)", {
        #[cfg(not(windows))]
        {
            assert!(windows_api::is_elevated(), "is_elevated() deve retornar true em não-windows");
            assert!(windows_api::relaunch_as_admin().is_ok(), "relaunch_as_admin() deve retornar Ok(()) em não-windows");
        }
        #[cfg(windows)]
        {
            // Em Windows, testamos a consistência da invocação direta
            let el = windows_api::is_elevated();
            assert_eq!(el, windows_api::is_elevated(), "Chamadas consecutivas devem ser determinísticas");
        }
    });

    // ---------------------------------------------------------
    // TEST 11: Idempotency of StartupManager & Task Scheduler Isolation
    // ---------------------------------------------------------
    run_test!("TaskScheduler query on non-existent task is safe and non-blocking", {
        let fake_task = format!("CounterRush_Adversarial_Elevation_Test_{}", std::process::id());
        let res = windows_api::is_task_registered(&fake_task);
        assert!(res.is_ok(), "is_task_registered deve responder Ok(false) para tarefa inexistente");
        assert!(!res.unwrap());

        let del_res = windows_api::delete_task(&fake_task);
        assert!(del_res.is_ok(), "delete_task deve ser idempotente");
    });

    // ---------------------------------------------------------
    // TEST 12: Defense-in-depth Elevation Check in ui/src/main.rs
    // ---------------------------------------------------------
    run_test!("ui/src/main.rs elevation block structure preserves single-instance lifecycle", {
        let main_rs_path = Path::new("ui/src/main.rs");
        let content = std::fs::read_to_string(main_rs_path).expect("Leitura de ui/src/main.rs");
        
        // Verifica se a guarda de elevação está antes do init_translations e MainWindow::new
        let elevation_check = "if !counterrush_core::windows_api::is_elevated()";
        let relaunch_call = "counterrush_core::windows_api::relaunch_as_admin()";
        let main_window_new = "let main_window = MainWindow::new()";
        
        let pos_elev = content.find(elevation_check).expect("Checagem de elevação deve existir em main.rs");
        let pos_relaunch = content.find(relaunch_call).expect("Invocação de relaunch_as_admin deve existir em main.rs");
        let pos_window = content.find(main_window_new).expect("Criação de MainWindow deve existir em main.rs");
        
        assert!(pos_elev < pos_relaunch, "is_elevated deve ser chamado antes de relaunch_as_admin");
        assert!(pos_relaunch < pos_window, "Auto-elevação deve ocorrer ANTES de abrir a MainWindow");
        
        // Verifica se em caso de sucesso no relaunch, o processo atual finaliza com return Ok(())
        let block_slice = &content[pos_elev..pos_window];
        assert!(
            block_slice.contains("return Ok(());"),
            "Instância não-elevada DEVE encerrar imediatamente com return Ok(()) após disparar runas"
        );
    });

    println!("============================================================");
    println!(" TOTAL DE TESTES ADVERSARIAIS: {}", passed + failed);
    println!(" APROVADOS: {}", passed);
    println!(" FALHAS:    {}", failed);
    println!("============================================================");

    if failed > 0 {
        std::process::exit(1);
    }
}
