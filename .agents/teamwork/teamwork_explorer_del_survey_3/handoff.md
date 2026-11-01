# Relatório de Investigação — Casos de Borda, Transição de Estado e Comandos de Build/Teste (Survey 3)

## 1. Observation (Observações Diretas)

### 1.1. Detecção do Estado Vazio na UI Slint
No arquivo `ui/main.slint`:
- **Propriedades Reativas do MainWindow** (linhas 43-48):
  ```slint
  in-out property <string> active-preset: "";
  in-out property <[PresetModel]> presets: [];
  in-out property <bool> show-create-modal: false;
  in-out property <string> create-modal-error: "";
  in-out property <string> launch-args-text: "";
  in-out property <bool> is-editing-launch-args: false;
  ```
- **Condição de Exibição dos Parâmetros de Inicialização** (linha 709):
  ```slint
  if root.active-preset != "" : HorizontalLayout { ... }
  ```
  *Quando `active-preset` é string vazia `""`, o campo de texto de parâmetros e seu botão de salvar/editar são completamente ocultados.*
- **Renderização Exclusiva do Estado Vazio (Sem Perfis)** (linhas 830-858):
  ```slint
  if root.presets.length == 0 : HorizontalLayout {
      alignment: center;
      spacing: 16px;

      TacticalButton {
          text: "OTIMIZAR AUTOMATICAMENTE";
          badge: "PERFIL RECOMENDADO • BAIXA LATÊNCIA";
          enabled: !root.is-launching;
          background-color: CS2Theme.accent-orange;
          ...
          clicked => {
              root.auto-optimize-clicked();
          }
      }

      AddPresetButton {
          is-empty: true;
          clicked => {
              root.create-modal-error = "";
              root.show-create-modal = true;
          }
      }
  }
  ```
  *Quando `root.presets.length == 0`, a UI renderiza exclusivamente os dois botões centrais: "OTIMIZAR AUTOMATICAMENTE" e "Criar Otimização +".*
- **Renderização da Lista de Perfis Existentes** (linhas 861-920):
  ```slint
  if root.presets.length > 0 : VerticalLayout {
      ...
      for preset in root.presets : PresetButton {
          file-name: preset.name;
          tag: preset.tag;
          is-active: root.active-preset == preset.name;
          ...
      }
  ```
  *Esta seção inteira só existe quando `root.presets.length > 0`.*

### 1.2. Inicialização do Estado Neutro Vazio no Backend Rust
No arquivo `ui/src/main.rs` (linhas 31-50):
```rust
// Se não existirem perfis salvos no disco, inicia no estado neutro desligado (R1)
let (resolved_path, _config) = if initial_presets.is_empty() {
    main_window.set_active_preset("".into());
    main_window.set_launch_args_text("".into());
    main_window.set_is_editing_launch_args(false);
    main_window.set_cpu_tweak_active(false);
    main_window.set_timer_tweak_active(false);
    main_window.set_priority_tweak_active(false);
    main_window.set_ram_tweak_active(false);
    main_window.set_gpu_accel_tweak_active(false);
    main_window.set_dpi_tweak_active(false);
    main_window.set_mem_compression_tweak_active(false);
    main_window.set_mmcss_tweak_active(false);
    main_window.set_hpet_tweak_active(false);
    main_window.set_game_bar_tweak_active(false);
    main_window.set_msi_tweak_active(false);
    main_window.set_power_saving_tweak_active(false);
    main_window.set_render_queue_tweak_active(false);
    (std::path::PathBuf::new(), counterrush_core::config::CustomOptimizationConfig::all_disabled())
} else { ... }
```
E em `config/src/lib.rs` (linhas 60-78):
`CustomOptimizationConfig::all_disabled()` desativa todos os 13 tweaks booleanos e inicializa `launch_args` vazio.

### 1.3. Mecanismo de Varredura e Atualização de Presets na UI
No arquivo `ui/src/main.rs`:
- **Função `scan_presets()`** (linhas 5-21):
  Chama `counterrush_core::list_optimization_configs()` (que lê e ordena todos os arquivos `.json` da pasta `config/optimzations/`) e mapeia para `Vec<PresetModel>`.
