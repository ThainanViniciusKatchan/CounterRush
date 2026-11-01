# Relatório de Investigação Técnica — Explorer 2
**Missão**: Investigar a detecção de perfis de otimização e o mecanismo de navegação de abas no Slint para o botão "Jogar CS2".

---

## 1. Observation

### 1.1 Detecção de Existência de Perfis de Otimização

#### A. No Nível de Sistema de Arquivos (Disco)
- Os perfis de otimização residem no diretório resolvido por `resolve_optimizations_dir()` em `core/src/lib.rs` (linhas 1013-1035):
  ```rust
  // core/src/lib.rs:1023-1026
  let direct = PathBuf::from("config/optimzations");
  if direct.is_dir() {
      return direct;
  }
  ```
- O módulo `core` lista todos os arquivos `.json` válidos usando `list_optimization_configs_in_dir` (linhas 1037-1055) e `list_optimization_configs()` (linhas 1057-1059):
  ```rust
  // core/src/lib.rs:1057-1059
  pub fn list_optimization_configs() -> Vec<PathBuf> {
      list_optimization_configs_in_dir(resolve_optimizations_dir())
  }
  ```
  Se nenhum arquivo `.json` existir no diretório `config/optimzations`, essa função retorna um vetor vazio `Vec::new()`.

#### B. No Nível Backend Rust (`ui/src/main.rs`)
- Em `ui/src/main.rs` (linhas 5-22), a função `scan_presets()` é responsável por consultar o disco e converter os arquivos em modelos Slint `PresetModel`:
  ```rust
  // ui/src/main.rs:5-22
  fn scan_presets() -> Vec<PresetModel> {
      let files = counterrush_core::list_optimization_configs();
      let mut list = Vec::new();
      for p in files {
          if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
              ...
              list.push(PresetModel { name: stem.into(), tag: tag.into() });
          }
      }
      list
  }
  ```
- Na inicialização (linhas 28-50):
  ```rust
  let initial_presets = scan_presets();
  main_window.set_presets(slint::ModelRc::new(slint::VecModel::from(initial_presets.clone())));
  if initial_presets.is_empty() {
      main_window.set_active_preset("".into());
      main_window.set_launch_args_text("".into());
      ...
  }
  ```
- Em tempo de execução, a thread de sincronização em segundo plano (linhas 93-143) monitora o disco a cada 1.5s (`loop_count % 3 == 0`) e atualiza o modelo Slint `ui.set_presets(...)` via `invoke_from_event_loop`. As ações de exclusão (`on_delete_profile_confirmed`), criação (`on_create_profile_confirmed`) e calibração (`on_auto_optimize_clicked`) também re-escaneiam e atualizam `ui.set_presets(...)`.

#### C. No Nível de Interface Slint (`ui/main.slint`)
- Em `ui/main.slint` (linha 45):
  ```slint
  in-out property <[PresetModel]> presets: [];
  in-out property <string> active-preset: "";
  ```
- O Slint rastreia a existência de perfis de forma nativa e reativa através de `root.presets.length`:
  - `root.presets.length == 0`: Nenhum perfil de otimização existe.
  - `root.presets.length > 0`: Existem perfis cadastrados.
- Essa verificação é utilizada diretamente nas linhas 834 e 865 de `ui/main.slint`:
  ```slint
  // ui/main.slint:834
  if root.presets.length == 0 : HorizontalLayout { ... } // Estado vazio
  // ui/main.slint:865
  if root.presets.length > 0 : VerticalLayout { ... }   // Lista de perfis
  ```

---

### 1.2 Mecanismo de Navegação de Abas no Slint

#### A. Propriedade de Controle de Abas
- Em `ui/main.slint` (linha 28):
  ```slint
  in-out property <int> active-tab: 0;
  ```
  O controle da aba ativa é realizado exclusivamente por um índice numérico inteiro (`int`).

