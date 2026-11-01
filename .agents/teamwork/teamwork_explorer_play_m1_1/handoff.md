# Relatório de Investigação Técnica — Botão "Jogar CS2" e Interceptação de Perfis Ausentes

## 1. Observation (Observações Diretas)

### 1.1 Localização e Declaração do Botão "Jogar CS2" na UI Slint
No arquivo `ui/main.slint`, linhas 367 a 395:
```slint
367:             // Seção Inferior: Botão JOGAR CS2 Centralizado + Barra de Status HUD
368:             VerticalLayout {
369:                 spacing: 6px;
370:                 alignment: center;
371: 
372:                 HorizontalLayout {
373:                     alignment: center;
374:                     padding-top: 6px;
375:                     padding-bottom: 2px;
376: 
377:                     TacticalButton {
378:                         text: root.is-launching ? "INICIANDO..." : "JOGAR CS2";
379:                         badge: root.is-launching ? "APLICANDO TWEAKS & STEAM" : "INICIAR COM ALTA PERFORMANCE";
380:                         enabled: !root.is-launching;
381:                         background-color: CS2Theme.accent-orange;
382:                         hover-color: #ff771a;
383:                         custom-border-color: #c2410c;
384:                         custom-border-hover-color: #ff851b;
385:                         custom-border-width: 3px;
386:                         text-color: #ffffff;
387:                         icon-src: @image-url("assets/icons/play.svg");
388:                         clicked => {
389:                             root.is-launching = true;
390:                             root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
391:                             root.launch-cs2-clicked();
392:                         }
393:                     }
394:                 }
```
O botão está localizado exclusivamente no interior do bloco da aba Painel (`if root.active-tab == 0 : VerticalLayout`).
Ele é uma instância do componente `TacticalButton` (`ui/components/tactical_button.slint`).

### 1.2 Callback Disparado no Slint
- Na linha 70 de `ui/main.slint`, o callback está declarado como:
  ```slint
  callback launch-cs2-clicked();
  ```
- No evento `clicked =>` (linhas 388-392 de `ui/main.slint`), antes de invocar o callback, a UI altera diretamente seu estado interno:
  ```slint
  root.is-launching = true;
  root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
  root.launch-cs2-clicked();
  ```

### 1.3 Tratamento do Callback no Backend Rust (`ui/src/main.rs`)
No arquivo `ui/src/main.rs`, linhas 228 a 337:
```rust
228:     let window_handle = main_window.as_weak();
229:     let launch_config_ref = active_config_path.clone();
230:     let fallback_path = resolved_path.clone();
231:     main_window.on_launch_cs2_clicked(move || {
...
247:             Some(ui) => {
248:                 ui.set_status_message("Otimizando sistema o CS2...".into());
...
269:         let target_cfg_path = launch_config_ref.read().map(|p| p.clone()).unwrap_or_else(|_| fallback_path.clone());
270:         std::thread::spawn(move || {
271:             let core = counterrush_core::OptimizerCore::default();
272:             let (resolved, mut config) = match core.load_or_create_custom_config(Some(&target_cfg_path)) {
...
300:             let _ = config.save_to_file(&resolved);
...
311:             match core.launch_custom(Some(&resolved), std::time::Duration::from_secs(90)) {
```
- O handler extrai o estado atual dos 13 switches de ajustes.
- Ele lê `target_cfg_path`. Quando a pasta de perfis está vazia (`config/optimzations`), `active_config_path` e `fallback_path` guardam `PathBuf::new()`.
- Em `core/src/lib.rs`, linhas 338-350, o método `load_or_create_custom_config`:
  ```rust
  let path = resolve_custom_config_path(config_path);
  let default_args = if !path.exists() {
      let (_, category) = self.get_hardware_profile()?;
      get_default_launch_args_for_category(category)
  } else {
      Vec::new()
  };
  let cfg = config::CustomOptimizationConfig::load_or_create(&path, default_args)?;
  ```
  Se `config_path` for vazio ou inexistente, `resolve_custom_config_path` resolve para `config/optimzations/custom_optimization.json`, e `load_or_create` **cria o arquivo automaticamente no disco**, salvando-o e lançando o CS2.

