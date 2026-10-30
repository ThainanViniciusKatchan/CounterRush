# Relatório de Investigação Backend & Arquitetura de Configuração (R1 & R3)

**Agente:** `teamwork_explorer_survey_1` (Backend Config Explorer)  
**Data:** 2026-10-30  
**Escopo:** Requisitos R1 (Estado inicial desativado na primeira execução) e R3 (Criação e persistência de novos perfis .json de otimização).

---

## 1. Observation (Observações Diretas)

### 1.1. Modelo `CustomOptimizationConfig` e Persistência
- **Localização:** `config/src/lib.rs`, linhas 20-105.
- **Definição da Struct (`config/src/lib.rs`, linhas 20-37):**
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
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
  ```
  Existem exatamente **14 campos**: `launch_args: Vec<String>` e **13 flags booleanas de otimização**.
- **Implementação do `Default` (`config/src/lib.rs`, linhas 39-58):**
  ```rust
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
  ```
  Observa-se que 8 das 13 flags são `true` na implementação padrão (`default`).
- **Serialização e Desserialização (`config/src/lib.rs`, linhas 68-90):**
  - Desserialização (`load_from_file`): Lê com `std::fs::read_to_string(path)`, faz `trim_start_matches('\u{feff}').trim()` para suportar arquivos com BOM UTF-8, e desserializa via `serde_json::from_str(trimmed)?`. Se o arquivo estiver em branco, salva e retorna o `Default`.
  - Serialização (`save_to_file`): Formata com `serde_json::to_string_pretty(self)?` e grava com `std::fs::write(path, json)?`. Garante automaticamente a criação de todos os diretórios pais com `std::fs::create_dir_all(parent)?` se `parent.exists()` for falso.

### 1.2. Diretório e Resolução de Caminhos
- **Localização:** `core/src/lib.rs`, linhas 825-957.
- **Grafia do Diretório:** O projeto utiliza a convenção `config/optimzations` (com "z").
- **Funções de Resolução de Caminho:**
  - `resolve_optimizations_dir() -> PathBuf` (`core/src/lib.rs:825-874`): Testa sequencialmente caminhos relativos ao executável (`current_exe()`), diretório de trabalho atual, diretórios pais (`../config/optimzations`, `../../config/optimzations`), e tempo de compilação (`CARGO_MANIFEST_DIR`). Retorna `PathBuf::from("config/optimzations")` como padrão.
  - `list_optimization_configs() -> Vec<PathBuf>` (`core/src/lib.rs:876-889`): Executa `std::fs::read_dir(&dir)`. Filtra entradas que sejam arquivos com extensão `"json"`. Retorna vetor ordenado (`files.sort()`). Não cria diretório se não existir (apenas retorna vetor vazio caso `read_dir` falhe).
  - `resolve_optimization_file_path(name: &str) -> PathBuf` (`core/src/lib.rs:891-899`): Concatena `resolve_optimizations_dir()` ao nome com garantia de extensão `.json`.
  - `resolve_custom_config_path(custom: Option<&Path>) -> PathBuf` (`core/src/lib.rs:901-935`): Fallback para `config/optimzations/custom_optimization.json`.

### 1.3. Comportamento Atual no Início da Aplicação (Violação do R1)
- **Localização:** `ui/src/main.rs`, linhas 5-73 e `ui/main.slint`, linhas 22-43.
- **Injeção de Perfis Fictícios (`ui/src/main.rs:21-30`):**
  ```rust
  if list.is_empty() {
      list.push(PresetModel {
          name: "AutoDetect".into(),
          tag: "HARDWARE ADAPTATIVO".into(),
      });
      list.push(PresetModel {
          name: "custom_optimization".into(),
          tag: "CUSTOMIZADO / MANUAL".into(),
      });
  }
  ```
  Se a pasta `config/optimzations` estiver vazia, `scan_presets()` injeta dois perfis inexistentes no disco.
- **Criação Prematura de Arquivo no Disco (`ui/src/main.rs:43-52`):**
  ```rust
  let (resolved_path, config) = match core.load_or_create_custom_config(None) {
      Ok(res) => res,
      Err(e) => { ... }
  };
  ```
  `core.load_or_create_custom_config(None)` chama `CustomOptimizationConfig::load_or_create`, o qual, ao detectar que `custom_optimization.json` não existe, imediatamente grava no disco um arquivo novo com os valores de `Default::default()`.
- **Ativação Forçada de Perfil e Otimizações (`ui/src/main.rs:54-71`):**
  ```rust
  if let Some(filename) = resolved_path.file_name().and_then(|s| s.to_str()) {
      let clean_name = filename.strip_suffix(".json").unwrap_or(filename);
      main_window.set_active_preset(clean_name.into());
  }
  main_window.set_cpu_tweak_active(config.cs2_affinity);
  // ... switches setados conforme a config default (muitos true)
  ```
- **Defaults na Interface Slint (`ui/main.slint:26-43`):**
  Todos os switches booleanos na janela Slint iniciam com default `true`, `active-preset` inicia com `"custom_optimization"`, e a lista `presets` inicia com dois elementos pré-fixados.

### 1.4. Ativação e Seleção de Perfil
- **Estado Ativo no Rust:** Mantido por `active_config_path = Arc::new(RwLock::new(resolved_path))` (`ui/src/main.rs:73`).
- **Callback de Seleção (`ui/src/main.rs:454-486`):**
  ```rust
  main_window.on_preset_selected(move |preset_name| {
      let target_path = counterrush_core::resolve_optimization_file_path(preset_name.as_str());
      if let Ok(mut current) = preset_config_ref.write() {
          *current = target_path.clone();
      }
      if let Ok(cfg) = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path) {
          // atualiza todos os 13 switches da interface
          ui.set_cpu_tweak_active(cfg.cs2_affinity);
          ...
      }
  });
  ```
- **Sincronização em Tempo Real (`ui/src/main.rs:78-176`):**
  Thread secundária que verifica modificações no arquivo do disco a cada 500ms e re-escaneia novos arquivos da pasta a cada 1.5s via `scan_presets()`.

---

## 2. Logic Chain (Linha de Raciocínio)

1. **Origem do Problema R1:**
   - O requisito R1 estipula: quando iniciado pela primeira vez e nenhum arquivo `.json` existir no diretório de otimizações, todas as otimizações devem iniciar desligadas (`false`) e nenhum perfil deve ser considerado ativo por padrão.
   - Atualmente, três fatores provocam a violação do R1:
     1. `scan_presets()` injeta `AutoDetect` e `custom_optimization` na lista de presets mesmo sem arquivos físicos.
     2. `main()` chama `core.load_or_create_custom_config(None)`, forçando a criação física de `custom_optimization.json` no disco com flags em `true`.
     3. A interface Slint e a inicialização de `main()` ativam o perfil `"custom_optimization"` e ativam os switches como `true`.
   - **Solução lógica para R1:**
     - Em `scan_presets()`: remover a injeção condicional fictícia, retornando vetor vazio `Vec::new()`.
     - Na inicialização em `main()`: verificar `initial_presets.is_empty()`. Se verdadeiro, **não** chamar `load_or_create_custom_config` para evitar escrita indevida no disco; definir `active-preset` como vazio (`""`); setar todas as 13 flags para `false` na UI; e manter `active_config_path` vazio ou apontando para None/arquivo não existente até uma ação explícita do usuário.
     - Em `config/src/lib.rs`: fornecer um método `CustomOptimizationConfig::all_disabled()` com todas as 13 flags em `false` e `launch_args` vazio.

2. **Fluxo do Requisito R3 (Criação e Persistência de Perfil):**
   - O usuário precisa fornecer: `nome do perfil`, `launch_args` e a seleção de quais das 13 otimizações devem ser ativadas.
   - A sanitização do nome deve remover espaços e caracteres ilegais para nomes de arquivo do Windows (`\ / : * ? " < > |`), garantindo a terminação `.json`.
   - O caminho do arquivo deve ser resolvido através de `counterrush_core::resolve_optimization_file_path(&clean_name)`.
   - Ao invocar `config.save_to_file(&target_path)`, o método já garante a criação da pasta `config/optimzations` se ela não existir (`create_dir_all`).
   - Após salvar com sucesso:
     1. Atualizar a lista de perfis na UI via `scan_presets()`.
     2. Definir o novo arquivo como `active_config_path`.
     3. Definir `active-preset` na UI com o nome do perfil recém-criado.
     4. Atualizar os switches da tela principal conforme as flags do novo perfil.
     5. Exibir mensagem de confirmação tática na barra de status.