#### B. Componente Barra Lateral (`Sidebar`)
- Em `ui/components/sidebar.slint` (linhas 44-46 e 92-118):
  ```slint
  export component Sidebar inherits Rectangle {
      in-out property <int> active-tab: 0;
      callback tab-selected(int);
      ...
      NavButton {
          text: "Painel";
          icon-src: @image-url("../assets/icons/home.svg");
          active: root.active-tab == 0;
          clicked => { root.active-tab = 0; root.tab-selected(0); }
      }
      NavButton {
          text: "Otimizações";
          icon-src: @image-url("../assets/icons/rocket.svg");
          active: root.active-tab == 1;
          clicked => { root.active-tab = 1; root.tab-selected(1); }
      }
      NavButton {
          text: "Configurações";
          icon-src: @image-url("../assets/icons/settings.svg");
          active: root.active-tab == 2;
          clicked => { root.active-tab = 2; root.tab-selected(2); }
      }
      NavButton {
          text: "Ranking CS";
          icon-src: @image-url("../assets/icons/crown.svg");
          active: root.active-tab == 3;
          clicked => { root.active-tab = 3; root.tab-selected(3); }
      }
  }
  ```
- Em `ui/main.slint` (linhas 82-95), a `Sidebar` está conectada por ligação bidirecional (two-way binding):
  ```slint
  Sidebar {
      active-tab <=> root.active-tab;
      tab-selected(idx) => {
          if idx == 0 {
              root.status-message = "Visualizando Painel Geral";
          } else if idx == 1 {
              root.status-message = "Ajustes Avançados de Hardware";
          } else if idx == 2 {
              root.status-message = "Configurações Gerais";
          } else {
              root.status-message = "Tabela de Classificação Valve";
          }
      }
  }
  ```

#### C. Renderização Condicional da Área Central
- Em `ui/main.slint`, a exibição de cada aba é estritamente condicionada a `root.active-tab`:
  - Linha 98: `if root.active-tab == 0 : VerticalLayout { ... }` (Aba Painel de Controle)
  - Linha 422: `if root.active-tab == 1 : VerticalLayout { ... }` (Aba Central de Otimizações)
  - Linha 961: `if root.active-tab == 2 : VerticalLayout { ... }` (Aba Configurações Gerais)
  - Linha 1023: `if root.active-tab == 3 : VerticalLayout { ... }` (Aba Ranking CS)

---

### 1.3 Comportamento Atual do Botão "Jogar CS2"
- Em `ui/main.slint` (linhas 377-394), localizado dentro da aba Painel (`root.active-tab == 0`):
  ```slint
  TacticalButton {
      text: root.is-launching ? "INICIANDO..." : "JOGAR CS2";
      badge: root.is-launching ? "APLICANDO TWEAKS & STEAM" : "INICIAR COM ALTA PERFORMANCE";
      enabled: !root.is-launching;
      background-color: CS2Theme.accent-orange;
      ...
      clicked => {
          root.is-launching = true;
          root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
          root.launch-cs2-clicked();
      }
  }
  ```
- Atualmente, ao ser clicado, ele dispara incondicionalmente `root.is-launching = true` e o callback `launch-cs2-clicked()`, sem verificar se existem perfis (`root.presets.length == 0`).
- No backend (`ui/src/main.rs:231-284`), se `active_config_path` estiver vazio, `load_or_create_custom_config` tenta carregar ou criar um arquivo `custom_optimization.json` padrão ao invés de barrar e notificar o usuário na interface.

---

