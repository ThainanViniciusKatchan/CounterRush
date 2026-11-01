# Relatório de Handoff — Explorer 3: Layout da Aba de Otimizações, Input de Parâmetros e Transição de Estados

**Tipo de Handoff:** Hard (Task Completa)  
**Data:** 2026-11-01  
**Autor:** Explorer 3 (`teamwork_explorer_survey_3`)  
**Destinatário:** Orchestrator (ID: `a2e09f4d-39a0-4d52-880b-d65100808272`) e Implementer  
**Documento Detalhado Suplementar:** `.agents/teamwork/teamwork_explorer_survey_3/analysis.md`

---

## 1. Observation (Observações Diretas)

Durante a exploração read-only na base de código do CounterRush, registraram-se as seguintes evidências e constatações diretas:

1. **Implementação Atual do Input de Parâmetros de Inicialização:**
   - **Arquivo:** `ui/main.slint`, linhas 770 a 888.
   - **Posição Atual:** Renderizado dentro do contêiner intermediário da aba 1 (Otimizações), **abaixo** dos cartões de seleção de perfil (`PresetButton`s).
   - **Guarda Condicional:** `if root.active-preset != "" : HorizontalLayout { ... }` (linha 770). Se `active-preset` for vazio, o campo não é instanciado na árvore visual.
   - **Componentes Visuais:**
     - Caixa de texto (`Rectangle`, linhas 774-844): `width: 722px; height: 44px;`, fundo `CS2Theme.bg-card`, borda `CS2Theme.border` (ou `CS2Theme.accent-orange` sob foco em edição).
     - Badge interno: `width: 88px; height: 22px;` com o texto `"PARÂMETROS"`.
     - Campo de edição (`TextInput`, linhas 817-832): `text <=> root.launch-args-text;`, `read-only: !root.is-editing-launch-args;`. Ao pressionar Enter (`accepted`), finaliza edição e emite `root.save-launch-args-clicked(root.launch-args-text);`.
     - Botão lateral de edição/salvamento (`Rectangle`, linhas 846-887): `width: 44px; height: 44px;` com ícone dinâmico que alterna entre `@image-url("assets/icons/pencil.svg")` e `@image-url("assets/icons/save.svg")`. Ao clicar, alterna `root.is-editing-launch-args` e salva os argumentos quando a edição é confirmada.
   - **Backend Rust (`ui/src/main.rs:636-671`):**
     - O callback `main_window.on_save_launch_args_clicked(move |args_str| { ... })` carrega a configuração ativa a partir de `active_config_path` (`Arc<RwLock<PathBuf>>`), atualiza `config.launch_args = args_str.split_whitespace().map(String::from).collect()`, salva no disco via `config.save_to_file(&target_path)` e reflete o valor atualizado na UI.

2. **Seção de "Otimizações Ativas" na UI:**
   - **Arquivo:** `ui/main.slint`, linhas 426 a 706.
   - **Conteúdo Renderizado:**
     - Cabeçalho: `"CENTRAL DE OTIMIZAÇÕES"` e `"13 Módulos de Baixa Latência"` (linhas 430-446).
     - Grade modular com 4 linhas horizontais de botões táteis (`TweakSwitch`, 188px x 50px cada):
       - *Linha 1 (linhas 453-515):* Núcleo 0 Off (`cpu_core_0`), Timer 0.5ms (`timer_resolution`), Prioridade Alta (`process_priority`), Limpar RAM (`empty_standby`).
       - *Linha 2 (linhas 518-580):* GPU Apps Off (`gpu_acceleration`), Otimização DPI (`dpi_optimization`), Compactar RAM (`memory_compression`), Agendador MMCSS (`mmcss`).
       - *Linha 3 (linhas 583-645):* HPET Off (`hpet`), Game Bar Off (`game_bar`), Modo MSI (`msi`), Power Save Off (`power_saving`).
       - *Linha 4 (linhas 648-705):* Render Queue (`render_queue`), `MasterToggleSwitch` (Ativar/Desativar todos os 13 módulos simultaneamente) e 2 espaçadores transparentes de 188px para manter o grid simétrico.