3. **Arquitetura e Princípios SOLID:**
   - Manter separação de responsabilidades: o crate `config` gerencia o modelo e a I/O do JSON; o crate `core` gerencia caminhos e regras de sistema; e o crate `ui` orquestra a interface e callbacks.
   - Nenhuma dependência cruzada desnecessária.

---

## 3. Caveats (Ressalvas e Limitações)

1. **Caracteres Inválidos em Nomes de Arquivo:**
   No Windows, arquivos não podem conter caracteres como `< > : " / \ | ? *` nem terminar em ponto ou espaço. A rotina de criação deve validar e sanitizar essa entrada antes de tentar gravar.
2. **Nome Duplicado:**
   Caso o usuário crie um perfil com nome já existente, o sistema deve sobrescrever ou alertar o usuário. Recomenda-se informar na interface caso já exista.
3. **Persistência de Switches sem Perfil Selecionado:**
   No estado R1, quando nenhum perfil existe e o usuário clica em um switch individual, o backend atualmente tenta gravar no `target_path`. Deve-se tratar esse caso para não gerar erro ou criar um `custom_optimization.json` apenas quando explicitamente solicitado, ou emitir mensagem "Crie um perfil para salvar alterações".
4. **Permissões de Administrador:**
   Certos tweaks (como Standby List, MMCSS e MSI) requerem privilégios elevados em tempo de execução; a criação do JSON, no entanto, é estritamente em espaço de usuário e não requer elevação.

