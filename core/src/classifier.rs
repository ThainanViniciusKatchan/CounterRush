use serde::{Deserialize, Serialize};
use windows_api::HardwareInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PcCategory {
    Economico,
    Intermediario,
    Potente,
}

impl std::fmt::Display for PcCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcCategory::Economico => write!(f, "PC Econômico"),
            PcCategory::Intermediario => write!(f, "PC Intermediário"),
            PcCategory::Potente => write!(f, "PC Potente"),
        }
    }
}

/// Classifica o hardware do computador com base nas especificações coletadas e nos requisitos do CS2.
///
/// ### Requisitos de Referência do CS2:
/// - SO: Windows 10
/// - CPU: Mínimo 4 threads físicas (ex.: Intel Core i5 750 ou superior)
/// - RAM: Mínimo 8 GB
/// - GPU: Mínimo 1 GB VRAM com suporte a DirectX 11 e Shader Model 5.0
///
/// ### Critérios de Classificação:
/// 1. **PC Potente:**
///    - RAM: >= 32 GB (ou >= 16 GB acompanhado de CPU >= 8 núcleos e VRAM >= 8 GB)
///    - CPU: >= 8 núcleos físicos ou >= 16 threads lógicas
///    - GPU: >= 8.0 GB de VRAM dedicada
///    *Adequado para jogos pesados, CS2 em altas taxas de quadros (240+ FPS) e cargas intensas.*
///
/// 2. **PC Intermediário:**
///    - RAM: >= 16 GB (ou >= 8 GB atendendo a CPU e GPU intermediárias)
///    - CPU: >= 6 núcleos físicos ou >= 12 threads lógicas
///    - GPU: >= 4.0 GB de VRAM dedicada
///    *Adequado para multitarefa, edição leve e jogos em configurações moderadas/altas (144+ FPS).*
///
/// 3. **PC Econômico:**
///    - Atende aos requisitos mínimos do CS2 (8 GB RAM, 4 threads, 1-3 GB VRAM) ou hardware básico.
///    *Adequado para tarefas cotidianas e CS2 em resoluções e configurações de entrada.*
pub fn classify_pc(info: &HardwareInfo) -> PcCategory {
    let cpu_cores = info.cpu.physical_cores;
    let cpu_threads = info.cpu.logical_cores;
    let ram_gb = info.ram.total_gb;
    let vram_gb = info.gpu.vram_gb;

    let is_high_end_cpu = cpu_cores >= 8 || cpu_threads >= 16;
    let is_high_end_ram = ram_gb >= 30.0;
    let is_high_end_gpu = vram_gb >= 7.5;

    if is_high_end_cpu && is_high_end_gpu && (is_high_end_ram || ram_gb >= 15.0) {
        return PcCategory::Potente;
    }

    let is_mid_cpu = cpu_cores >= 6 || cpu_threads >= 12;
    let is_mid_ram = ram_gb >= 15.0;
    let is_mid_gpu = vram_gb >= 3.5;

    if is_mid_cpu && is_mid_gpu && is_mid_ram {
        return PcCategory::Intermediario;
    }

    if (is_high_end_gpu || is_mid_gpu) && (is_mid_ram || is_high_end_ram) {
        return PcCategory::Intermediario;
    }

    PcCategory::Economico
}
