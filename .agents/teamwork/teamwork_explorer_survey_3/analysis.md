# Análise Técnica Detalhada — Explorer 3
**Foco:** Input de Parâmetros de Inicialização, Otimizações Ativas, Seleção de Arquivos e Interação de Layout/Estados (R1, R2, R3, R4)
**Data:** 2026-11-01
**Autor:** Explorer 3 (`teamwork_explorer_survey_3`)

---

## 1. Localização e Implementação Atual do Input de Parâmetros de Inicialização

### 1.1 Localização no Código Slint
- **Arquivo:** `ui/main.slint`
- **Linhas:** 770 a 888 (dentro do contêiner `root.active-tab == 1`, linhas 418 a 942).
- **Posição Atual na Árvore de UI:** Está dentro do bloco `VerticalLayout` intermediário (linhas 709-889), renderizado **diretamente abaixo** da linha horizontal de seleção de perfis (`PresetButton`s, linhas 741-767).
- **Condição de Renderização:**
  ```slint
  if root.active-preset != "" : HorizontalLayout {
      alignment: center;
      spacing: 10px;
      ...
  }
  ```
  *(Se não houver perfil ativo selecionado, o campo não é renderizado).*

### 1.2 Componentes Visuais e Estilização
O bloco do input é composto por um `HorizontalLayout` com dois elementos principais:
1. **Caixa do Campo de Texto (`Rectangle`, linhas 774-844):**
   - **Dimensões:** `width: 722px; height: 44px;`
   - **Fundo:** Reativo ao modo de edição: `root.is-editing-launch-args ? CS2Theme.bg-surface (#111824) : CS2Theme.bg-card (#162132);`
   - **Borda:** `border-radius: CS2Theme.radius-sm (4px);`
   - **Espessura da borda:** `root.is-editing-launch-args ? (launch-input.has-focus ? 2px : 1px) : 1px;`
   - **Cor da borda:** Reativa ao foco do input:
     - Com foco: `CS2Theme.accent-orange (#ff6a00)`
     - Em edição sem foco: `CS2Theme.border-hover (#3b5275)`
     - Somente leitura: `CS2Theme.border (#24334a)`
   - **Badge Identificador Tático (`Rectangle`, linhas 791-815):**
     - Dimensões: `width: 88px; height: 22px; border-radius: CS2Theme.radius-xs (2px);`
     - Background: `root.is-editing-launch-args ? CS2Theme.accent-orange-soft : CS2Theme.bg-surface;`
     - Texto: `"PARÂMETROS"` (`font-size: 8px; font-weight: 900; letter-spacing: 0.6px;`)
   - **Widget `TextInput` (`launch-input`, linhas 817-832):**
     - Binding bidirecional de valor: `text <=> root.launch-args-text;`
     - Modo somente leitura: `read-only: !root.is-editing-launch-args;`
     - Cor do texto: `root.is-editing-launch-args ? CS2Theme.text-primary (#f8fafc) : CS2Theme.text-muted (#94a3b8);`
     - Tipografia: `font-size: 11px; font-weight: 600; single-line: true; vertical-alignment: center;`
     - Evento `accepted =>`: ao pressionar `Enter`, se `is-editing-launch-args == true`, encerra a edição (`root.is-editing-launch-args = false;`) e invoca `root.save-launch-args-clicked(root.launch-args-text);`.
   - **Placeholder Customizado (`Text`, linhas 835-843):**
     - Renderizado condicionalmente se `root.launch-args-text == ""`.
     - Texto exibido: `"Ex: -novid -high +fps_max 0"` (se `is-editing-launch-args`) ou `"Nenhum parâmetro de inicialização configurado"` (se bloqueado).
     - Cor: `CS2Theme.text-dim (#64748b)`.

