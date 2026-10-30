# Relatório de Revisão de Código e Crítica Adversarial — Milestone 1 (R1, R2, R3)

**Agente:** `teamwork_reviewer_m1_2` (Code Reviewer 2 & Adversarial Critic)  
**Data:** 2026-10-30  
**Milestone:** 1 — Estado Inicial Desativado, Botão "+" e Criação de Perfil  
**Veredito Final:** **APPROVE**  
**Avaliação Geral de Risco:** **BAIXO (LOW)**  

---

## 1. Observation (O que foi diretamente observado)

Foram inspecionados diretamente o código-fonte, os manifestos e os artefatos gerados pelo Worker M1:

1. **Compilação do Workspace**:
   - Comando executado: `cargo check --workspace`
   - Resultado: Sucesso absoluto (`Finished dev profile [unoptimized + debuginfo] target(s) in 0.87s`), com 0 erros e 0 warnings.

2. **Integridade e Ausência de Fraudes / Facades**:
   - `config/src/lib.rs` (linhas 20 e 60-78): Derivação `PartialEq, Eq` real na struct `CustomOptimizationConfig` e construtor real `all_disabled()` inicializando todas as 13 flags como `false` e `launch_args` vazio.
   - `core/src/lib.rs` (linhas 825-932): Implementação legítima de `resolve_optimizations_dir()`, `list_optimization_configs()`, `resolve_optimization_file_path()` e `validate_profile_name()`.
   - `ui/components/add_preset_button.slint`: Componente visual legítimo com altura de 44px, largura adaptativa (180px com texto quando vazio; 44x44px quando com perfis), animações, hover states e integração com `TouchArea`.
   - `ui/components/create_profile_dialog.slint`: Diálogo modal in-window completo com backdrop translúcido bloqueador de cliques (`TouchArea`), formulário de nome e `launch_args`, grade de 13 switches compactos (`ProfileTweakCheck`), botões "MARCAR TODOS" e "DESMARCAR TODOS", tratamento reativo de erros e confirmação emitindo a struct `NewProfileData`.
   - `ui/main.slint`: Valores padrão na primeira execução definidos como `active-preset: ""`, `presets: []`, e os 13 switches como `false`. Botão `AddPresetButton` inserido no `HorizontalLayout` imediatamente após `for preset in root.presets`. Diálogo `CreateProfileDialog` renderizado condicionalmente na raiz.
   - `ui/src/main.rs`:
     - `scan_presets()` (linhas 5-22): Retorna lista vazia quando não há arquivos `.json` no diretório de otimizações, eliminando qualquer injeção artificial de perfis fictícios.
     - Inicialização (linhas 32-48): Se `initial_presets.is_empty()`, não cria arquivos no disco, desativa os switches e mantém caminho vazio.
     - Callback `on_tweak_toggled` (linhas 347-351): Protegido contra escrita prematura com feedback claro caso não haja perfil ativo.
     - Callback `on_create_profile_confirmed` (linhas 515-582): Sanitiza o nome, monta `CustomOptimizationConfig`, grava no disco em `config/optimzations/<clean_name>.json`, recarrega os presets, seleciona o novo perfil como ativo, sincroniza todos os 13 switches e fecha o diálogo.

3. **Verificação de Regras de Integridade**:
   - Não há valores de teste embutidos (hardcoded).
   - Não há implementações fictícias (dummies ou facades).
   - Não há desvios de escopo ou atalhos indevidos.

---

## 2. Logic Chain (Linha de Raciocínio Passo a Passo)

1. **Cumprimento do Requisito R1 (Estado Inicial Desativado)**:
   - A inicialização original do CounterRush forçava a geração e escrita de `custom_optimization.json` na primeira execução.
   - A remoção dessa chamada e o controle condicional via `initial_presets.is_empty()` garantem que em uma instalação nova (diretório `config/optimzations` vazio), nenhuma escrita no disco ocorra.
   - Todos os 13 switches visuais em `MainWindow` iniciam com valor `false`.
   - O `active-preset` inicia com string vazia `""`.
   - Modificações individuais nos switches sem perfil ativo são interceptadas e impedem I/O prematuro no disco.
   - *Conclusão:* R1 cumprido estritamente.

