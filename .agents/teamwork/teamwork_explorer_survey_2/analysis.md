# Relatório Técnico de Análise da Base de Código — Explorer 2

**Data:** 2026-11-01  
**Autor:** Explorer 2 (Teamwork CounterRush)  
**Objetivo:** Investigação read-only sobre o gerenciamento de configurações `.json`, fluxo de otimização automática por hardware e transições de estado da interface na aba de Otimizações.

---

## 1. Visão Geral e Arquitetura do Projeto

O CounterRush é construído em Rust modular organizado em Cargo Workspaces, com camada de apresentação reativa utilizando a engine Slint (`slint 1.8`):
- `config`: Estrutura de dados serializável (`CustomOptimizationConfig`), leitura, gravação e tolerância a falhas (ex.: remoção de UTF-8 BOM).
- `core`: Orquestrador central (`OptimizerCore`), resolução de diretórios de perfis, detecção de hardware, calibração e categorização de perfis (`classifier`).
- `tweaks`: Implementações individuais e verificação de status dos 13 módulos de latência do Windows e do jogo.
- `windows_api`: Comunicação com APIs de baixo nível do Windows, PowerShell e detecção de hardware do sistema.
- `ui`: Ponto de entrada gráfico (`main.rs`), componentes e janelas Slint (`ui/main.slint`, `components/*.slint`).

---

## 2. Gerenciamento de Estado e Persistência de Arquivos `.json` (Item 1)

### 2.1. Onde os arquivos são salvos
- **Diretório de Otimizações:** Os arquivos de configuração de perfil são armazenados no diretório `config/optimzations` (resolvido por `counterrush_core::resolve_optimizations_dir()` em `core/src/lib.rs:1013-1035`). O caminho busca `exe_dir/config/optimzations` ou o diretório relativo `config/optimzations`.
- **Arquivos Padrão:**
  - `AutoDetect.json`: Perfil adaptativo gerado automaticamente baseado no hardware.
  - `custom_optimization.json`: Perfil manual / customizado.
  - `<perfil>.json`: Perfis personalizados criados pelo usuário.
- **Cache de Hardware:** O relatório de hardware detectado fica em `config/hardware/Hardware.json` (resolvido por `resolve_hardware_config_path()` em `core/src/lib.rs:1166-1219`).

### 2.2. Como os arquivos são listados e carregados
1. **Listagem:** A função `counterrush_core::list_optimization_configs()` (em `core/src/lib.rs:1057-1059`) invoca `list_optimization_configs_in_dir`, lendo o diretório `config/optimzations`, filtrando entradas que terminam com `.json` (case-insensitive) e ordenando-as alfabeticamente.
2. **Transformação para o Modelo de UI:** Em `ui/src/main.rs:5-22`, a função `scan_presets()` itera sobre os caminhos retornados e cria instâncias de `PresetModel`:
   ```rust
   PresetModel {
       name: stem.into(),
       tag: match stem {
           "AutoDetect" => "HARDWARE ADAPTATIVO",
           "custom_optimization" => "CUSTOMIZADO / MANUAL",
           _ => "PERFIL DO USUÁRIO",
       }.into(),
   }
   ```
3. **Carregamento da Configuração:** `CustomOptimizationConfig::load_from_file(&path)` (em `config/src/lib.rs:87-97`):
   - Lê o conteúdo como string com `std::fs::read_to_string`.
   - Remove eventuais marcas de ordem de byte UTF-8 (`\u{feff}`).
   - Se o arquivo estiver vazio, cria e grava os padrões.
   - Desserializa via `serde_json::from_str`.
4. **Estado na Aplicação Slint:**
   - `presets: in-out property <[PresetModel]>`: Modelo reativo contendo a lista atual de perfis.
   - `active-preset: in-out property <string>`: Nome do perfil atualmente selecionado (vazio `""` se nenhum existir).
   - `active_config_path: Arc<RwLock<PathBuf>>`: Ponteiro de sincronização em tempo real contendo o caminho do arquivo selecionado em disco.

### 2.3. Sincronização em Tempo Real (Thread de Monitoramento)
Em `ui/src/main.rs:90-225`, uma thread dedicada roda em loop com sleep de 500ms:
- A cada 1.5s monitora o disco (`scan_presets()`). Se a lista mudar:
  - Se a lista ficar vazia, atualiza `presets` para vazio, limpa `active_preset = ""` e `launch_args_text = ""`.
  - Se o perfil selecionado foi removido, seleciona o primeiro restante.
- Monitora a data de modificação (`mtime`) do arquivo ativo: se o JSON for modificado externamente no disco, recarrega e propaga imediatamente as alterações para os toggles e launch args na UI através de `slint::invoke_from_event_loop`.

---

## 3. Lógica de "Otimizar Automaticamente" Baseada em Hardware (Item 2)