3. **Seção de Seleção de Arquivos de Otimização:**
   - **Arquivo:** `ui/main.slint`, linhas 709 a 767.
   - **Conteúdo Renderizado:**
     - Separador tático com o texto `"PERFIS DE OTIMIZAÇÃO (CONFIG/OPTIMZATIONS)"` (linhas 713-739).
     - Contêiner `HorizontalLayout` com loop de renderização `for preset in root.presets : PresetButton { ... }` (linhas 745-758), onde cada card exibe o nome do arquivo, a tag de categoria e a pílula de estado (`ATIVO` / `USAR`).
     - Botão de criação `AddPresetButton` (`ui/components/add_preset_button.slint`, linhas 760-766), com `width: root.is-empty ? 180px : 44px;`. Quando vazio exibe `+` e o texto `"NOVO PERFIL"`. Ao ser clicado, abre o modal `CreateProfileDialog` (`root.show-create-modal = true;`).

4. **Botão Inferior a Ser Removido (Requisito R2):**
   - **Arquivo:** `ui/main.slint`, linhas 891 a 918.
   - O botão `TacticalButton` `"OTIMIZAR AUTOMATICAMENTE"` encontra-se na seção inferior da tela, logo acima da Barra de Status HUD (`ui/main.slint:920-941`).

---

## 2. Logic Chain (Cadeia Lógica de Raciocínio)

1. **Reposicionamento do Input de Parâmetros de Inicialização (R3):**
   - O requisito R3 determina que o input de parâmetros de inicialização deve estar localizado *diretamente abaixo das otimizações ativas e antes da seleção de arquivos de otimização*.
   - A partir da Observação 1 e 2, as otimizações ativas encerram-se na linha 706 (ao final da 4ª linha da grade de switches). A seção de seleção de perfis inicia-se na linha 709 com o separador `"PERFIS DE OTIMIZAÇÃO"`.
   - Consequentemente, para cumprir R3, basta mover o bloco condicional `if root.active-preset != "" : HorizontalLayout { ... }` (linhas 770-888) para a linha 707.
   - Isso estabelece a ordem visual exata: **Grade de Otimizações Ativas (13 switches) -> Input de Parâmetros de Inicialização -> Divisor e Seleção de Perfis -> Barra HUD**.

2. **Remoção do Botão Inferior (R2) e Equilíbrio Espacial:**
   - A exclusão do `TacticalButton` em `ui/main.slint:896-917` remove a redundância inferior e libera ~60px de espaço vertical.
   - A janela possui altura fixa de 620px (`min-height: 620px; max-height: 620px`). A área útil interna da aba é de ~586px.
   - A soma das alturas com o input reposicionado é: Cabeçalho (~30px) + Grade de Switches (242px) + Input de Parâmetros (54px) + Seção de Perfis (~65px) + Barra HUD (~18px) = **409px**.
   - Restam ~177px de margem vertical livre, garantindo que o layout ficará arejado, elegante e imune a truncamento ou overflow visual.

3. **Interação com o Estado Sem Arquivos .json (R1 x R3):**
   - No estado sem arquivos `.json` (`root.presets.length == 0`), o backend define `root.active-preset = ""` e todos os switches como `false`.
   - Pela guarda condicional `if root.active-preset != ""`, o input de parâmetros de inicialização **permanece automaticamente oculto** quando não há perfis `.json`.
   - Isso resolve a interação entre R1 e R3 com total coerência: não existindo arquivo de perfil, não há como ler nem para onde salvar parâmetros de inicialização.
   - No lugar da listagem de perfis inexistentes, renderizam-se com exclusividade os dois botões de ação demandados por R1:
     1. Botão "Otimizar Automaticamente" (acionando `root.auto-optimize-clicked()`).
     2. Botão "Criar Otimização +" (`AddPresetButton` com `is-empty: true`).
   - Quando o usuário clica em "Otimizar Automaticamente", o arquivo `AutoDetect.json` é gerado no disco, `root.presets.length` torna-se 1, `root.active-preset` torna-se `"AutoDetect"`, e a UI transiciona reativamente para o estado completo (R4), no qual o input de parâmetros de inicialização surge imediatamente posicionado abaixo da grade de otimizações.

4. **Ajuste no Componente `AddPresetButton` (R1):**
   - O componente `ui/components/add_preset_button.slint` já possui a lógica de alternância visual via `root.is-empty`.
   - Para atender ao requisito R1, altera-se o texto condicional da linha 33 de `"NOVO PERFIL"` para `"Criar Otimização"` e ajusta-se a largura de `180px` para `210px` quando `root.is-empty == true`.

---

## 3. Caveats (Ressalvas e Alternativas de Design)

