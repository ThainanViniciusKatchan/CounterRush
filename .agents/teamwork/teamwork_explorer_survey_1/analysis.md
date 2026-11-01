# Relatório de Análise Técnica — Arquitetura da UI de Otimizações

**Data:** 2026-11-01  
**Agente:** Explorer 1 (`teamwork_explorer_survey_1`)  
**Workspace:** `g:\Outros computadores\Meu laptop\Desktop\CounterRush`  
**Escopo:** Investigação técnica e mapeamento da arquitetura da interface da aba de Otimizações para atendimento dos requisitos R1, R2, R3 e R4.

---

## 1. Resumo Executivo

A interface do CounterRush é desenvolvida em **Rust 1.75+** com a biblioteca de UI reativa **Slint 1.8**.  
A aba de Otimizações encontra-se inteiramente centralizada no arquivo `ui/main.slint` (linhas 418 a 942) sob a condição `if root.active-tab == 1`, suportada por componentes modulares em `ui/components/` e pelo runtime em `ui/src/main.rs`.

A análise identificou com precisão cirúrgica:
1. **O botão inferior "OTIMIZAR AUTOMATICAMENTE" a ser removido (R2)** está em `ui/main.slint:891-918`.
2. **A lista e seleção de perfis `.json` e o botão de adicionar (R1/R4)** estão em `ui/main.slint:741-767` e `ui/components/add_preset_button.slint`.
3. **A verificação da existência de arquivos `.json`** é feita reativamente via `root.presets.length == 0` (Slint) e `scan_presets().is_empty()` (Rust).
4. **O campo de input dos parâmetros de inicialização a ser reposicionado (R3)** está em `ui/main.slint:770-888`, atualmente renderizado *abaixo* dos botões de perfil, devendo ser transferido para *logo abaixo da grade de otimizações ativas* e *antes da seleção de arquivos*.
5. **A exibição exclusiva dos dois botões de ação ("Otimizar Automaticamente" e "+ Criar Otimização") no estado sem arquivos `.json` (R1)** foi detalhada com duas alternativas arquiteturais (Empty State da área de ações vs. Empty State da aba inteira), com especificação completa de propriedades, eventos e estilos visuais conforme o Design System CS2.

---

## 2. Mapeamento da Arquitetura da UI (Aba de Otimizações)

### 2.1 Estrutura de Arquivos e Componentes

| Arquivo | Papel na Arquitetura |
|---|---|
| `ui/main.slint` | **View Principal da Aplicação**. Declara o componente `MainWindow`, estado reativo global (`presets`, `active-preset`, `status-message`, flags dos 13 switches, callbacks) e o layout de todas as abas. Linhas 418–942 definem a aba de Otimizações. |
| `ui/theme.slint` | **Design System Tokens (`CS2Theme`)**. Cores primárias (`accent-orange`, `accent-ct-blue`, `bg-main`, `bg-surface`, `bg-card`), tipografia, bordas e raios (`radius-sm`, `radius-xs`). |
| `ui/components/sidebar.slint` | **Navegação Lateral**. `Sidebar` com `NavButton`s. O índice `active-tab: 1` ativa a aba "Ajustes Avançados de Hardware" / Otimizações. |
| `ui/components/tweak_switch.slint` | **Switches Modulares**. Contém `TweakSwitch` (usado 13 vezes na grade) e `MasterToggleSwitch` (botão de ligar/desligar tudo). |
| `ui/components/preset_button.slint` | **Botão de Perfil de Configuração**. Renderiza cada arquivo `.json` existente com indicador LED, tag e botão `ATIVO`/`USAR`. |
| `ui/components/add_preset_button.slint` | **Botão de Adicionar Perfil**. Atualmente com largura dinâmica (180px em vazio, 44px quando preenchido), símbolo `+` e texto "NOVO PERFIL". |
| `ui/components/tactical_button.slint` | **Botão Tático Primário**. Utilizado no botão inferior atual de auto-otimização (largura 350px, badge, ícone de foguete e destaque laranja). |
| `ui/components/create_profile_dialog.slint` | **Modal de Criação**. `CreateProfileDialog` exibido como overlay quando `show-create-modal == true`, com validação de nome, switches e parâmetros. |
| `ui/src/main.rs` | **Backend Rust**. Orquestrador de eventos, sincronização do disco (`config/optimzations/*.json`) com a UI via callbacks Slint e thread em background. |

### 2.2 Fluxo de Dados e Ciclo de Vida