- **Atualização na Criação de Perfil** (linhas 602-604):
  ```rust
  let updated_presets = scan_presets();
  ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
  ```
- **Atualização na Otimização Automática** (linhas 477-478):
  ```rust
  let updated_presets = scan_presets();
  ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
  ```
- **Tratamento de Exclusão de Arquivos na Thread de Sincronização em Segundo Plano** (linhas 106-137):
  ```rust
  if names_clone.is_empty() {
      ui.set_active_preset("".into());
      ui.set_launch_args_text("".into());
      ui.set_is_editing_launch_args(false);
      if let Ok(mut cur) = sync_active_ref.write() {
          *cur = std::path::PathBuf::new();
      }
  } else {
      let current_active = ui.get_active_preset().to_string();
      if !names_clone.contains(&current_active) {
          let first_preset = &names_clone[0];
          let target = counterrush_core::resolve_optimization_file_path(first_preset);
          if let Ok(mut cur) = sync_active_ref.write() {
              *cur = target.clone();
          }
          ui.set_active_preset(first_preset.as_str().into());
          if let Ok(cfg) = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target) {
              ui.set_launch_args_text(cfg.launch_args.join(" ").into());
          }
          ui.set_is_editing_launch_args(false);
      }
  }
  ```

### 1.4. Comandos de Compilação, Teste e Hot Reload
- `Cargo.toml`: Workspace com os membros `core`, `windows_api`, `tweaks`, `config`, `ui`.
- `cargo check --workspace`: Executado com sucesso (`Finished dev profile [unoptimized + debuginfo] target(s) in 0.99s`).
- `cargo test -p core -- --list`: Executado com sucesso, listando 7 testes unitários em `core/src/lib.rs`:
  - `tests::test_autodetect_config_generation_and_no_custom_optimization`
  - `tests::test_autodetect_evaluation_respects_hardware_limits`
  - `tests::test_create_and_persist_new_profile`
  - `tests::test_isolated_build_dir_does_not_escape_to_project_dir`
  - `tests::test_list_optimization_configs_empty_directory`
  - `tests::test_profile_name_validation`
  - `tests::test_resolve_autodetect_config_path_edge_cases`
- `slint-viewer --version`: Slint Viewer 1.18.1 instalado.
- `preview.cmd`: Script nativo na raiz do projeto executando `slint-viewer ui/main.slint -I ui --auto-reload %*`.

---

## 2. Logic Chain (Cadeia Lógica de Raciocínio)

### 2.1. Como garantir o retorno exato ao estado padrão vazio (Requisito Crítico 1)
1. **Premissa UI (Observação 1.1):** A interface Slint chaveia o modo visual baseado em:
   - `root.presets.length == 0`: exibe exclusivamente os dois botões "OTIMIZAR AUTOMATICAMENTE" e "Criar Otimização +".
   - `root.active-preset != ""`: exibe o bloco de parâmetros.
2. **Premissa Backend (Observação 1.2):** O estado padrão vazio é formalmente definido no projeto pela inicialização quando `initial_presets.is_empty()`, requerindo:
   - `root.presets`: `slint::ModelRc::new(slint::VecModel::default())` (comprimento 0).
   - `root.active-preset`: `"".into()` (desmonta a barra de parâmetros).
   - `root.launch-args-text`: `"".into()` (limpa os argumentos).
   - `root.is-editing-launch-args`: `false`.
   - `active_config_path`: `std::path::PathBuf::new()` (impede tentativas de salvar em arquivo inexistente).
   - **Todos os 13 switches de tweaks definidos para `false`:** Caso um perfil excluído estivesse com switches ativos (ex: `cpu_core_0`, `timer_resolution`), deixá-los ligados quando não há nenhum perfil ativo criaria um estado órfão e inconsistente. A função `CustomOptimizationConfig::all_disabled()` reflete esse estado padrão neutro.