1. **Escopo da Exibição Exclusiva dos Dois Botões (R1):**
   - **Opção A (Recomendada pela Engenharia de UI):** Mantém a grade com os 13 switches visíveis no topo (com todos os toggles em `false`/desligados), oculta o input de parâmetros de inicialização (`active-preset == ""`), e exibe com exclusividade os dois botões na seção de ação/perfis.
     - *Justificativa:* Permite ao usuário visualizar de imediato todo o catálogo de otimizações oferecido pelo CounterRush antes de tomar uma decisão, sem transições bruscas de tela.
   - **Opção B (Interpretação Literal Estrita da Aba):** Oculta toda a grade de 13 switches caso `root.presets.length == 0`, centralizando apenas os 2 botões de ação e a barra de status na aba 1.
     - *Justificativa:* Cumpre no sentido mais rigoroso a frase *"a aba de Otimizações deve exibir exclusivamente dois botões de ação"*.
   - *Ambas as opções são triviais de implementar no Slint através de uma condicional `if root.presets.length == 0` / `if root.presets.length > 0`.*
2. **Read-Only Rigoroso:** O Explorer 3 não modificou nenhum arquivo de código-fonte (`.rs`, `.slint`, `.json`). As propostas foram validadas exclusivamente por análise estática e mapeamento referencial.

---

## 4. Conclusion (Conclusão e Diagnóstico)

A arquitetura do CounterRush está perfeitamente preparada para receber as alterações com risco mínimo:
- O input de parâmetros de inicialização está isolado em um contêiner autocontido (`ui/main.slint:770-888`) e pode ser movido para logo abaixo das otimizações ativas (linha 707) sem impactos adversos nos bindings ou callbacks.
- A remoção do botão inferior de auto-otimização (R2) limpa o layout e disponibiliza o espaço vertical necessário.
- A ocultação natural do input quando `active-preset == ""` harmoniza perfeitamente o requisito de reposicionamento (R3) com o requisito de estado vazio (R1).
- O backend Rust em `ui/src/main.rs` já possui todas as funções e callbacks necessários (`on_auto_optimize_clicked`, `on_create_profile_confirmed`, `on_save_launch_args_clicked`, `scan_presets`), não exigindo nenhuma alteração de lógica no backend Rust.

---

## 5. Verification Method (Método de Verificação Independente)

Para que o orquestrador, o implementador e o revisor possam verificar de forma autônoma:

1. **Checagem de Compilação dos Arquivos Slint e Crate UI:**
   ```powershell
   cargo check -p counterrush-ui
   ```
2. **Validação Visual Interativa via Auto-Reload:**
   ```cmd
   g:\Outros computadores\Meu laptop\Desktop\CounterRush\preview.cmd
   ```
   *Procedimento:* O comando inicializa o `slint-viewer` na janela principal. Qualquer alteração nos arquivos `.slint` atualizará a UI instantaneamente na tela.
3. **Teste de Estado Vazio (Cenário R1, R2):**
   - Esvaziar temporariamente a pasta `config/optimzations/` (movendo arquivos para uma pasta temporária fora da base).
   - Iniciar o preview ou aplicação (`cargo run -p counterrush-ui`).
   - Confirmar visualmente:
     - [ ] O botão inferior "OTIMIZAR AUTOMATICAMENTE" não existe no rodapé da tela.
     - [ ] O campo de input de parâmetros de inicialização NÃO está renderizado.
     - [ ] São exibidos com exclusividade os dois botões: "Otimizar Automaticamente" e o botão com o símbolo `+` e o texto "Criar Otimização".
     - [ ] O clique em "Otimizar Automaticamente" gera `AutoDetect.json` e transiciona a tela para o estado preenchido.
4. **Teste de Estado Preenchido e Reposicionamento (Cenário R3, R4):**
   - Restaurar os arquivos `.json` na pasta `config/optimzations/`.
   - Confirmar visualmente:
     - [ ] O campo de input para parâmetros de inicialização está renderizado **diretamente abaixo** das 13 otimizações ativas e **antes** da linha de seleção de arquivos `.json`.
     - [ ] Clicar no botão de edição (lápis) desbloqueia o campo de input e alterna o ícone para o disquete de salvamento.
     - [ ] Alterar o texto e clicar em salvar (ou pressionar Enter) persiste os novos argumentos no arquivo `.json` ativo.
     - [ ] Alternar entre perfis atualiza o texto do input com os parâmetros do perfil selecionado.