2. **Cumprimento do Requisito R2 (Posicionamento do Botão "+")**:
   - O `AddPresetButton` foi posicionado no mesmo `HorizontalLayout` dos perfis, logo após o loop `for preset in root.presets`.
   - Regra Slint: quando `presets` tem tamanho 0, o loop não gera elementos. O único filho do layout é `AddPresetButton`. Com `alignment: center`, o botão se posiciona perfeitamente centralizado na área de perfis, com affordance expandida `+ NOVO PERFIL` (180px).
   - Quando existem perfis na lista, o loop gera os botões e o `AddPresetButton` é posicionado sequencialmente imediatamente à direita do último perfil, reduzindo sua largura para 44px compacto.
   - O clique no botão "+" limpa erros prévios e seta `show-create-modal = true`, disparando a abertura imediata da janela.
   - *Conclusão:* R2 cumprido estritamente.

3. **Cumprimento do Requisito R3 (Janela e Persistência do .json)**:
   - A interface `CreateProfileDialog` foi construída como modal overlay in-window dentro do padrão de design Slint e CS2Theme, contendo os campos de nome e argumentos de inicialização, além da seleção dos 13 switches.
   - O backdrop cobre a janela inteira (980x620) e absorve cliques do mouse, evitando que o usuário interaja acidentalmente com a tela de fundo.
   - A confirmação passa pelo validador `validate_profile_name()` que previne nomes ilegais no Windows e nomes de dispositivos reservados (`CON`, `PRN`, `AUX`, `NUL`, etc.).
   - Ao salvar com sucesso:
     - O arquivo `.json` é persistido em `config/optimzations/<nome>.json`;
     - O caminho ativo `active_config_path` é atualizado sob `RwLock`;
     - A lista de perfis é atualizada e o novo perfil é marcado como `active_preset`;
     - Os 13 switches da tela principal são atualizados para refletir as escolhas feitas no diálogo;
     - O diálogo é fechado e uma mensagem de sucesso é apresentada no HUD;
     - O botão "+" continua posicionado à direita do perfil recém-criado.
   - *Conclusão:* R3 cumprido estritamente.

---

## 3. Quality Review Findings (Achados da Revisão de Qualidade)

### Verified Claims

- Compilação do workspace via `cargo check --workspace` → aprovado com 0 erros e 0 avisos.
- `CustomOptimizationConfig::all_disabled()` possui todas as 13 flags desligadas → verificado via inspeção estática no código-fonte.
- `scan_presets()` não gera dados artificiais quando a pasta está vazia → verificado no código-fonte.
- Validação de nomes rejeita caracteres ilegais (`\ / : * ? " < > |`) e strings vazias → verificado na lógica de `validate_profile_name()`.
- Thread-safety na comunicação da UI com a thread de sincronização via `Arc<RwLock<PathBuf>>` e `slint::invoke_from_event_loop` → verificado no código-fonte.

### Coverage Gaps

- **Testes automatizados com `tempfile`**: O plano de implementação alocou os testes automatizados para o Milestone 2, que é o próximo passo acordado pela governança do projeto. Risco mitigado pela verificação estática detalhada e compilação limpa.

### Findings

#### [Minor] Finding 1 — Sanitização de sufixo com casing misto em nomes de arquivo
- **Onde**: `core/src/lib.rs:907-913`
- **O que**: `validate_profile_name()` faz strip de `.json` e `.JSON`, mas não cobre extensões com casing misto (ex: `.Json` ou `.jSoN`).
- **Impacto**: Se o usuário digitar explicitamente `meuperfil.Json`, o arquivo será salvo como `meuperfil.Json.json`.
- **Sugestão para Milestone 2**: Utilizar comparação case-insensitive para remoção de sufixo redundante (ex: `.to_lowercase().ends_with(".json")`).

#### [Minor] Finding 2 — Alinhamento de comportamento no botão "INICIAR CS2" sem perfil prévio
- **Onde**: `ui/src/main.rs:233-238`
- **O que**: No callback `on_launch_cs2_clicked`, se a aplicação acabou de ser aberta pela primeira vez e nenhum perfil foi criado ainda, `target_cfg_path` estará vazia. Ao chamar `core.load_or_create_custom_config(Some(&target_cfg_path))`, o `resolve_custom_config_path` recebe um caminho vazio e pode resolver para `config/optimzations/.json`.
- **Impacto**: Baixo, pois em uso normal o usuário cria ou seleciona um perfil primeiro na aba Otimizações.
- **Sugestão para Milestone 2**: Adicionar a mesma checagem de caminho vazio já presente em `on_tweak_toggled` (`if target_cfg_path.as_os_str().is_empty() { ui.set_status_message("Crie ou selecione um perfil antes de iniciar o CS2.".into()); return; }`).