2. **Botão de Alternância Edição / Salvamento (`Rectangle`, linhas 846-887):**
   - **Dimensões:** `width: 44px; height: 44px;` (quadrado alinhado à altura da caixa de texto).
   - **Largura total combinada:** `722px + 10px (spacing) + 44px = 776px`.
   - **Fundo:** `edit-touch.has-hover ? CS2Theme.bg-card-hover : CS2Theme.bg-card;`
   - **Ícone Dinâmico (`Image`, linhas 859-871):**
     - Modo Edição (`is-editing-launch-args == true`): `@image-url("assets/icons/save.svg")` (ícone de disquete).
     - Modo Visualização (`is-editing-launch-args == false`): `@image-url("assets/icons/pencil.svg")` (ícone de lápis).
     - Cor: `CS2Theme.accent-orange` ou `CS2Theme.accent-orange-bright` ao hover.
   - **Comportamento do Clique (`TouchArea`, linhas 873-886):**
     - Ao clicar em modo visualização: ativa edição (`root.is-editing-launch-args = true;`), exibe mensagem HUD e transfere foco para o campo (`launch-input.focus();`).
     - Ao clicar em modo edição: desativa edição (`root.is-editing-launch-args = false;`), exibe status HUD e invoca o callback `root.save-launch-args-clicked(root.launch-args-text);`.

### 1.3 Backend Rust (Binding, Estado e Salvamento)
- **Arquivo:** `ui/src/main.rs`
- **Variáveis de Estado na MainWindow:**
  - `launch-args-text: string` (`main_window.set_launch_args_text(...)` / `get_launch_args_text()`)
  - `is-editing-launch-args: bool` (`main_window.set_is_editing_launch_args(...)` / `get_is_editing_launch_args()`)
- **Ponteiro de Caminho:** `active_config_path` (`Arc<RwLock<PathBuf>>`), que rastreia o arquivo `.json` ativo em `config/optimzations/`.
- **Callback de Salvamento (`ui/src/main.rs:636-671`):**
  ```rust
  main_window.on_save_launch_args_clicked(move |args_str| {
      if let Some(ui) = window_handle.upgrade() {
          let target_path = if let Ok(guard) = save_config_ref.read() {
              guard.clone()
          } else {
              return;
          };
          if target_path.as_os_str().is_empty() {
              ui.set_status_message("Crie ou selecione um perfil para salvar alterações.".into());
              return;
          }
          let mut config = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
              .unwrap_or_default();
          let new_args: Vec<String> = args_str.as_str().split_whitespace().map(|s| s.to_string()).collect();
          config.launch_args = new_args;
          match config.save_to_file(&target_path) {
              Ok(()) => {
                  let file_stem = target_path.file_stem().and_then(|s| s.to_str()).unwrap_or("perfil");
                  ui.set_status_message(format!("Parâmetros de inicialização salvos no perfil '{file_stem}'.").into());
                  ui.set_launch_args_text(config.launch_args.join(" ").into());
              }
              Err(err) => {
                  ui.set_status_message(format!("Falha ao salvar parâmetros: {err}").into());
              }
          }
      }
  });
  ```
- **File Watcher Thread (`ui/src/main.rs:165-177`):** Caso o arquivo `.json` seja alterado externamente, a thread sincroniza o conteúdo para `launch-args-text`, desde que `!ui.get_is_editing_launch_args()` (evitando sobrescrever enquanto o usuário digita).

---

## 2. Seção de "Otimizações Ativas" na UI

### 2.1 Localização
- **Arquivo:** `ui/main.slint`, linhas 426 a 706.
- **Contexto:** Seção superior de `if root.active-tab == 1 : VerticalLayout`.

### 2.2 Estrutura e Elementos Renderizados
A seção de "Otimizações Ativas" contém:
1. **Cabeçalho Tático (linhas 430-446):**
   - Título: `"CENTRAL DE OTIMIZAÇÕES"` (16px, 900, `CS2Theme.text-primary`).
   - Badge descritivo: `"13 Módulos de Baixa Latência"` (11px, 800, `CS2Theme.status-success-bright`).