3. **Conclusão para o Caso Vazio:** O callback de exclusão no Rust deve, ao detectar que a lista de presets resultante ficou vazia (`remaining_presets.is_empty()`), executar exatamente esses resets síncronos na UI Slint e no ponteiro de configuração.

### 2.2. Exclusão de Perfil Ativo vs Inativo (Pergunta 2)
1. **Caso Perfil Inativo Excluído (e restam outros perfis):**
   - O perfil ativo atual não foi afetado.
   - Sua configuração continua válida e em vigor.
   - Nenhuma alteração deve ser feita em `root.active-preset`, `root.launch-args-text`, `active_config_path` ou nos 13 switches.
   - Apenas `scan_presets()` é chamado e `root.presets` é atualizado via `ModelRc` para remover visualmente o botão do perfil deletado.
2. **Caso Perfil Ativo Excluído (e ainda restam outros perfis):**
   - O perfil ativo que estava selecionado deixou de existir no disco.
   - Deixar `root.active-preset` com o nome do perfil excluído causará erro de I/O em qualquer ajuste subsequente de switch ou argumento.
   - Deixar `root.active-preset = ""` enquanto restam perfis violaria o padrão R4, que mantém a seleção ativa do primeiro perfil disponível (`initial_presets[0]`).
   - A thread de sincronização já estabeleceu o padrão de fallback em `ui/src/main.rs:126-136`:
     - Seleciona automaticamente o primeiro perfil restante (`&remaining_presets[0].name`).
     - Carrega sua configuração (`load_from_file`).
     - Atualiza `root.active-preset`, `root.launch-args-text`, os 13 switches de tweaks e `active_config_path`.
     - Isso garante transição fluida, sem falhas de referência nula ou estado desincronizado.
3. **Caso Perfil Ativo Excluído (e não resta nenhum perfil):**
   - Aplica a lógica da Seção 2.1 (retorno ao estado neutro vazio).

### 2.3. Sincronização de Lista de Perfis (Pergunta 3)
1. Conforme observado em `ui/src/main.rs:602-604` e `ui/src/main.rs:477-478`, a reatividade do Slint responde instantaneamente à substituição do `ModelRc` associado a `root.presets`.
2. A exclusão disparada pelo modal da UI deve executar a deleção no sistema de arquivos e a chamada `ui.set_presets(ModelRc::new(VecModel::from(scan_presets())))` **diretamente dentro do callback Rust da UI**.
3. Não se deve esperar a thread em segundo plano (que roda com intervalo de 1.5s), eliminando qualquer atraso perceptível de interface.

### 2.4. Comandos e Ferramentas (Pergunta 4)
1. O comando `cargo check --workspace --examples` valida sintaxe e tipagem de todo o projeto (Rust + compilação Slint via `slint-build`).
2. O utilitário `slint-viewer` (com script `preview.cmd`) fornece o hot reload oficial para a interface Slint (`ui/main.slint`), atendendo à regra `.agent/rules/relus.md` para inspeção de UI.
3. O comando `cargo test -p core` executa os testes unitários do subsistema de perfis e configurações.

---

## 3. Caveats (Ressalvas e Limitações)

1. **Ícone de Lixeira (`trash.svg`):**
   - No diretório `ui/assets/icons/`, atualmente não existe um ícone dedicado de lixeira (`trash.svg`). Para a implementação, um arquivo SVG vetorial limpo de lixeira tática precisará ser criado em `ui/assets/icons/trash.svg` ou incorporado como SVG inline.
2. **Propagação de Cliques em Botões Slint:**
   - No `PresetButton` (`ui/components/preset_button.slint`), o `TouchArea` cobre atualmente toda a área do card (`clicked => { root.selected(); }`).
   - Ao posicionar o botão com o ícone de lixeira, ele deve ter seu próprio `TouchArea` ou ser isolado para que o clique na lixeira **não dispare acidentalmente** a seleção/ativação do perfil (`root.selected()`), mas apenas a abertura do modal de confirmação.
