# Handoff Report — Explorer 1: Mapeamento da UI da Aba de Otimizações

**Data:** 2026-11-01  
**Autor:** Explorer 1 (`teamwork_explorer_survey_1`)  
**Destinatário:** Orchestrator (`a2e09f4d-39a0-4d52-880b-d65100808272`) e Implementer  
**Status:** Concluído com sucesso (Hard Handoff)  
**Documento detalhado de referência:** `.agents/teamwork/teamwork_explorer_survey_1/analysis.md`

---

## 1. Observation

Durante a investigação exploratória read-only na base de código do CounterRush, observou-se:

1. **Stack Tecnológica da Interface:**
   - O projeto é um workspace Rust (versão 0.8.0), utilizando a biblioteca gráfica **Slint 1.8** (`ui/Cargo.toml:10`).
   - O viewer suporta Hot-Reload interativo via script `preview.cmd`: `slint-viewer ui/main.slint -I ui --auto-reload`.

2. **Aba de Otimizações na View Principal (`ui/main.slint`):**
   - A aba é ativada quando `root.active-tab == 1` e estende-se das linhas 418 até 942.
   - O topo contém o cabeçalho "CENTRAL DE OTIMIZAÇÕES" (linhas 430-446) e a grade com 13 módulos `TweakSwitch` e 1 `MasterToggleSwitch` (linhas 452-706).