### 3.1. Módulos Envolvidos
- `windows_api::detect_hardware()`: Detecta CPU (frequência, núcleos físicos/lógicos, modelo), RAM (GB total/disponível, velocidade), GPU (modelo, VRAM total em GB, versão de driver) e versão do Windows.
- `classifier::classify_pc(&hardware)` (em `core/src/classifier.rs:45-72`): Classifica o computador em:
  - `PcCategory::Potente` (ex.: CPU >= 8 núcleos, VRAM >= 7.5GB, RAM >= 16GB)
  - `PcCategory::Intermediario` (ex.: CPU >= 6 núcleos, VRAM >= 3.5GB, RAM >= 15GB)
  - `PcCategory::Economico` (hardware de entrada / baseline)
- `counterrush_core::OptimizerCore::save_autodetect_config(None)` (em `core/src/lib.rs:745-753`):
  1. Chama `self.generate_autodetect_config()`.
  2. Determina argumentos de inicialização recomendados por categoria via `get_default_launch_args_for_category(category)`.
  3. Calibra os 13 tweaks verificando restrições de hardware e status do sistema:
     - `cs2_affinity`: requer CPU com mais de 4 núcleos físicos (`physical_cores > 4`).
     - `disable_memory_compression`: requer mais de 16 GB de RAM (`ram_gb > 16.0`).
     - `apply_render_queue`: ativado se GPU NVIDIA ou AMD detectada.
     - `disable_gpu_acceleration`: ativado para Econômico e Intermediário, mantido desativado se Potente.
     - `empty_standby_list`, `apply_dpi_optimization`, `apply_timer_resolution`, `apply_mmcss`, `disable_hpet`, `disable_game_bar`, `enable_msi`, `disable_power_saving`: avaliados e definidos de acordo com o hardware.
  4. Salva a configuração gerada em `config/optimzations/AutoDetect.json`.
  5. Retorna `(path, config, report)`.

### 3.2. Callback Existente no Backend Rust
Em `ui/src/main.rs:466-517`:
```rust
main_window.on_auto_optimize_clicked(move || {
    if let Some(ui) = window_handle.upgrade() {
        ui.set_status_message("Detectando hardware e calibrando otimizações...".into());

        let core = counterrush_core::OptimizerCore::default();
        match core.save_autodetect_config(None) {
            Ok((path, config, report)) => {
                if let Ok(mut current) = auto_opt_ref.write() {
                    *current = path.clone();
                }

                let updated_presets = scan_presets();
                ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                ui.set_active_preset("AutoDetect".into());
                ui.set_launch_args_text(config.launch_args.join(" ").into());
                ui.set_is_editing_launch_args(false);

                ui.set_cpu_tweak_active(config.cs2_affinity);
                ui.set_timer_tweak_active(config.apply_timer_resolution);
                ui.set_priority_tweak_active(config.cs2_priority);
                ui.set_ram_tweak_active(config.empty_standby_list);
                ui.set_gpu_accel_tweak_active(config.disable_gpu_acceleration);
                ui.set_dpi_tweak_active(config.apply_dpi_optimization);
                ui.set_mem_compression_tweak_active(config.disable_memory_compression);
                ui.set_mmcss_tweak_active(config.apply_mmcss);
                ui.set_hpet_tweak_active(config.disable_hpet);
                ui.set_game_bar_tweak_active(config.disable_game_bar);
                ui.set_msi_tweak_active(config.enable_msi);
                ui.set_power_saving_tweak_active(config.disable_power_saving);
                ui.set_render_queue_tweak_active(config.apply_render_queue);

                let msg = if report.blocked_tweaks.is_empty() {
                    format!(
                        "Hardware detectado ({:?} • {}). Perfil AutoDetect gerado e aplicado!",
                        report.category, report.hardware.gpu.model
                    )
                } else {
                    format!(
                        "Hardware detectado ({:?} • {}). Perfil AutoDetect gerado ({} ajustes calibrados para seu PC).",
                        report.category,
                        report.hardware.gpu.model,
                        report.blocked_tweaks.len()
                    )
                };
                ui.set_status_message(msg.into());
            }
            Err(err) => {
                ui.set_status_message(format!("Falha ao detectar hardware automaticamente: {err}").into());
            }
        }
    }
});
```

---

## 4. Acionamento no Estado Sem Arquivos `.json` e Fluxos de Refresh (Itens 3 e 4)

### 4.1. Condição do Estado Vazio na UI
No Slint, a ausência de arquivos `.json` é refletida diretamente pela propriedade reativa:
```slint
root.presets.length == 0
```
Quando esta condição é verdadeira:
1. `root.active-preset` é `""` (vazio).
2. O campo de parâmetros de inicialização (`launch-args-text`) já possui a guarda reativa `if root.active-preset != ""` (não é exibido).
3. A seção de gerenciamento de perfis deve exibir **exclusivamente dois botões de ação**:
   - **Botão 1:** "Otimizar Automaticamente"
     - Ao ser clicado, chama `root.auto-optimize-clicked()`.
   - **Botão 2:** "Criar Otimização" com símbolo `+`
     - Componente `AddPresetButton` existente, configurado com o texto "Criar Otimização" (quando `is-empty: true`), que ao ser clicado chama `root.show-create-modal = true; root.create-modal-error = "";`.