---

## 4. Conclusion (Conclusão e Proposta de Implementação)

### 4.1. Resumo dos Ajustes Necessários por Módulo

| Módulo / Arquivo | Linhas | Alteração Necessária |
| :--- | :--- | :--- |
| `config/src/lib.rs` | 59-66 | Adicionar método `pub fn all_disabled() -> Self` (ou `new_disabled()`) com todas as 13 flags `false` e `launch_args` vazio. |
| `config/src/lib.rs` | 106+ | Adicionar testes unitários automatizados cobrindo serialização/desserialização e estado desativado. |
| `core/src/lib.rs` | 876-889 | Garantir que `list_optimization_configs` funcione perfeitamente com pasta inexistente (já retorna vazio sem erro). Adicionar testes unitários. |
| `ui/src/main.rs` | 21-30 | Em `scan_presets()`: remover a adição dos presets fictícios `AutoDetect` e `custom_optimization`. Se não houver arquivos, retornar lista vazia `list`. |
| `ui/src/main.rs` | 41-73 | Na inicialização: se `initial_presets.is_empty()`, **não** chamar `load_or_create_custom_config(None)`. Definir `active-preset` como `""` e todos os switches para `false`. |
| `ui/src/main.rs` | 333-356 | No callback `on_tweak_toggled`: se nenhum perfil estiver ativo (`active_config_path` vazio), não falhar, ou inicializar sob demanda o arquivo `custom_optimization.json`. |
| `ui/src/main.rs` | Novo callback | Implementar callback `on_create_preset(name, launch_args, options)` que sanitiza o nome, monta o `CustomOptimizationConfig`, chama `save_to_file`, atualiza presets, seleciona o perfil e sincroniza os switches da tela. |
| `ui/main.slint` | 26-43 | Inicializar propriedades de tweaks com default `false`, `active-preset: ""` e `presets: []`. |
| `ui/main.slint` | 734-751 | Inserir botão "+" (R2) posicionado após o último perfil ou como único botão quando a lista estiver vazia, e conectar ao diálogo de criação (R3). |

### 4.2. Snippet Proposto para `config/src/lib.rs`:
```rust
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
}
```