2. **Grade Modular de Otimizações (linhas 450-705):**
   - Composta por 4 linhas horizontais de botões táteis (`TweakSwitch`), espaçadas verticalmente em 14px (`spacing: 14px;`):
     - **Linha 1 (linhas 453-515):**
       1. `Núcleo 0 Off` (`cpu-tweak-active`) — Afinidade de CPU.
       2. `Timer 0.5ms` (`timer-tweak-active`) — Temporizador global de 0.5ms.
       3. `Prioridade Alta` (`priority-tweak-active`) — Prioridade do processo CS2.
       4. `Limpar RAM` (`ram-tweak-active`) — Esvaziamento de lista de páginas em standby.
     - **Linha 2 (linhas 518-580):**
       5. `GPU Apps Off` (`gpu-accel-tweak-active`) — Aceleração de hardware em apps secundários.
       6. `Otimização DPI` (`dpi-tweak-active`) — Tela cheia exclusiva e DPI sem escala.
       7. `Compactar RAM` (`mem-compression-tweak-active`) — Desativa compactação de memória do Windows.
       8. `Agendador MMCSS` (`mmcss-tweak-active`) — Otimização de prioridade multimídia no registro.
     - **Linha 3 (linhas 583-645):**
       9. `HPET Off` (`hpet-tweak-active`) — Desativação do HPET no BCD.
       10. `Game Bar Off` (`game-bar-tweak-active`) — Desativação da Xbox Game Bar e GameDVR.
       11. `Modo MSI` (`msi-tweak-active`) — Modo de interrupção PCIe baseada em mensagens.
       12. `Power Save Off` (`power-saving-tweak-active`) — Desativação de PCIe ASPM e economia USB.
     - **Linha 4 (linhas 648-705):**
       13. `Render Queue` (`render-queue-tweak-active`) — Fila de renderização (FlipQueue AMD / Persistence Mode NVIDIA).
       14. `MasterToggleSwitch` (linhas 666-687) — Botão mestre de ativação/desativação simultânea de todos os 13 tweaks (`is-all-tweaks-active`).
       15 e 16. Dois retângulos transparentes de 188px de largura para balancear visualmente a 4ª coluna da grade.
- **Dimensões:** Cada botão possui `button-width: 188px; height: 50px;`. Largura total da grade: `4 * 188px + 3 * 14px = 794px`, preenchendo exatamente a largura útil da tela.

---

## 3. Seção de Seleção de Arquivos de Otimização

### 3.1 Localização
- **Arquivo:** `ui/main.slint`, linhas 709 a 767.
- **Contexto:** Seção intermediária abaixo da grade de otimizações.

### 3.2 Estrutura e Elementos Renderizados
1. **Separador com Rótulo (linhas 713-739):**
   - Linhas horizontais sutis laterais (`Rectangle { width: 18px; height: 1px; background: CS2Theme.border; }`).
   - Texto central: `"PERFIS DE OTIMIZAÇÃO (CONFIG/OPTIMZATIONS)"` (`font-size: 10px; font-weight: 800; letter-spacing: 0.8px; color: CS2Theme.text-dim;`).
2. **Lista Horizontal de Perfis (linhas 741-767):**
   - Contêiner: `HorizontalLayout` com alinhamento centralizado. Espaçamento adaptativo: `root.presets.length > 2 ? 10px : 16px;`.
   - Cards de perfis existentes:
     ```slint
     for preset in root.presets : PresetButton {
         file-name: preset.name;
         tag: preset.tag;
         is-active: root.active-preset == preset.name;
         accent-color: preset.name == "AutoDetect" ? CS2Theme.accent-ct-cyan : CS2Theme.accent-orange;
         active-bg: preset.name == "AutoDetect" ? CS2Theme.accent-ct-soft : CS2Theme.accent-orange-soft;
         button-width: root.presets.length > 2 ? 250px : 350px;
         selected => {
             root.active-preset = preset.name;
             root.is-editing-launch-args = false;
             root.status-message = "Perfil ativo: " + preset.name;
             root.preset-selected(preset.name);
         }
     }
     ```
   - Botão de Criação / Adição:
     ```slint
     AddPresetButton {
         is-empty: root.presets.length == 0;
         clicked => {
             root.create-modal-error = "";
             root.show-create-modal = true;
         }
     }
     ```