### 1.4 Estado da Aba "Otimizações" Quando Não Existem Perfis
- Em `ui/main.slint` (linhas 834-862), quando `root.presets.length == 0`:
  ```slint
  if root.presets.length == 0 : HorizontalLayout {
      alignment: center;
      spacing: 16px;

      TacticalButton {
          text: "OTIMIZAR AUTOMATICAMENTE";
          badge: "PERFIL RECOMENDADO • BAIXA LATÊNCIA";
          enabled: !root.is-launching;
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
- O componente `AddPresetButton` (`ui/components/add_preset_button.slint:33-41`) renderiza o botão com label `+ Criar Otimização`. Ao clicar, ativa `root.show-create-modal = true;`, abrindo o `CreateProfileDialog` já existente.
- A barra de parâmetros de inicialização fica oculta porque possui a trava `if root.active-preset != ""` (linha 713).
- Os 13 switches de otimização são exibidos, permitindo visualização de todos os módulos de latência.

---

## 2. Logic Chain

1. **Premissa de Detecção de Perfis**:
   - `core::list_optimization_configs()` inspeciona `config/optimzations/`. Se não houver arquivos `.json`, retorna vetor vazio.
   - `scan_presets()` em `ui/src/main.rs` mapeia esses arquivos para `Vec<PresetModel>`.
   - `MainWindow.presets` recebe esse modelo.
   - Logo, na interface Slint, a expressão booleana `root.presets.length == 0` reflete com 100% de exatidão e reatividade se existem ou não perfis de otimização disponíveis.

2. **Premissa do Botão "Jogar CS2"**:
   - O botão "Jogar CS2" está em `ui/main.slint` na seção `if root.active-tab == 0`.
   - Ao clicar no botão, pode-se avaliar a guarda reativa `if root.presets.length == 0`.
   - Se `root.presets.length == 0`, o sistema NÃO deve disparar o callback Rust `launch-cs2-clicked()`, NÃO deve definir `root.is-launching = true`, e DEVE abrir o novo modal definindo `root.show-no-profile-modal = true`.
   - Caso `root.presets.length > 0`, mantém o fluxo existente de inicialização.

3. **Premissa da Navegação de Abas e Transição**:
   - A propriedade `root.active-tab` é do tipo `int`, sendo `0` para "Painel" e `1` para "Otimizações".
   - Por possuir two-way binding (`active-tab <=> root.active-tab`), qualquer alteração em `root.active-tab`:
     - Altera imediatamente a visibilidade dos blocos condicionais da janela principal.
     - Atualiza o botão ativo destacado na `Sidebar`.
   - Portanto, a transição para a aba "Otimizações" ao clicar no botão "Criar Perfil" no novo modal pode ser realizada **puramente no Slint** com:
     ```slint
     root.show-no-profile-modal = false;
     root.active-tab = 1;
     root.status-message = "Ajustes Avançados de Hardware";
     ```
   - Não há necessidade de comunicação com o backend Rust para trocar abas, pois o gerenciamento de navegação no Slint já é 100% local no frontend.

4. **Premissa da Experiência do Usuário na Aba Otimizações**:
   - Ao chegar na aba Otimizações com `presets.length == 0`, a interface já renderiza automaticamente a seção vazia contendo `+ Criar Otimização` (que abre o modal `CreateProfileDialog`) e `OTIMIZAR AUTOMATICAMENTE` (que gera e ativa o perfil AutoDetect via detecção de hardware).
   - Isso garante uma experiência sem becos sem saída, dando ao usuário as opções ideais de criação manual ou calibração automática recomendada.

---

## 3. Caveats

1. **Abertura Automática do Modal de Criação vs. Apenas Navegar para a Aba**:
   - O requisito do usuário afirma textualmente: *"O botão criar perfil leva o usuário para a aba otimizações"*.
   - Levar para a aba otimizações (`root.active-tab = 1`) já expõe diretamente o botão "+ Criar Otimização" e o botão "OTIMIZAR AUTOMATICAMENTE".
   - Caso o orquestrador/usuário prefira que o formulário de criação de perfil (`CreateProfileDialog`) já abra aberto imediatamente ao trocar de aba, bastaria adicionar `root.show-create-modal = true;` no clique. No entanto, seguir estritamente o pedido de transição de aba (`root.active-tab = 1`) é mais limpo e permite ao usuário escolher entre criação manual ou autodetect.
2. **Defensividade no Backend Rust (`ui/src/main.rs`)**:
   - Embora o bloqueio no Slint impeça o acionamento de `launch-cs2-clicked` quando `presets.length == 0`, é uma boa prática adicionar uma checagem defensiva em `ui/src/main.rs` dentro de `on_launch_cs2_clicked` (`if ui.get_presets().row_count() == 0 { ui.set_show_no_profile_modal(true); ui.set_is_launching(false); return; }`), garantindo integridade contra chamadas acidentais.
3. **Consistência de Status Message**:
   - Ao trocar para a aba 1 via botão do modal, deve-se atualizar `root.status-message = "Ajustes Avançados de Hardware";` para manter a coerência visual com o comportamento da `Sidebar`.
4. **Isolamento de Cliques no Modal**:
   - Assim como em `DeleteProfileDialog` e `CreateProfileDialog`, o novo componente de diálogo deve possuir um `TouchArea` de bloqueio de fundo cobrindo `width: 980px; height: 620px;` para evitar que cliques no fundo transpassem para componentes da janela principal.

---

## 4. Conclusion

Respostas objetivas e estruturadas para as 5 perguntas da investigação:

| # | Pergunta | Resposta Conclusiva |
|---|---|---|
| **1** | Como a aplicação sabe se existem perfis de otimização? | **No Slint:** via propriedade reativa `root.presets.length == 0` (vazio) ou `> 0` (com perfis). **No Rust:** via `scan_presets().is_empty()` e `counterrush_core::list_optimization_configs()`. **No Disco:** pela presença/ausência de arquivos `.json` na pasta `config/optimzations/`. |
| **2** | Como a navegação de abas funciona em `ui/main.slint`? | Através da propriedade inteira `in-out property <int> active-tab: 0;` em `MainWindow`, vinculada bidirecionalmente com `Sidebar.active-tab`. A renderização é controlada por blocos `if root.active-tab == X`. |
| **3** | Qual é o identificador ou índice da aba "Painel" e "Otimizações"? | **Aba "Painel":** Índice `0`.<br>**Aba "Otimizações":** Índice `1`.<br>*(Aba 2 = Configurações, Aba 3 = Ranking CS)*. |
| **4** | Como o botão "Criar Perfil" no novo modal deve mudar a aba ativa para "Otimizações"? Isso pode ser feito puramente no Slint, ou precisa de callback Rust, ou ambos? | **Pode e deve ser feito puramente no Slint!** Basta atribuir `root.active-tab = 1;` e `root.status-message = "Ajustes Avançados de Hardware";` no clique do botão "Criar Perfil". Não requer nenhum callback Rust para troca de aba. Um guard no Rust em `on_launch_cs2_clicked` é recomendado apenas como defesa em profundidade. |
| **5** | O que acontece se o usuário estiver na aba otimizações sem nenhum perfil? Já existe tela/botão de criação nessa aba? | **Sim, já existe completamente implementado.** Quando `root.presets.length == 0`, a aba exibe dois botões de ação: `"OTIMIZAR AUTOMATICAMENTE"` (gera o perfil adaptativo AutoDetect) e `"+ Criar Otimização"` (`AddPresetButton { is-empty: true }`, que abre o `CreateProfileDialog`). |

### Especificação do Novo Componente Proposto
- **Arquivo**: `ui/components/no_profile_dialog.slint`
- **Dimensões do Card**: `460px x 190px`, centralizado, tema tático CS2 (`CS2Theme.bg-card`, borda `CS2Theme.accent-orange`).
- **Texto Requerido**: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`
- **Botões Requeridos**:
  1. `"Criar Perfil"` (estilo primário, acento laranja): fecha o modal, define `root.active-tab = 1;` e atualiza o status.
  2. `"Sair"` (estilo secundário/cancelar): fecha o modal com `root.show-no-profile-modal = false;`.