### 1.4 Estado de Existência de Perfis na UI
No arquivo `ui/main.slint`, o estado de existência de perfis já é rastreado reativamente:
- `in-out property <[PresetModel]> presets: [];` (linha 45 de `ui/main.slint`).
- Na aba Otimizações (`active-tab == 1`), a interface já usa exatamente `root.presets.length == 0` (linha 834) para renderizar a interface de estado sem perfis, e `root.presets.length > 0` (linha 865) para listar perfis existentes.
- Em `ui/src/main.rs`, linhas 28-30 e no thread de watcher (linhas 105-115), `main_window.set_presets(...)` é mantido atualizado a partir de `scan_presets()`.

### 1.5 Padrão de Modais no Projeto
- `ui/components/delete_profile_dialog.slint` e `ui/components/create_profile_dialog.slint`:
  - Ambos são overlays com `background: rgba(11, 15, 23, 0.88)` e bloqueio de cliques de fundo através de um `TouchArea { mouse-cursor: default; }`.
  - Apresentam um cartão retangular centralizado (`background: CS2Theme.bg-card`, `border-radius: CS2Theme.radius-md`).
  - São controlados por flags booleanas declaradas em `ui/main.slint` (`root.show-delete-modal`, `root.show-create-modal`).
  - São renderizados ao final do componente `MainWindow` em `ui/main.slint` (linhas 1086-1109).

### 1.6 Navegação de Abas
No arquivo `ui/components/sidebar.slint`, a aba de índice 1 corresponde a "Otimizações":
- Linhas 99-104:
  ```slint
  NavButton {
      text: "Otimizações";
      icon-src: @image-url("../assets/icons/rocket.svg");
      active: root.active-tab == 1;
      clicked => { root.active-tab = 1; root.tab-selected(1); }
  }
  ```
- Portanto, mudar `root.active-tab = 1` transiciona o usuário diretamente para a aba Otimizações.

---

## 2. Logic Chain (Cadeia de Raciocínio Lógico)

1. **Risco Crítico Atual com a Base Vazia**:
   - Conforme observado em 1.3, quando todos os perfis são excluídos (ou na ausência inicial de perfis), `target_cfg_path` fica vazio (`PathBuf::new()`).
   - Se o usuário clica em "JOGAR CS2", `ui/src/main.rs` chama `core.load_or_create_custom_config(Some(&target_cfg_path))`.
   - Essa chamada gera silenciosamente o arquivo `config/optimzations/custom_optimization.json` no disco sem autorização do usuário e inicia o jogo, quebrando o comportamento desejado onde nenhum jogo deve ser iniciado caso não haja perfis de otimização.

2. **Ponto Ideal de Interceptação: Primário na Camada Slint**:
   - Conforme observado em 1.2, o botão "JOGAR CS2" atualmente executa `root.is-launching = true;` imediatamente no momento do clique.
   - Se interceptássemos unicamente no Rust através do callback `launch-cs2-clicked`, a interface Slint já teria alterado seu estado para `is-launching = true`, mudando o texto do botão para "INICIANDO..." e o badge para "APLICANDO TWEAKS & STEAM", gerando flicker e necessitando de um evento assíncrono para reverter o botão.
   - Conforme observado em 1.4, a interface Slint já possui a lista reativa `root.presets` sincronizada com o disco (`root.presets.length == 0`).
   - Ao interceptar diretamente no evento `clicked =>` do `TacticalButton` em `ui/main.slint:388`:
     - Se `root.presets.length == 0`: a UI define `root.show-no-profiles-modal = true`, sem acionar `root.is-launching = true`, sem mudar textos e sem disparar `root.launch-cs2-clicked()`.
     - Se `root.presets.length > 0`: a UI prossegue com o fluxo normal (`is-launching = true; launch-cs2-clicked();`).

3. **Salvaguarda Defensiva na Camada Rust (Defesa em Profundidade)**:
   - Seguindo os princípios de Clean Architecture e robustez exigidos pelo projeto, o handler `on_launch_cs2_clicked` em `ui/src/main.rs` não deve assumir cegamente que a UI sempre bloqueou requisições inválidas.
   - Adicionando no início de `on_launch_cs2_clicked` uma verificação:
     ```rust
     let presets = scan_presets();
     if presets.is_empty() {
         if let Some(ui) = window_handle.upgrade() {
             ui.set_is_launching(false);
             ui.set_status_message("Nenhum perfil de otimização encontrado.".into());
             ui.set_show_no_profiles_modal(true);
         }
         return;
     }
     ```
     garante-se que nenhum arquivo `.json` seja recriado indevidamente no disco e nenhum processo externo seja disparado sob nenhuma hipótese.