3. **Botão Inferior a Ser Removido (Requisito R2):**
   - Localizado em `ui/main.slint:891-918` dentro de `VerticalLayout`:
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
                 ...
                 clicked => { root.auto-optimize-clicked(); }
             }
         }
         // Barra de Status HUD Tática (linhas 920-941)
     }
     ```
   - O botão `TacticalButton` está posicionado imediatamente antes da Barra de Status HUD.

4. **Input de Parâmetros de Inicialização (Requisito R3):**
   - Localizado em `ui/main.slint:770-888` sob a condição `if root.active-preset != ""`.
   - Atualmente renderizado **abaixo** dos botões de seleção de perfil (`PresetButton`s), contrariando a hierarquia solicitada por R3.

5. **Listagem de Perfis e Botão de Adicionar (Requisitos R1 e R4):**
   - Localizado em `ui/main.slint:741-767`:
     ```slint
     HorizontalLayout {
         alignment: center;
         spacing: root.presets.length > 2 ? 10px : 16px;
         for preset in root.presets : PresetButton { ... }
         AddPresetButton {
             is-empty: root.presets.length == 0;
             clicked => {
                 root.create-modal-error = "";
                 root.show-create-modal = true;
             }
         }
     }
     ```
   - O componente `AddPresetButton` em `ui/components/add_preset_button.slint:33` exibe atualmente o texto `"NOVO PERFIL"` quando `is-empty: true`, com largura de 180px.
   - O modal de criação `CreateProfileDialog` é renderizado como overlay em `ui/main.slint:1070-1079` quando `root.show-create-modal == true`.

6. **Mecanismo de Carregamento e Estado Vazio no Backend (`ui/src/main.rs`):**
   - `scan_presets()` (linhas 5-22) chama `counterrush_core::list_optimization_configs()` para listar arquivos `.json` em `config/optimzations/`.
   - Na inicialização (linhas 32-50) e no watcher em tempo real (linhas 93-142), quando a lista de arquivos está vazia, o backend zera todos os 13 switches para `false`, define `active-preset` como `""` e `launch-args-text` como `""`.
   - Em `ui/src/main.rs:384-387`, alternar switches quando não há perfil ativo exibe: `"Crie ou selecione um perfil para salvar alterações."`.

---

## 2. Logic Chain

1. **Da Remoção do Botão Inferior (R2):**
   - O botão inferior em `ui/main.slint:896-917` é redundante com o botão de auto-otimização a ser exibido no estado vazio.
   - A sua exclusão remove código desnecessário e libera ~60px de espaço vertical na janela de altura fixa (620px), permitindo acomodar confortavelmente o input de parâmetros de inicialização reposicionado sem risco de overflow.
   - A Barra de Status HUD imediatamente abaixo do botão (linhas 920-941) deve ser preservada pois fornece o feedback tático ao usuário em tempo real.

2. **Do Reposicionamento do Input de Parâmetros (R3):**
   - Conforme R3, o input de parâmetros deve ficar "localizado diretamente abaixo das otimizações ativas e antes da seleção de arquivos de otimização".
   - As "otimizações ativas" são os 13 módulos de switches (linhas 452-706).
   - Movendo o bloco de `if root.active-preset != ""` de linhas 770-888 para a linha 707 (logo após a Grade e antes do cabeçalho da seção de perfis), a hierarquia visual atende rigorosamente ao requisito.

3. **Do Tratamento do Botão de Adicionar (R1):**
   - O requisito exige que o botão de adicionar exiba o símbolo `+` e o texto "Criar Otimização".
   - O componente `ui/components/add_preset_button.slint` já possui a estrutura com o caractere `+` (linha 23) e o texto condicional (linha 32). Bastará alterar a string `"NOVO PERFIL"` para `"Criar Otimização"` (ou `"CRIAR OTIMIZAÇÃO"`) e ajustar a propriedade de largura quando `is-empty: true` (de 180px para ~230px).
   - Ao ser clicado, ele continuará abrindo o modal `CreateProfileDialog` já existente e integrado.

4. **Da Exibição Exclusiva dos 2 Botões no Estado Vazio (R1 e R4):**
   - A condição de estado vazio é identificada diretamente no Slint por `root.presets.length == 0`.
   - Quando `root.presets.length == 0`:
     - Não existem perfis `.json`, logo `root.active-preset == ""` e o input de parâmetros já se oculta naturalmente.
     - A seção de seleção de perfis não deve renderizar botões de preset inexistentes.
     - Em seu lugar, renderiza-se um container com espaçamento e alinhamento centralizado contendo exclusivamente:
       1. Botão "Otimizar Automaticamente" (acionando `root.auto-optimize-clicked()`).
       2. Botão "+ Criar Otimização" (`AddPresetButton` acionando a abertura do modal).
     - Quando o usuário aciona qualquer um dos dois botões, um arquivo `.json` é gerado no disco (`AutoDetect.json` ou perfil customizado). Imediatamente `root.presets.length` torna-se `> 0`, fazendo a UI alternar de forma reativa e instantânea para o estado completo (R4).

---

## 3. Caveats

1. **Abordagem de Exibição no Estado Vazio (Área de Ações vs. Aba Inteira):**
   - **Abordagem A (Recomendada):** Mantém a grade de 13 switches no topo (em estado neutro/desativado) e exibe com exclusividade os 2 botões na área de perfis/ações. Vantagem: o usuário visualiza todo o potencial técnico do CounterRush antes mesmo de otimizar.
   - **Abordagem B:** Oculta completamente a grade de switches quando `root.presets.length == 0`, centralizando apenas os 2 botões de ação e a barra de status HUD. Vantagem: cumpre no sentido mais estrito a frase "são exibidos apenas os dois botões".
   - *Nota:* O plano de implementação deve explicitar ambas para validação e aprovação do usuário.
2. **Largura e Dimensões Fixas da Janela:**
   - A janela possui largura e altura fixas em 980x620px (`ui/main.slint:18-23`). Qualquer ajuste de layout deve obrigatoriamente respeitar essas dimensões para não cortar elementos ou forçar comportamento inadequado.

---

## 4. Conclusion

A base de código do CounterRush é modular, bem tipada e altamente responsiva a alterações reativas no Slint.  
Todos os 5 pontos focais foram mapeados com precisão:
- O botão inferior a ser removido (R2) foi identificado e seu isolamento não quebra dependências.
- O campo de input de parâmetros (R3) pode ser reposicionado diretamente entre a grade de switches e a seção de perfis.
- O botão de adicionar e o modal de criação (R1) já possuem lógica completa de backend e necessitam apenas de ajuste textual e dimensional em Slint.
- O chaveamento condicional reativo baseado em `root.presets.length == 0` permite alternar com elegância e consistência entre o estado vazio (2 botões exclusivos) e o estado com arquivos (R4).

---

## 5. Verification Method

Para verificação independente pelo Implementer ou pelo usuário:

1. **Checagem de Compilação do Crate UI:**
   ```powershell
   cargo check --manifest-path "g:\Outros computadores\Meu laptop\Desktop\CounterRush\Cargo.toml" -p counterrush-ui
   ```
2. **Visualização Interativa com Auto-Reload:**
   ```cmd
   g:\Outros computadores\Meu laptop\Desktop\CounterRush\preview.cmd
   ```
   *Condição de validação:* Alterações em arquivos `.slint` são refletidas instantaneamente na janela de preview sem necessidade de recompilação do binário Rust.
3. **Cenário 1 — Estado Sem Arquivos .json (R1, R2):**
   - Mover temporariamente arquivos de `config/optimzations/` para um diretório backup.
   - Abrir o preview na aba 1 (Otimizações).
   - Inspecionar visualmente:
     - O botão inferior "OTIMIZAR AUTOMATICAMENTE" não existe no rodapé da janela.
     - São exibidos com exclusividade os botões "Otimizar Automaticamente" e "+ Criar Otimização".
4. **Cenário 2 — Ação de Otimização Automática e Criação:**
   - Clicar em "Otimizar Automaticamente": o perfil `AutoDetect.json` é criado e a interface transiciona para o estado com perfil ativo.
   - Clicar em "+ Criar Otimização": o modal `CreateProfileDialog` abre normalmente.
5. **Cenário 3 — Estado com Arquivos Existentes e Input Reposicionado (R3, R4):**
   - Restaurar os arquivos `.json` em `config/optimzations/`.
   - Verificar se o campo de parâmetros de inicialização é renderizado **diretamente abaixo** das otimizações ativas (grade de switches) e **antes** da seleção de arquivos `.json`.