```
[ Disco: config/optimzations/*.json ]
              ▲
              │ counterrush_core::list_optimization_configs()
              ▼
   [ Rust: scan_presets() em ui/src/main.rs ]
              │
              ▼ ui.set_presets(ModelRc<VecModel<PresetModel>>)
    [ Slint: root.presets em ui/main.slint ]
              │
    ┌─────────┴─────────────────────────────────┐
    ▼                                           ▼
presets.length == 0                     presets.length > 0
[ Estado Vazio (R1) ]                   [ Estado com Perfis (R4) ]
- 2 Botões de Ação Exclusivos           - Grade de 13 switches ativos
  1. "Otimizar Automaticamente"         - Input de Parâmetros (R3)
  2. "+ Criar Otimização"               - Lista de PresetButtons + AddPresetButton
```

---

## 3. Localização da Lista/Seleção de Arquivos .json e Detecção de Estado

### 3.1 Onde e Como é Carregado (Backend Rust)

- **Arquivo:** `ui/src/main.rs`
- **Linhas 5 a 22:**
  ```rust
  fn scan_presets() -> Vec<PresetModel> {
      let files = counterrush_core::list_optimization_configs();
      let mut list = Vec::new();
      for p in files {
          if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
              let tag = match stem {
                  "AutoDetect" => "HARDWARE ADAPTATIVO",
                  "custom_optimization" => "CUSTOMIZADO / MANUAL",
                  _ => "PERFIL DO USUÁRIO",
              };
              list.push(PresetModel {
                  name: stem.into(),
                  tag: tag.into(),
              });
          }
      }
      list
  }
  ```
- **Linhas 32 a 50:**
  Verifica se `initial_presets.is_empty()`. Se estiver vazio:
  - Define `active_preset` como `""`;
  - Define `launch_args_text` como `""`;
  - Desativa todos os 13 switches (`cpu_tweak_active`, `timer_tweak_active`, etc., todos `false`);
  - Define o caminho ativo como vazio.
- **Linhas 93 a 142:**
  Thread em background monitora modificações a cada 1.5s na pasta `config/optimzations`. Se arquivos forem criados ou deletados, reexecuta `scan_presets()` e sincroniza com a UI.

### 3.2 Onde e Como é Renderizado (Slint)

- **Arquivo:** `ui/main.slint`
- **Linhas 741 a 767:**
  ```slint
  HorizontalLayout {
      alignment: center;
      spacing: root.presets.length > 2 ? 10px : 16px;

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

      AddPresetButton {
          is-empty: root.presets.length == 0;
          clicked => {
              root.create-modal-error = "";
              root.show-create-modal = true;
          }
      }
  }
  ```
- **Condição de Verificação:**
  - Em Slint: `root.presets.length == 0` indica que nenhum arquivo `.json` existe.
  - Quando `root.presets.length == 0`, o loop `for preset in root.presets` não renderiza nada, deixando apenas o `AddPresetButton`.

---

## 4. Localização e Remoção do Botão Inferior "Otimizar Automaticamente" (R2)

### 4.1 Localização Exata
- **Arquivo:** `ui/main.slint`
- **Linhas:** 891 a 918
- **Código atual:**
  ```slint
  // Seção Inferior: Botão OTIMIZAR AUTOMATICAMENTE Centralizado + Barra de Status HUD
  VerticalLayout {
      spacing: 6px;
      alignment: center;

      HorizontalLayout {
          alignment: center;
          padding-top: 6px;
          padding-bottom: 2px;

          TacticalButton {
              text: "OTIMIZAR AUTOMATICAMENTE";
              badge: "PERFIL RECOMENDADO • BAIXA LATÊNCIA";
              enabled: !root.is-launching;
              background-color: CS2Theme.accent-orange;
              hover-color: #ff771a;
              custom-border-color: #c2410c;
              custom-border-hover-color: #ff851b;
              custom-border-width: 3px;
              button-width: 350px;
              text-color: #ffffff;
              icon-src: @image-url("assets/icons/rocket.svg");
              clicked => {
                  root.auto-optimize-clicked();
              }
          }
      }

      // Barra de Status HUD Tática
      HorizontalLayout {
          alignment: center;
          spacing: 8px;

          Rectangle {
              width: 6px;
              height: 6px;
              border-radius: 3px;
              y: (parent.height - self.height) / 2;
              background: CS2Theme.status-success;
          }

          Text {
              text: root.status-message;
              font-size: 10px;
              font-weight: 600;
              letter-spacing: 0.3px;
              color: CS2Theme.text-dim;
              horizontal-alignment: center;
          }
      }
  }
  ```