### 4.3. Snippet Proposto para Criação de Perfil no Backend (`ui/src/main.rs`):
```rust
// Callback para criação de novo perfil (.json)
let window_handle = main_window.as_weak();
let create_config_ref = active_config_path.clone();
main_window.on_create_preset_confirmed(move |name_input, launch_args_input, opts| {
    if let Some(ui) = window_handle.upgrade() {
        let clean_name = name_input.trim()
            .replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "")
            .trim_end_matches(".json")
            .to_string();

        if clean_name.is_empty() {
            ui.set_status_message("Nome do perfil inválido ou vazio!".into());
            return;
        }

        let launch_args: Vec<String> = launch_args_input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let new_config = counterrush_core::config::CustomOptimizationConfig {
            launch_args,
            empty_standby_list: opts.empty_standby_list,
            cs2_priority: opts.cs2_priority,
            cs2_affinity: opts.cs2_affinity,
            disable_gpu_acceleration: opts.disable_gpu_acceleration,
            apply_dpi_optimization: opts.apply_dpi_optimization,
            disable_memory_compression: opts.disable_memory_compression,
            apply_timer_resolution: opts.apply_timer_resolution,
            apply_mmcss: opts.apply_mmcss,
            disable_hpet: opts.disable_hpet,
            disable_game_bar: opts.disable_game_bar,
            enable_msi: opts.enable_msi,
            disable_power_saving: opts.disable_power_saving,
            apply_render_queue: opts.apply_render_queue,
        };

        let target_path = counterrush_core::resolve_optimization_file_path(&clean_name);
        match new_config.save_to_file(&target_path) {
            Ok(_) => {
                if let Ok(mut current) = create_config_ref.write() {
                    *current = target_path.clone();
                }

                let updated_presets = scan_presets();
                ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                ui.set_active_preset(clean_name.as_str().into());

                // Sincroniza switches da UI
                ui.set_cpu_tweak_active(new_config.cs2_affinity);
                ui.set_timer_tweak_active(new_config.apply_timer_resolution);
                ui.set_priority_tweak_active(new_config.cs2_priority);
                ui.set_ram_tweak_active(new_config.empty_standby_list);
                ui.set_gpu_accel_tweak_active(new_config.disable_gpu_acceleration);
                ui.set_dpi_tweak_active(new_config.apply_dpi_optimization);
                ui.set_mem_compression_tweak_active(new_config.disable_memory_compression);
                ui.set_mmcss_tweak_active(new_config.apply_mmcss);
                ui.set_hpet_tweak_active(new_config.disable_hpet);
                ui.set_game_bar_tweak_active(new_config.disable_game_bar);
                ui.set_msi_tweak_active(new_config.enable_msi);
                ui.set_power_saving_tweak_active(new_config.disable_power_saving);
                ui.set_render_queue_tweak_active(new_config.apply_render_queue);

                ui.set_status_message(format!("Perfil '{clean_name}' criado e selecionado com sucesso!").into());
            }
            Err(e) => {
                ui.set_status_message(format!("Erro ao salvar perfil: {e}").into());
            }
        }
    }
});
```

---

## 5. Verification Method (Método de Verificação Independente)

1. **Verificação de Compilação e Sintaxe:**
   ```powershell
   cargo check --workspace
   ```
2. **Execução de Testes Automatizados (R4):**
   ```powershell
   cargo test -p config
   cargo test -p core
   ```
3. **Teste Funcional de Inicialização sem Perfis (R1):**
   - Mover temporariamente os arquivos de `config/optimzations` para um diretório de backup:
     ```powershell
     New-Item -ItemType Directory -Force -Path "config/backup_optimzations"
     Move-Item -Path "config/optimzations/*.json" -Destination "config/backup_optimzations"
     ```
   - Iniciar o aplicativo e checar visualmente se:
     - A lista de perfis está vazia (apenas o botão "+" visível).
     - Nenhum perfil está marcado como ativo (`active-preset == ""`).
     - Todos os 13 switches iniciam na posição desligada (`false`).
     - Nenhum arquivo foi criado indevidamente em `config/optimzations`.
4. **Teste Funcional de Criação de Perfil (R3):**
   - Clicar no botão "+", preencher o nome (ex: `Competitivo_Mirage`), parâmetros de inicialização (ex: `-console -novid +fps_max 400`), selecionar 3 otimizações e salvar.
   - Validar se o arquivo `config/optimzations/Competitivo_Mirage.json` foi criado no disco com as opções corretas.
   - Validar se o perfil se tornou ativo imediatamente e os switches refletem as opções marcadas.
   - Restaurar os arquivos de backup:
     ```powershell
     Move-Item -Path "config/backup_optimzations/*.json" -Destination "config/optimzations"
     Remove-Item -Path "config/backup_optimzations"
     ```
5. **Condições de Invalidação:**
   - Se `cargo test` falhar em qualquer módulo.
   - Se a pasta `config/optimzations` continuar recebendo um arquivo `custom_optimization.json` automaticamente na inicialização sem arquivos.
   - Se os switches iniciarem em `true` quando não existirem arquivos no disco.