- **Comportamento do `AddPresetButton` (`ui/components/add_preset_button.slint`):**
  - Quando `is-empty == false` (existem perfis): largura de 44px, exibe apenas `+`.
  - Quando `is-empty == true` (sem perfis): largura de 180px, exibe `+` e o texto `"NOVO PERFIL"`.

---

## 4. Análise e Proposta Exata de Reposicionamento do Input (R3)

### 4.1 Hierarquia Atual vs Hierarquia Solicitada
- **Hierarquia Atual:**
  1. Grade de Otimizações Ativas (13 módulos) [linhas 426-706]
  2. Divisor e Seleção de Arquivos de Otimização (`PresetButton`s + `AddPresetButton`) [linhas 709-767]
  3. **Input de Parâmetros de Inicialização (`launch-input`) [linhas 770-888]**
  4. Botão Inferior "OTIMIZAR AUTOMATICAMENTE" [linhas 896-917]
  5. Barra de Status HUD [linhas 920-941]

- **Hierarquia Solicitada por R3:**
  1. Grade de Otimizações Ativas (13 módulos)
  2. **Input de Parâmetros de Inicialização (`launch-input`)**
  3. Divisor e Seleção de Arquivos de Otimização (`PresetButton`s + `AddPresetButton`)
  4. Barra de Status HUD *(o botão inferior pré-existente "OTIMIZAR AUTOMATICAMENTE" é removido por R2)*

### 4.2 Proposta Exata de Mudança no Código Slint
Para reposicionar o input:
1. O bloco condicional de linhas 770-888 de `ui/main.slint`:
   ```slint
   // Campo de Parâmetros de Inicialização e Botão de Edição/Salvamento
   if root.active-preset != "" : HorizontalLayout {
       alignment: center;
       spacing: 10px;
       Rectangle { /* Caixa TextInput com 722px */ }
       Rectangle { /* Botão Lápis/Salvar com 44px */ }
   }
   ```
   deve ser extraído de dentro do `VerticalLayout` da seção de perfis e inserido **imediatamente após a Grade de Otimizações** (após a linha 706) e **imediatamente antes do separador** `"PERFIS DE OTIMIZAÇÃO (CONFIG/OPTIMZATIONS)"` (linha 709).
2. **Cálculo de Layout e Altura da Janela:**
   - Altura total da janela: 620px (fixa).
   - Altura útil interna: ~586px.
   - Cabeçalho: ~30px.
   - Grade de Otimizações: 4 * 50px + 3 * 14px = 242px.
   - Input de Parâmetros: 44px + 10px spacing = 54px.
   - Seção de Perfis: ~65px.
   - Barra de Status HUD: ~18px.
   - **Espaço total consumido:** `30 + 242 + 54 + 65 + 18 = 409px`.
   - **Espaço livre restante:** `586 - 409 = ~177px`.
   - **Conclusão:** O reposicionamento cabe confortavelmente na janela sem nenhum corte de elemento. A remoção do botão inferior de 52px (R2) garante ampla folga visual e equilíbrio estético.

---

## 5. Avaliação dos Estados: Sem Arquivos .json vs Com Arquivos .json (R1 x R3)

### 5.1 Estado com Arquivos `.json` (`root.presets.length > 0`)
- Pelo menos um arquivo de configuração existe na pasta `config/optimzations/`.
- `root.presets.length >= 1`.
- `root.active-preset != ""` (ex: `"AutoDetect"`).
- **Comportamento dos Elementos:**
  1. Grade de Otimizações: visível, com os switches refletindo as preferências gravadas no perfil ativo.
  2. Input de Parâmetros de Inicialização: **visível**, renderizado diretamente abaixo da grade de otimizações e acima da seleção de perfis, exibindo os `launch_args` do perfil selecionado e permitindo edição e salvamento.
  3. Seção de Perfis: exibe o cabeçalho "PERFIS DE OTIMIZAÇÃO", os cards `PresetButton` para cada arquivo encontrado e o botão `AddPresetButton` em modo compacto (ícone `+`, 44px).
  4. Barra HUD: visível no rodapé com a mensagem tática de status.