### 4.2 Impacto e Diagnóstico para Remoção (R2)
- O `TacticalButton` das linhas 896 a 917 deve ser **completamente removido**.
- A **Barra de Status HUD** (linhas 920 a 941) **deve ser preservada** no rodapé da tela. Ela é padrão visual do CounterRush em todas as abas e exibe os feedbacks reativos disparados pelas ações do usuário (`root.status-message`).
- A remoção do botão inferior libera aproximadamente 60px de altura na vertical, permitindo um layout mais respirado e harmonioso para o reposicionamento do input de parâmetros (R3).

---

## 5. Localização do Botão de Adicionar Otimização e Modal de Criação (R1)

### 5.1 Botão de Adicionar Pré-Existente
- **Arquivo do componente:** `ui/components/add_preset_button.slint`
- **Código atual:**
  ```slint
  export component AddPresetButton inherits Rectangle {
      in property <bool> is-empty: false;
      callback clicked();

      width: root.is-empty ? 180px : 44px;
      height: 44px;
      ...
      HorizontalLayout {
          alignment: center;
          spacing: 8px;

          Text {
              text: "+";
              font-size: 20px;
              font-weight: 900;
              color: touch.has-hover ? CS2Theme.accent-orange-bright : CS2Theme.accent-orange;
              vertical-alignment: center;
              horizontal-alignment: center;
          }

          if root.is-empty : Text {
              text: "NOVO PERFIL";
              font-size: 11px;
              font-weight: 800;
              letter-spacing: 0.8px;
              color: touch.has-hover ? CS2Theme.text-primary : CS2Theme.text-muted;
              vertical-alignment: center;
          }
      }
  ```
- **Adaptação necessária (R1):**
  - O texto `"NOVO PERFIL"` deve ser alterado para `"Criar Otimização"` (com o símbolo `+`).
  - A largura quando `is-empty` deve ser expandida de 180px para aproximadamente 220px a 240px, garantindo que o texto "Criar Otimização" não seja truncado nem sofra overflow.
  - Alternativamente, expor uma propriedade `label` ou criar variante tática para harmonizar lado a lado com o botão "Otimizar Automaticamente".

### 5.2 Modal de Criação de Perfil
- **Arquivo do modal:** `ui/components/create_profile_dialog.slint`
- **Invocação em `ui/main.slint` (linhas 1070 a 1079):**
  ```slint
  if root.show-create-modal : CreateProfileDialog {
      error-message: root.create-modal-error;
      cancel => {
          root.show-create-modal = false;
          root.create-modal-error = "";
      }
      confirmed(data) => {
          root.create-profile-confirmed(data);
      }
  }
  ```
- **Handler no backend Rust (`ui/src/main.rs:561-631`):**
  O callback `on_create_profile_confirmed` já está totalmente implementado e funcional: valida o nome digitado, grava o novo `.json` na pasta de otimizações, recarrega os presets via `scan_presets()`, seleciona o novo perfil e atualiza os switches e input de parâmetros.

---

## 6. Reposicionamento do Input de Parâmetros de Inicialização (R3)

### 6.1 Localização Atual
- **Arquivo:** `ui/main.slint`
- **Linhas:** 770 a 888 (atualmente renderizado *depois* da seleção de presets).

### 6.2 Nova Localização Requerida (R3)
- O requisito R3 estabelece:
  > *"O campo de entrada (input) para alterar os parâmetros de inicialização deve ser reposicionado para ficar localizado diretamente abaixo das otimizações ativas e antes da seleção de arquivos de otimização."*

### 6.3 Nova Sequência Vertical de Elementos na Aba de Otimizações:
1. **Cabeçalho**: "CENTRAL DE OTIMIZAÇÕES" (linhas 430–446)
2. **Otimizações Ativas (Grade de Switches)**: 4 linhas de `TweakSwitch` + `MasterToggleSwitch` (linhas 452–706)
3. **[NOVO LOCAL - R3] Input de Parâmetros de Inicialização**:
   - Renderizado condicionalmente (`if root.active-preset != ""` ou `root.presets.length > 0`).
   - Bloco com retângulo de 722px com `TextInput` e botão de 44px com ícone de lápis/salvar.
4. **Separador e Seção de Seleção de Perfis**:
   - Título "PERFIS DE OTIMIZAÇÃO (CONFIG/OPTIMZATIONS)".
   - No estado com `.json` (`root.presets.length > 0`): `PresetButton`s dos perfis + `AddPresetButton` (`+`).
   - No estado sem `.json` (`root.presets.length == 0`): Os 2 botões de ação ("Otimizar Automaticamente" e "+ Criar Otimização").
