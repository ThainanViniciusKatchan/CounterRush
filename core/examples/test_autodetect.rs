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

    println!("\n[3] Avaliação de Hardware e Status Atual das Otimizações:");
    for eval in &report.evaluations {
        if !eval.supported {
            println!("  [BLOQUEADO POR HARDWARE] {}", eval.name);
            println!("    ID no JSON:  {} (false)", eval.id);
            println!("    Status SO:   Não aplicável");
            println!("    Critério:    {}", eval.reason);
        } else if eval.optional_default_off && !eval.active {
            println!("  [OPCIONAL / PADRÃO OFF] {}", eval.name);
            println!("    ID no JSON:  {} (false)", eval.id);
            println!(
                "    Status SO:   {}",
                if eval.already_applied { "Já aplicado" } else { "Inativo" }
            );
            println!("    Critério:    {}", eval.reason);
        } else if eval.already_applied {
            println!("  [JÁ OTIMIZADO NO SO] {}", eval.name);
            println!("    ID no JSON:  {} (true)", eval.id);
            println!("    Status SO:   Já otimizado (preservado sem reaplicar)");
            println!("    Critério:    {}", eval.reason);
        } else {
            println!("  [PENDENTE PARA OTIMIZAR] {}", eval.name);
            println!("    ID no JSON:  {} (true)", eval.id);
            println!(
                "    Status SO:   {}",
                if eval.needs_optimization {
                    "Pendente (será aplicado)"
                } else {
                    "Verificado"
                }
            );
            println!("    Critério:    {}", eval.reason);
        }
        println!();
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
    let count_already = report.already_optimized_tweaks.len();
    let count_pending = report.pending_tweaks.len();
    let count_optional = report.evaluations.iter().filter(|e| e.optional_default_off && !e.active).count();
    let count_blocked = report.blocked_tweaks.len();

    println!("\n================================================================================");
    println!("  Resumo da Autodetecção:");
    println!("    - Otimizações recomendadas no AutoDetect.json:    {}", count_active);
    println!("    - Já otimizadas atualmente no sistema:            {}", count_already);
    println!("    - Pendentes (ainda precisam ser otimizadas):      {}", count_pending);
    println!("    - Opcionais mantidas desativadas:                 {}", count_optional);
    println!("    - Bloqueadas por limite de hardware:              {}", count_blocked);
    println!("================================================================================");

    if !report.already_optimized_tweaks.is_empty() {
        println!("\nJá otimizadas no sistema: {}", report.already_optimized_tweaks.join(", "));
    }
    if !report.pending_tweaks.is_empty() {
        println!("Pendentes para otimizar: {}", report.pending_tweaks.join(", "));
    }

    if count_blocked > 0 {
        println!("\nRestrições Ativas no Sistema:");
        for (idx, blocked) in report.blocked_tweaks.iter().enumerate() {
            println!("  {}. {}", idx + 1, blocked);
        }
    }

    println!("\n[SUCESSO] Configuração 'AutoDetect.json' gerada conforme hardware e estado atual do sistema.");
}