3. **Privilégios de Execução no Terminal:**
   - Em terminais Windows dentro do ambiente sandbox, caminhos de unidade virtual `G:\` podem exigir execução sem sandbox (`BypassSandbox: true`).
4. **Regras de Modificação do Projeto (.agent/rules/relus.md):**
   - Este agente é estritamente de investigação (READ-ONLY). Nenhum código de produção foi alterado. Qualquer execução de testes automatizados ou alterações de código exigirá plano prévio aprovado pelo usuário.

---

## 4. Conclusion (Conclusão e Diretrizes para Implementação)

1. **Estado Vazio Conforme Solicitado:**
   - O Slint já possui a reatividade completa estruturada: quando `root.presets.length == 0`, a seção com lista de perfis é desmontada e os dois botões "OTIMIZAR AUTOMATICAMENTE" e "Criar Otimização +" aparecem automaticamente.
   - Para garantir retorno perfeito ao estado padrão, a rotina de exclusão deve:
     - Chamar `std::fs::remove_file(&target_path)` (preferencialmente encapsulado em função do `core::delete_optimization_config`).
     - Chamar `scan_presets()`.
     - Atualizar `ui.set_presets(...)`.
     - Se `presets.is_empty()`:
       - `ui.set_active_preset("".into())`
       - `ui.set_launch_args_text("".into())`
       - `ui.set_is_editing_launch_args(false)`
       - Setar todos os 13 switches para `false` (`all_disabled()`)
       - Limpar `active_config_path` (`PathBuf::new()`)
       - `ui.set_status_message("Perfil '{nome}' excluído com sucesso. Nenhum perfil restante.".into())`
2. **Exclusão de Perfil Ativo vs Inativo:**
   - Se o perfil excluído for **inativo**: nada muda na seleção atual; apenas o botão do perfil é removido da lista.
   - Se o perfil excluído for o **ativo** e ainda restarem outros: seleciona automaticamente o primeiro perfil restante (`remaining_presets[0]`), carrega seu `.json`, atualiza `active-preset`, `launch-args-text`, `active_config_path` e os 13 switches de otimização para coincidir com o novo perfil ativo.
3. **Modal de Confirmação:**
   - Requer novo componente Slint (ex: `ui/components/delete_profile_dialog.slint` ou integrado em `main.slint`), seguindo o padrão já estabelecido em `CreateProfileDialog`:
     - Overlay com fundo escuro semi-transparente bloqueando cliques traseiros.
     - Card central tático com o texto exato requerido: `"Deseja realmente excluir o perfil {Nome do perfil}?"`.
     - Botões "Cancelar" (fecha o modal) e "Excluir" (dispara o callback Rust para exclusão definitiva).

---

## 5. Verification Method (Método de Verificação Independente)

Para que qualquer agente ou usuário possa verificar independentemente os achados:

1. **Verificação de Compilação Geral:**
   ```powershell
   cd "g:\Outros computadores\Meu laptop\Desktop\CounterRush"
   cargo check --workspace --examples
   ```
   *Condição de invalidação: qualquer erro de sintaxe ou dependência não resolvida.*

2. **Verificação dos Testes Unitários de Presets:**
   ```powershell
   cargo test -p core -- --list
   ```
   *Verificar que os 7 testes de configuração e perfis listados existem e compilam.*

3. **Verificação do Hot Reload Slint:**
   ```powershell
   .\preview.cmd
   ```
   *Abre a janela do Slint Viewer com auto-reload ativo apontando para `ui/main.slint`.*

4. **Inspeção de Código Estático:**
   - `ui/main.slint:830-858`: inspecionar a cláusula `if root.presets.length == 0` para confirmar os botões do estado vazio.
   - `ui/main.slint:709`: inspecionar a cláusula `if root.active-preset != ""` para confirmar a dependência dos parâmetros de inicialização.
   - `ui/src/main.rs:31-50`: inspecionar a definição do estado inicial neutro quando `initial_presets.is_empty()`.
   - `ui/src/main.rs:91-142`: inspecionar o tratamento de remoção de arquivos e seleção de fallback.