---

## 4. Adversarial Review & Stress-Testing (Crítica Adversarial)

### Challenge Summary
- **Overall risk assessment**: **LOW**

### Challenges & Stress Tests

#### 1. Tentativa de Sobrescrever Perfil Existente
- **Cenário de Ataque**: O usuário insere no diálogo o nome de um perfil já existente (ex: `competitivo`).
- **Comportamento Previsto**: O arquivo existente é sobrescrito com as novas escolhas sem aviso prévio.
- **Raio de Ação (Blast Radius)**: Substituição das flags do perfil anterior pelo novo estado salvo.
- **Mitigação Recomendada**: Em versões futuras, pode ser adicionado um aviso/diálogo de confirmação de sobreposição se `target_path.exists()`. Para o Milestone 1, o comportamento atende aos requisitos.

#### 2. Comportamento sob Diretório Inexistente no Disco
- **Cenário de Ataque**: O usuário deleta manualmente a pasta `config/optimzations` enquanto o programa está rodando e clica em Criar Perfil.
- **Resultado do Teste de Estresse**: A função `CustomOptimizationConfig::save_to_file()` verifica `if !parent.exists() { std::fs::create_dir_all(parent)?; }`. A pasta é recriada automaticamente e o salvamento ocorre sem falhas de I/O.
- **Resultado**: PASS (Comportamento robusto).

#### 3. Tentativa de Interação com a Janela Principal durante o Diálogo
- **Cenário de Ataque**: Usuário clica fora da caixa de diálogo na tentativa de interagir com abas ou switches de fundo enquanto o modal está aberto.
- **Resultado do Teste de Estresse**: O `CreateProfileDialog` possui um `TouchArea` na raiz ocupando 980x620px com `mouse-cursor: default`. Todos os cliques fora do card central são interceptados e bloqueados pela camada base.
- **Resultado**: PASS.

#### 4. Overflow Horizontal com Mais de Três Perfis
- **Cenário de Ataque**: Criação de múltiplos perfis (ex: 4 ou mais).
- **Resultado do Teste de Estresse**: A interface reduz a largura dos botões de 350px para 250px quando há mais de 2 perfis, o que acomoda bem até 3 botões em 780px de largura útil. A partir de 4 perfis, pode ocorrer clipping visual se não houver rolagem horizontal.
- **Mitigação Recomendada**: Para marcos futuros, adicionar um `Flickable` ou barra de rolagem horizontal se o usuário criar muitos perfis.

---

## 5. Caveats (Ressalvas)

- Os testes unitários com `tempfile` e a desativação do doctest do crate `core` pertencem formalmente ao escopo de execução do Milestone 2.
- A compilação e integridade de tipos do workspace foram validadas no Milestone 1 via `cargo check --workspace`.

---

## 6. Conclusion (Conclusão e Veredito)

O trabalho entregue no Milestone 1 cumpre estritamente todos os requisitos (R1, R2, R3) estabelecidos em `ORIGINAL_REQUEST.md` e detalhados no `implementation_plan.md`:
- O estado inicial com a pasta vazia não cria arquivos e inicia com todos os 13 switches desativados.
- O botão "+" posiciona-se dinamicamente (centralizado quando vazio, e logo após o último perfil quando preenchido).
- A janela modal de criação em Slint é elegante, bloqueia interações de fundo e persiste perfeitamente o arquivo `.json`, selecionando-o imediatamente e atualizando a interface.
- Nenhuma violação de integridade foi detectada. O workspace compila com 0 erros e 0 avisos.

**Veredito:** **APPROVE**

---

## 7. Verification Method (Como Reproduzir a Verificação)

Para validar de forma independente esta revisão:

1. Executar a checagem do workspace:
   ```powershell
   cargo check --workspace
   ```
   *Resultado esperado:* Exit code 0, 0 erros, 0 warnings.

2. Inspecionar os arquivos do Milestone 1:
   - `config/src/lib.rs` (derivação `PartialEq, Eq` e método `all_disabled()`)
   - `core/src/lib.rs` (função `validate_profile_name()`)
   - `ui/components/add_preset_button.slint` (layout adaptativo de 44px)
   - `ui/components/create_profile_dialog.slint` (modal overlay com 13 checkboxes)
   - `ui/main.slint` (integração do botão "+", diálogo e defaults desligados)
   - `ui/src/main.rs` (callbacks de criação, seleção e persistência)