4. **Componente do Modal Requisitado (`NoProfilesDialog`)**:
   - Requisitos do usuário em `ORIGINAL_REQUEST.md`:
     1. Mensagem com o texto exato: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`.
     2. Dois botões: `"Criar Perfil"` e `"Sair"`.
     3. Botão `"Criar Perfil"`: leva o usuário para a aba Otimizações.
     4. Botão `"Sair"`: fecha o modal.
   - Conforme observado em 1.5 e 1.6:
     - Cria-se `ui/components/no_profiles_dialog.slint` seguindo o design tático do CounterRush (`CS2Theme`).
     - Ações no Slint:
       - No evento `exit`: `root.show-no-profiles-modal = false;` (usuário permanece na aba atual).
       - No evento `create-profile`: `root.show-no-profiles-modal = false; root.active-tab = 1; root.status-message = "Ajustes Avançados de Hardware";`. Na aba 1 (Otimizações), o usuário visualizará imediatamente os botões "OTIMIZAR AUTOMATICAMENTE" e "CRIAR NOVO PERFIL" (conforme linha 834 de `ui/main.slint`).

---

## 3. Caveats (Ressalvas e Limitações)

1. **Persistência de Arquivos no Disco**:
   - A propriedade `root.presets` reflete a existência de arquivos em `config/optimzations/*.json`. O loop em background (`ui/src/main.rs:99`) atualiza essa lista a cada ~1.5 segundos caso haja mudanças externas no sistema de arquivos, além da atualização imediata realizada nas ações de criação e exclusão da própria UI.
2. **Ambiente de Investigação Read-Only**:
   - Esta investigação foi estritamente somente-leitura em conformidade com as instruções do subagente e as regras do projeto. Nenhuma alteração foi realizada nos arquivos de código-fonte.
3. **Escopo Delimitado**:
   - O requisito do usuário solicita que o botão "Criar Perfil" leve para a aba Otimizações (`root.active-tab = 1`). Ele não solicita que o modal secundário `CreateProfileDialog` seja aberto automaticamente ao transicionar de aba (a aba de otimizações já expõe de forma proeminente o botão de autodetecção e o botão de criar perfil).

---

## 4. Conclusion (Conclusão Estruturada e Plano de Arquivos)

### 4.1 Respostas Diretas às Perguntas da Investigação

| Pergunta | Resposta Conclusiva |
|---|---|
| **Onde e como o botão "Jogar CS2" está declarado?** | Declarado em `ui/main.slint` (linhas 377–393) dentro da aba `active-tab == 0` (Painel), utilizando o componente `TacticalButton` (`ui/components/tactical_button.slint`). |
| **Qual callback é disparado quando clicado?** | Dispara o callback `root.launch-cs2-clicked()`, declarado na linha 70 de `ui/main.slint`. |
| **Como o backend Rust trata esse callback atualmente?** | Registrado em `ui/src/main.rs` (linhas 228–337). Lê os switches, obtém o path do perfil e chama `core.load_or_create_custom_config`. **Atenção:** se não houver perfis, ele cria automaticamente `custom_optimization.json` no disco e inicia o CS2! |
| **Como e onde interceptar caso não existam perfis?** | No bloco `clicked =>` do `TacticalButton` em `ui/main.slint` (linha 388), avaliando `if root.presets.length == 0`. Se verdadeiro, ativa o modal (`root.show-no-profiles-modal = true`) sem marcar `is-launching = true` e sem chamar o callback de lançamento. |
| **Interceptar direto no Slint, via Rust ou ambos?** | **Ambos**. O Slint é o interceptador primário para garantir zero flicker visual e navegação de abas imediata; o Rust atua como barreira de segurança defensiva em `on_launch_cs2_clicked` para evitar efeitos colaterais caso receba o comando sem perfis válidos. |
| **Quais arquivos precisarão ser alterados?** | 1 novo arquivo (`ui/components/no_profiles_dialog.slint`) e 2 arquivos existentes modificados (`ui/main.slint` e `ui/src/main.rs`). |

### 4.2 Arquivos e Alterações Específicas Detalhadas

1. **`ui/components/no_profiles_dialog.slint`** (NOVO ARQUIVO):
   - Criar o componente modal tático:
     - Backdrop escuro bloqueador (`rgba(11, 15, 23, 0.88)`).
     - Card com borda de destaque e cabeçalho "PERFIL DE OTIMIZAÇÃO NECESSÁRIO" e botão fechar `✕`.
     - Mensagem obrigatória com o texto exato: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`.
     - Botão `"Sair"` chamando callback `exit()`.
     - Botão `"Criar Perfil"` chamando callback `create_profile()`.

2. **`ui/main.slint`** (MODIFICAÇÃO):
   - Importar o novo componente: `import { NoProfilesDialog } from "components/no_profiles_dialog.slint";`.
   - Adicionar propriedade reativa: `in-out property <bool> show-no-profiles-modal: false;`.
   - Modificar o `clicked =>` do `TacticalButton` (linhas 388–393):
     ```slint
     clicked => {
         if root.presets.length == 0 {
             root.show-no-profiles-modal = true;
         } else {
             root.is-launching = true;
             root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
             root.launch-cs2-clicked();
         }
     }
     ```
   - Adicionar overlay ao final do arquivo:
     ```slint
     if root.show-no-profiles-modal : NoProfilesDialog {
         exit => {
             root.show-no-profiles-modal = false;
         }
         create-profile => {
             root.show-no-profiles-modal = false;
             root.active-tab = 1;
             root.status-message = "Ajustes Avançados de Hardware";
         }
     }
     ```

3. **`ui/src/main.rs`** (MODIFICAÇÃO):
   - No início do handler `main_window.on_launch_cs2_clicked` (linha 231):
     ```rust
     let presets = scan_presets();
     if presets.is_empty() {
         if let Some(ui) = window_handle.upgrade() {
             ui.set_is_launching(false);
             ui.set_status_message("Nenhum perfil de otimização encontrado.".into());
             ui.set_show_no_profiles_modal(true);
         }
         return;
     }
     ```

---

## 5. Verification Method (Método de Verificação Independente)

Para o implementador e auditores verificarem de forma reproduzível e independente:

1. **Verificação de Compilação e Sintaxe Slint**:
   ```powershell
   cargo check --workspace
   ```
   Garante que `NoProfilesDialog` seja corretamente incorporado pelo macro `slint::include_modules!()` e que todas as propriedades e callbacks estejam em conformidade com o compilador Slint.

2. **Verificação dos Testes Unitários do Core**:
   ```powershell
   cargo test --workspace
   ```
   Garante que nenhuma funcionalidade do `core`, `windows_api` ou `tweaks` seja afetada.

3. **Verificação Funcional Manual da Interface**:
   - Iniciar o aplicativo sem nenhum perfil em `config/optimzations/` (ou excluir todos os perfis existentes pela aba Otimizações).
   - Retornar à aba "Painel" (`active-tab == 0`).
   - Clicar no botão "JOGAR CS2".
   - **Critério 1**: O modal deve abrir exibindo exatamente:
     - Texto: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`.
     - Botões: `"Criar Perfil"` e `"Sair"`.
   - **Critério 2**: Clicar em `"Sair"` (ou no botão `"✕"`):
     - O modal fecha.
     - O aplicativo permanece na aba "Painel".
     - Nenhum arquivo é criado em `config/optimzations/`.
     - O jogo CS2 NÃO é iniciado.
     - O botão "JOGAR CS2" permanece habilitado (não fica travado em "INICIANDO...").
   - **Critério 3**: Clicar novamente em "JOGAR CS2" e clicar no botão `"Criar Perfil"`:
     - O modal fecha.
     - A interface navega automaticamente para a aba "Otimizações" (`active-tab == 1`), exibindo os botões de ação para gerar ou criar um perfil.
   - **Critério 4**: Após criar um perfil na aba Otimizações e retornar à aba Painel, clicar em "JOGAR CS2":
     - O modal NÃO deve aparecer.
     - O fluxo normal de lançamento deve ser executado (`is-launching = true;`).
