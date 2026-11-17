use core::OptimizerCore;

fn main() {
    println!("================================================================================");
    println!("     CounterRush — Detecção de Hardware e Geração do AutoDetect.json");
    println!("================================================================================");

    let core = OptimizerCore::default();
    let hw_config_path = core::resolve_hardware_config_path(None);

    println!("\n[1] Verificando arquivo de cache de hardware: {}", hw_config_path.display());
    let (hw_path, _hardware, _category, from_cache) = match core.get_or_detect_hardware(None) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("\n[ERRO CRÍTICO] Falha ao obter especificações de hardware: {err}");
            std::process::exit(1);
        }
    };

    if from_cache {
        println!("  -> [CACHE IDENTIFICADO] Arquivo 'Hardware.json' encontrado. Dados carregados do arquivo.");
    } else {
        println!("  -> [CACHE INEXISTENTE] 'Hardware.json' não encontrado. Executada detecção e arquivo criado.");
    }

    let (autodetect_path, config, report) = match core.save_autodetect_config(None) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("\n[ERRO CRÍTICO] Falha ao processar autodetecção e gravar AutoDetect.json: {err}");
            std::process::exit(1);
        }
    };

    println!("\n[2] Especificações de Hardware Utilizadas:");
    println!("  - Processador (CPU):  {}", report.hardware.cpu.model);
    println!(
        "    Núcleos / Threads:  {} físicos / {} lógicos (Frequência: {} MHz)",
        report.hardware.cpu.physical_cores,
        report.hardware.cpu.logical_cores,
        report.hardware.cpu.frequency_mhz
    );
    println!("  - Placa de Vídeo (GPU): {}", report.hardware.gpu.model);
    println!("    VRAM Dedicada:      {:.2} GB", report.hardware.gpu.vram_gb);
    println!("  - Memória RAM:        {:.2} GB", report.hardware.ram.total_gb);
    match report.hardware.ram.speed_mhz {
        Some(speed) => println!("    Velocidade da RAM:  {} MHz", speed),
        None => println!("    Velocidade da RAM:  Padrão / Não identificada"),
    }
    println!("  - Perfil Classificado: {}", report.category);

    println!("\n[3] Avaliação de Hardware e Status das Otimizações:");
    let is_runtime_tweak = |id: &str| {
        matches!(
            id,
            "empty_standby_list" | "cs2_affinity" | "cs2_priority" | "apply_timer_resolution"
        )
    };

    for eval in &report.evaluations {
        let tipo = if is_runtime_tweak(&eval.id) {
            "Inicialização (sem teste)"
        } else {
            "Permanente"
        };

        let (tag, status_so) = if !eval.supported {
            ("[BLOQUEADO POR HARDWARE]", "Não aplicável")
        } else if eval.optional_default_off && !eval.active && !eval.already_applied {
            ("[OPCIONAL / PADRÃO OFF]", "Inativo")
        } else if eval.already_applied {
            ("[JÁ OTIMIZADO NO SO]", "Applied")
        } else if is_runtime_tweak(&eval.id) {
            ("[EXECUÇÃO AO INICIAR]", "Sem teste de status")
        } else {
            ("[PENDENTE PARA OTIMIZAR]", "Default")
        };

        println!("  {} {} ({})", tag, eval.name, tipo);
        println!("    ID no JSON:  {} = {}", eval.id, eval.active);
        println!("    Status SO:   {}", status_so);
        println!("    Critério:    {}\n", eval.reason);
    }

    println!("[4] Argumentos de Inicialização do CS2 (Launch Args):");
    if config.launch_args.is_empty() {
        println!("  Nenhum argumento definido.");
    } else {
        for arg in &config.launch_args {
            println!("  - {}", arg);
        }
    }

    println!("\n[5] Persistência dos Arquivos de Configuração:");
    println!("  - Hardware:    {}", hw_path.display());
    println!("  - AutoDetect:  {}", autodetect_path.display());
    if autodetect_path.exists() {
        if let Ok(metadata) = std::fs::metadata(&autodetect_path) {
            println!("  - Tamanho:     {} bytes", metadata.len());
        }
        println!("  - Status:      Arquivo 'AutoDetect.json' gravado e verificado com sucesso no disco.");
    }

    let count_active = report.evaluations.iter().filter(|e| e.active).count();
    let count_runtime_active = report
        .evaluations
        .iter()
        .filter(|e| e.active && is_runtime_tweak(&e.id))
        .count();
    let count_permanent_pending = report
        .evaluations
        .iter()
        .filter(|e| e.active && !is_runtime_tweak(&e.id))
        .count();
    let count_already = report.already_optimized_tweaks.len();
    let count_optional = report
        .evaluations
        .iter()
        .filter(|e| e.optional_default_off && !e.active && !e.already_applied)
        .count();
    let count_blocked = report.blocked_tweaks.len();

    println!("\n================================================================================");
    println!("  Resumo da Autodetecção:");
    println!("    - Marcadas como true no AutoDetect.json:          {}", count_active);
    println!("      * Execução ao iniciar o jogo (sem teste):       {}", count_runtime_active);
    println!("      * Permanentes pendentes (Status Default):       {}", count_permanent_pending);
    println!("    - Permanentes já aplicadas (marcadas false):      {}", count_already);
    println!("    - Opcionais mantidas desativadas (false):         {}", count_optional);
    println!("    - Bloqueadas por limite de hardware (false):      {}", count_blocked);
    println!("================================================================================");

    if !report.already_optimized_tweaks.is_empty() {
        println!("\nJá aplicadas -> false: {}", report.already_optimized_tweaks.join(", "));
    }
    if !report.pending_tweaks.is_empty() {
        println!("Ativas no JSON -> true: {}", report.pending_tweaks.join(", "));
    }

    if count_blocked > 0 {
        println!("\nRestrições Ativas no Sistema:");
        for (idx, blocked) in report.blocked_tweaks.iter().enumerate() {
            println!("  {}. {}", idx + 1, blocked);
        }
    }

    println!("\n[SUCESSO] Configuração 'AutoDetect.json' gerada conforme hardware e estado atual do sistema.");
}