### 4.2. Efeito Cascata de Atualização da UI (Refresh Flow)
Quando o usuário clica em "Otimizar Automaticamente":
1. `root.auto-optimize-clicked()` dispara o callback do backend.
2. `core.save_autodetect_config(None)` grava `AutoDetect.json` em `config/optimzations/`.
3. `ui.set_presets(updated_presets)` atualiza o modelo de dados Slint:
   - Imediatamente, `root.presets.length` passa de 0 para 1.
   - A condição de estado vazio `root.presets.length == 0` deixa de ser satisfeita.
   - A interface renderiza reativamente a seção com presets (`PresetButton` ativo de `AutoDetect` e botão `AddPresetButton` no modo compacto).
4. `ui.set_active_preset("AutoDetect".into())` ativa a condição `root.active-preset != ""`:
   - O campo de input de parâmetros de inicialização torna-se visível imediatamente acima dos botões de seleção de perfil.
   - Preenche com os parâmetros recomendados para a categoria de hardware detectada.
5. Os 13 setters `ui.set_*_tweak_active(...)` atualizam todos os switches na tela para o estado calibrado.
6. `ui.set_status_message(...)` atualiza a barra de status HUD no rodapé.

---

## 5. Mapeamento das Mudanças Necessárias de UI (Requisitos R1, R2, R3, R4)

### 5.1. R2 — Remoção do Botão Inferior de Otimização Automática
Em `ui/main.slint:891-918`:
- **Atualmente:** Existe um bloco `VerticalLayout` com um `TacticalButton` ("OTIMIZAR AUTOMATICAMENTE") centralizado logo acima da Barra de Status HUD.
- **Mudança:** Remover o `HorizontalLayout` com o `TacticalButton`, mantendo exclusivamente a Barra de Status HUD Tática na seção inferior.

### 5.2. R3 — Reposicionamento do Input de Parâmetros de Inicialização
Em `ui/main.slint:770-888`:
- **Atualmente:** O campo de parâmetros de inicialização está localizado após os botões de seleção de perfis de otimização.
- **Mudança:** Mover o bloco `if root.active-preset != "" : HorizontalLayout { ... }` para cima, ficando posicionado **diretamente abaixo da grade de otimizações ativas (13 módulos)** e **antes do cabeçalho/divisória "PERFIS DE OTIMIZAÇÃO (CONFIG/OPTIMZATIONS)"**.

### 5.3. R1 — Estado Sem Arquivos `.json`
- **Componente `AddPresetButton` (`ui/components/add_preset_button.slint:32-40`):**
  - Alterar o texto de `"NOVO PERFIL"` para `"Criar Otimização"`.
  - Ajustar a largura `width: root.is-empty ? 190px : 44px;` para acomodar com folga o novo texto.
- **Na Seção de Perfis (`ui/main.slint:741-767`):**
  - Se `root.presets.length == 0`:
    Renderizar exclusivamente:
    1. Botão "Otimizar Automaticamente":
       Pode ser um botão com largura de ~220px, altura de 44px (igual ao `AddPresetButton`), bordas e paleta com tema CS2Theme, ícone de foguete ou similar, e evento `clicked => { root.auto-optimize-clicked(); }`.
    2. Botão `AddPresetButton { is-empty: true; clicked => { ... } }` que renderiza `+ Criar Otimização`.
  - Se `root.presets.length > 0`:
    Renderizar a lista existente de `PresetButton`s em loop `for preset in root.presets` + o `AddPresetButton { is-empty: false; ... }`.

### 5.4. R4 — Preservação do Fluxo Existente com Arquivos `.json`
- Quando existem arquivos na pasta `config/optimzations`, a lista exibe os botões de perfis (`PresetButton`), o botão `+` compacto para criar novos perfis, o campo de parâmetros de inicialização editável e salvável, e os 13 switches continuam totalmente funcionais.

---

## 6. Conclusão Técnica

Todas as peças de backend já estão completamente implementadas e validadas:
- A geração e calibração por hardware via `OptimizerCore::save_autodetect_config` funciona perfeitamente.
- O callback `on_auto_optimize_clicked` já realiza todo o ciclo de criação, persistência, leitura e atualização de estado reativo da interface.
- A thread de monitoramento em segundo plano garante a consistência se arquivos forem adicionados ou excluídos manualmente no disco.
- As mudanças necessárias concentram-se estritamente na reorganização visual e declaração de componentes no arquivo `ui/main.slint` e no ajuste de texto do `ui/components/add_preset_button.slint`.