### 5.2 Estado sem Arquivos `.json` (`root.presets.length == 0`)
- A pasta `config/optimzations/` está vazia.
- `scan_presets()` retorna vetor vazio (`[]`).
- `root.presets.length == 0`.
- `root.active-preset == ""` (string vazia).
- Backend define todos os 13 toggles de tweaks para `false` (`CustomOptimizationConfig::all_disabled()`).
- `root.launch-args-text == ""`.

#### Interação Crítica entre R1 e R3:
1. **Comportamento do Input de Parâmetros de Inicialização:**
   - O input possui a condição `if root.active-preset != ""`.
   - Como `root.active-preset` é `""`, **o input de parâmetros de inicialização NÃO É RENDERIZADO**.
   - Isso é absolutamente consistente: não há arquivo para onde gravar parâmetros, nem arquivo do qual carregar. O campo de input permanecerá oculto até que um perfil seja criado ou detectado.
2. **Interpretação do Requisito R1:**
   - O requisito R1 diz: *"Quando não existir nenhum arquivo de configuração `.json` no projeto, a aba de Otimizações deve exibir exclusivamente dois botões de ação: Um botão com o texto 'Otimizar Automaticamente' ... e o botão de adicionar pré-existente, agora com o símbolo `+` e o texto 'Criar Otimização'"*.
   - Duas abordagens arquiteturais são possíveis e foram analisadas:

| Abordagem | Descrição | Vantagens | Considerações |
|---|---|---|---|
| **Abordagem A (Recomendada — Ações Exclusivas na Seção de Perfis)** | A Grade de 13 switches permanece visível no topo (com switches desligados), o input de launch args fica oculto (`active-preset == ""`), e na área de seleção são exibidos exclusivamente os dois botões: "Otimizar Automaticamente" e "Criar Otimização +". | O usuário visualiza imediatamente todas as capacidades técnicas do app; preserva estabilidade visual ao transicionar de estado. | A grade de switches fica visível, embora inativa. |
| **Abordagem B (Empty State Integral da Aba)** | Se `presets.length == 0`, a grade inteira de 13 switches é ocultada na aba 1, exibindo no centro da tela unicamente os 2 botões de ação. | Atende no sentido mais literal e estrito à frase "a aba deve exibir exclusivamente dois botões". | Ao clicar em otimizar, a interface sofre uma transição brusca com o surgimento repentino de 13 switches. |

3. **Adaptação do `AddPresetButton` para R1:**
   - Em `ui/components/add_preset_button.slint`:
     - Alterar o texto de `"NOVO PERFIL"` para `"Criar Otimização"`.
     - Alterar a largura no modo vazio de `180px` para `210px` a `230px`, garantindo legibilidade perfeita do texto ao lado do símbolo `+`.
   - Ao ser clicado, já invoca o modal `CreateProfileDialog`, cumprindo o critério de aceitação de R1.
4. **Novo Botão "Otimizar Automaticamente" no Estado Vazio:**
   - Posicionado ao lado do `AddPresetButton` ("Criar Otimização +") em um `HorizontalLayout { alignment: center; spacing: 16px; }`.
   - Aciona diretamente o callback `root.auto-optimize-clicked()`, já implementado e funcional no backend Rust.
   - Assim que acionado, `counterrush_core` detecta o hardware, grava `AutoDetect.json`, recarrega a UI, `root.presets.length` passa a ser 1, `root.active-preset` passa a ser `"AutoDetect"`, e a UI transiciona reativamente para o estado completo (R4) com o input de parâmetros de inicialização visível sob a grade de otimizações.