5. **Rodapé**: Barra de Status HUD Tática.

---

## 7. Detalhamento da Implementação: Exibição Exclusiva dos 2 Botões no Estado Vazio (R1/R4)

### 7.1 Comparativo das Abordagens

| Aspecto | Abordagem A: Empty State da Área de Ações (Recomendada) | Abordagem B: Empty State da Aba Inteira |
|---|---|---|
| **Conceito** | Mantém a grade de 13 switches desativados visíveis (mostrando os recursos do app) e exibe os 2 botões de ação na área central de perfis. | Oculta switches e input, exibindo no centro da tela exclusivamente os dois botões de ação em destaque. |
| **Fidelidade ao R1/AC** | Atende fielmente à organização funcional dos blocos de perfis. | Atende de forma ultraestrita à frase *"são exibidos apenas os dois botões"*. |
| **Hierarquia Visual** | O usuário enxerga a capacidade do sistema mesmo sem perfil criado. | Tela limpa com foco 100% nas duas ações de onboarding. |
| **Comportamento ao Clicar** | Clicar em "Otimizar Automaticamente" ou "+ Criar Otimização" cria o `.json` e imediatamente os switches se ativam e o perfil é selecionado. | Igual: a criação do `.json` muda reativamente `root.presets.length > 0`, revelando toda a interface completa. |

### 7.2 Especificação dos 2 Botões de Ação no Estado Vazio

```slint
if root.presets.length == 0 : HorizontalLayout {
    alignment: center;
    spacing: 16px;
    padding-top: 12px;
    padding-bottom: 12px;

    // Botão 1: Otimizar Automaticamente
    TacticalButton {
        text: "OTIMIZAR AUTOMATICAMENTE";
        badge: "HARDWARE ADAPTATIVO • 0.5ms";
        button-width: 280px;
        background-color: CS2Theme.accent-orange;
        hover-color: #ff771a;
        custom-border-color: #c2410c;
        custom-border-hover-color: #ff851b;
        custom-border-width: 2px;
        text-color: #ffffff;
        icon-src: @image-url("assets/icons/rocket.svg");
        clicked => {
            root.auto-optimize-clicked();
        }
    }

    // Botão 2: Criar Otimização (+)
    AddPresetButton {
        is-empty: true;
        button-width: 240px; // Ou largura estendida em add_preset_button.slint
        clicked => {
            root.create-modal-error = "";
            root.show-create-modal = true;
        }
    }
}
```

No componente `ui/components/add_preset_button.slint`:
- Atualizar o texto quando `is-empty: true` de `"NOVO PERFIL"` para `"CRIAR OTIMIZAÇÃO"` (ou `"Criar Otimização"`).
- O símbolo `+` é exibido à esquerda ou à direita com tipografia de destaque (`font-weight: 900`, `font-size: 20px`), atendendo ao critério:
  > *"o botão de adicionar pré-existente, agora com o símbolo `+` e o texto 'Criar Otimização'"*.

---

## 8. Verificação e Métodos de Validação

1. **Compilação e Verificação Rust:**
   ```powershell
   cargo check --manifest-path "g:\Outros computadores\Meu laptop\Desktop\CounterRush\Cargo.toml" -p counterrush-ui
   ```
2. **Validação Visual Interativa (Hot Reload):**
   ```cmd
   preview.cmd
   ```
   Ou:
   ```powershell
   slint-viewer ui/main.slint -I ui --auto-reload
   ```
3. **Cenário de Teste Vazio (R1):**
   - Renomear temporariamente a pasta `config/optimzations` ou esvaziá-la.
   - Abrir o app / viewer na aba de Otimizações.
   - Constatar: são exibidos exclusivamente os botões "Otimizar Automaticamente" e "+ Criar Otimização".
   - Constatar: o botão inferior "Otimizar Automaticamente" não existe mais no rodapé.
   - Clicar em "Otimizar Automaticamente" -> Deve gerar `AutoDetect.json`, atualizar a UI e exibir o estado completo com o perfil ativo.
4. **Cenário de Teste com Arquivos Existentes (R3 & R4):**
   - Com arquivos `.json` na pasta, abrir a aba de Otimizações.
   - Constatar: a grade de otimizações aparece no topo.
   - Constatar: o campo de entrada de parâmetros de inicialização está posicionado logo abaixo da grade e antes da seleção de perfis de otimização.
   - Constatar: a listagem e seleção de perfis continuam 100% operacionais.