---

## 5. Verification Method

Para verificar e validar as constatações de forma independente:

1. **Inspeção de Código e Layout**:
   - Visualizar `ui/main.slint`: checar linhas 28 (`active-tab`), 45 (`presets`), 82-95 (`Sidebar`), 377-394 (`TacticalButton` "JOGAR CS2") e 834-862 (estado vazio de `presets.length == 0`).
   - Visualizar `ui/components/sidebar.slint`: checar linhas 92-118 para os índices 0, 1, 2 e 3 das abas.
   - Visualizar `ui/src/main.rs`: checar linhas 5-22 (`scan_presets()`) e 231-284 (`on_launch_cs2_clicked`).
2. **Checagem de Compilação do Workspace**:
   - Executar: `cargo check --workspace`
   - Deve compilar sem erros de sintaxe ou tipos.
3. **Verificação de Validação Slint**:
   - Inspecionar a compatibilidade dos tipos com `slint-viewer ui/main.slint` (caso disponível) ou via compilação do crate `ui` (`cargo check -p counterrush_ui`).
4. **Condição de Invalidação**:
   - Caso `root.active-tab = 1` não trocasse a aba visível na tela central ou não destacasse a `Sidebar`, nossa conclusão seria invalidada; todavia, a linha 83 `active-tab <=> root.active-tab` e as linhas 98 e 422 comprovam matematicamente a reatividade do Slint.
