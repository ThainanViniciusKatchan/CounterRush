# Relatório de Revisão de Código e Análise Adversarial — Milestone 1

**Revisor:** `teamwork_reviewer_m1_1` (Code Reviewer & Adversarial Critic)  
**Data:** 2026-10-30  
**Milestone:** Milestone 1 (R1, R2, R3)  
**Alvo da Revisão:** Alterações submetidas por `teamwork_worker_m1_1`  
**Veredito Final:** **APPROVE**  

---

## 1. Observation (O que foi observado)

### 1.1. Inspeção Detalhada do Código Modificado e Criado
Foram inspecionados diretamente os 6 arquivos de código envolvidos no escopo do Milestone 1:

1. **`config/src/lib.rs`**:
   - Linha 20: Derivação de traços estendida: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`.
   - Linhas 60-78: Construtor público `CustomOptimizationConfig::all_disabled()` inicializando `launch_args: Vec::new()` e todas as 13 flags booleanas de otimização estritamente como `false`.
   - Linhas 99-109: Método `save_to_file()` garantindo criação recursiva de diretórios pais (`std::fs::create_dir_all(parent)`).

2. **`core/src/lib.rs`**:
   - Linhas 825-874: `resolve_optimizations_dir()` prioriza o diretório canônico `config/optimzations` relativo ao executável ou workspace.
   - Linhas 876-889: `list_optimization_configs()` lista arquivos `.json` ordenados alfabeticamente.
   - Linhas 891-899: `resolve_optimization_file_path(name)` anexa com segurança `.json` caso não esteja presente.
   - Linhas 901-932: `validate_profile_name(name)` implementa sanitização rigorosa:
     - Rejeita strings vazias ou apenas espaços em branco;
     - Trata sufixos redundantes `.json` e `.JSON`;
     - Rejeita caracteres ilegais no Windows (`\ / : * ? " < > |`);
     - Rejeita nomes reservados de periféricos NTFS (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`).

3. **`ui/components/add_preset_button.slint`**:
   - Linha 4: Propriedade booleana `is-empty`.
   - Linhas 7-8: Altura padrão de 44px (idêntica ao `PresetButton`). Largura adaptativa: `is-empty ? 180px : 44px`.
   - Linhas 18-41: Quando vazio (`is-empty == true`), exibe texto auxiliar `"+ NOVO PERFIL"` com estética CS2Theme e transições animadas; quando há perfis (`is-empty == false`), recolhe para botão quadrado tático "+".
   - Linhas 43-48: `TouchArea` configurado com `mouse-cursor: pointer` e emissão do callback `clicked()`.

4. **`ui/components/create_profile_dialog.slint`**:
   - Linhas 3-19: Declaração da struct de transferência de dados `NewProfileData` contendo `name: string`, `launch-args: string` e as 13 flags booleanas individuais.
   - Linhas 21-75: Subcomponente `ProfileTweakCheck` tático com checkbox estilizado ("✓") e área de toque reativa.
   - Linhas 77-106: Diálogo overlay modal em tamanho total (980x620px), fundo escuro translúcido (`rgba(11, 15, 23, 0.88)`), e bloqueador de cliques `TouchArea` que consome todos os eventos de mouse, impedindo interação involuntária com a janela principal ao fundo.
   - Linhas 109-268: Card centralizado (720x540px) com cabeçalho, botão rápido de fechamento "✕", campo de entrada para nome do arquivo `.json` com indicação de foco e campo para argumentos Steam (`launch_args`).
   - Linhas 271-435: Seção dos 13 switches organizados em grade de 2 colunas, acompanhados dos botões de conveniência `"MARCAR TODOS"` e `"DESMARCAR TODOS"`.
   - Linhas 438-531: Rodapé com exibição de mensagem de erro reativa (`error-message != ""`), botão `"CANCELAR"` e botão primário `"CRIAR E ATIVAR PERFIL"`.

5. **`ui/main.slint`**:
   - Linhas 27-43: Propriedades padrão da `MainWindow` em estado estritamente neutro: todos os 13 switches inicializados em `false`, `active-preset: ""`, `presets: []`.
   - Linhas 42-43: Propriedades de controle `show-create-modal: false` e `create-modal-error: ""`.
   - Linhas 724-762: `HorizontalLayout` centralizado contendo o loop `for preset in root.presets : PresetButton` imediatamente seguido de `AddPresetButton`.
   - Linhas 943-953: Renderização condicional do diálogo `if root.show-create-modal : CreateProfileDialog`.

6. **`ui/src/main.rs`**:
   - Linhas 5-22: `scan_presets()` varre exclusivamente os arquivos reais de `config/optimzations`, sem injeção fictícia de perfis não existentes.
   - Linhas 31-48: Tratamento condicional para diretório vazio (`initial_presets.is_empty()`):
     - `active-preset` definido como `""`;
     - Todos os 13 switches desativados (`false`);
     - Caminho ativo inicializado como `PathBuf::new()` e configuração como `CustomOptimizationConfig::all_disabled()`;
     - Nenhuma chamada a `load_or_create_custom_config()`, prevenindo criação de arquivos no disco antes da ação do usuário.
   - Linhas 120-122: Thread de sincronização periódica de disco ignora caminhos vazios (`current_path.as_os_str().is_empty()`), evitando acessos desnecessários de I/O.
   - Linhas 346-351: No callback `on_tweak_toggled`, se nenhum perfil estiver ativo, emite aviso na barra de status HUD sem forçar criação prematura de arquivo.
   - Linhas 513-583: Callback `on_create_profile_confirmed`:
     - Valida o nome via `counterrush_core::validate_profile_name`; em caso de erro, atualiza `create-modal-error` e mantém o modal aberto;
     - Mapeia os 13 switches e tokeniza `launch_args` com `split_whitespace()`;
     - Grava o arquivo `.json` em `config/optimzations/<clean_name>.json`;
     - Atualiza o `active_config_path`;
     - Atualiza o modelo de presets e define o novo perfil como `active_preset`;
     - Sincroniza todos os 13 switches da janela principal;
     - Fecha o modal e exibe confirmação na barra HUD.

### 1.2. Verificação de Compilação
- **Comando:** `cargo check --workspace`
- **Ambiente:** Windows PowerShell / Cargo
- **Resultado:**
  ```text
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.10s
  ```
- **Código de saída:** 0 (Zero erros, zero advertências).

---

## 2. Logic Chain (Linha de Raciocínio Passo a Passo)

1. **Atendimento de R1 (Estado Inicial Neutro e Desligado)**:
   - *Premissa:* Na primeira execução com diretório `config/optimzations` vazio, nenhuma otimização deve iniciar ativa, nenhum perfil fictício deve ser considerado corrente e nenhum arquivo deve ser gerado no disco sem comando do usuário.
   - *Verificação:* Em `ui/src/main.rs:32-48`, se `initial_presets.is_empty()`, `main()` não chama o método gerador de arquivo, seta `active_preset: ""` e define os 13 switches como `false`. Em `ui/main.slint:27-41`, os valores default também são `false`. O requisito R1 está plenamente satisfeito.

2. **Atendimento de R2 (Posicionamento e Ação do Botão "+")**:
   - *Premissa:* O botão "+" deve estar visível e centralizado quando a pasta estiver vazia, e posicionado imediatamente após o último perfil caso existam perfis. Clicar no botão deve abrir a interface de criação.
   - *Verificação:* `AddPresetButton` está inserido no `HorizontalLayout` com `alignment: center` imediatamente após `for preset in root.presets`. Se `presets.length == 0`, o botão assume largura de 180px com texto e fica centralizado. Se `presets.length > 0`, os botões de perfil ocupam o início e o botão "+" (em 44x44px compacto) posiciona-se logo à direita do último item. O clique aciona `show-create-modal = true`. O requisito R2 está plenamente satisfeito.

3. **Atendimento de R3 (Modal de Criação, Validação e Persistência de Perfil .json)**:
   - *Premissa:* Interface in-window contendo nome, argumentos Steam e 13 switches. Validação de nome, persistência em `config/optimzations/<nome>.json` via `CustomOptimizationConfig`, seleção imediata como ativo e sincronização dos switches.
   - *Verificação:* O componente `CreateProfileDialog` atende 100% dos campos e opções. `validate_profile_name` protege contra nomes ilegais. Ao confirmar, o arquivo é salvo em `config/optimzations/<clean_name>.json`, a lista de botões é recarregada, o novo perfil é marcado como ativo, os 13 switches principais são sincronizados e o modal é fechado. O requisito R3 está plenamente satisfeito.

4. **Integridade da Implementação (Anti-Cheat & No-Facades)**:
   - Não foram encontrados valores mocados, stubs fictícios, desvios maliciosos ou bypasses de regras. Toda a lógica de leitura, validação, serialização JSON e sincronização com Slint é genuína e operacional.

---

## 3. Caveats & Análise Adversarial (Cenários de Borda e Resiliência)

### 3.1. Cenários de Borda Avaliados

| Cenário de Teste / Stress | Comportamento Observado | Classificação | Avaliação |
| :--- | :--- | :--- | :--- |
| **Tentativa de nome vazio ou com espaços** (`""`, `"   "`) | Rejeitado por `validate_profile_name`, exibe erro no modal sem fechá-lo | Protegido | Aprovado |
| **Tentativa de caracteres ilegais** (`*`, `:`, `"`, `\`, `/`, `?`, `<`, `>`, `\|`) | Rejeitado por `validate_profile_name`, exibe erro no modal sem fechá-lo | Protegido | Aprovado |
| **Tentativa de nomes de periféricos NTFS** (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`) | Rejeitado pela lista de palavras reservadas do Windows em maiúsculas | Protegido | Aprovado |
| **Cliques fora do card do modal durante exibição** | `TouchArea` bloqueador no fundo (980x620) consome cliques e impede acionamento involuntário dos controles da janela | Protegido | Aprovado |
| **Reabertura do modal após fechamento** | Slint recria a instância sob condição booleana, redefinindo os campos para o estado padrão limpo | Protegido | Aprovado |
| **Extensão com caixa mista** (ex: `perfil.Json`) | `strip_suffix` trata `.json` e `.JSON`, mas não `.Json`. O resolvedor de caminhos gera `perfil.Json.json` | Oportunidade Menor | Baixo Risco |
| **Tentativa de clicar em "INICIAR CS2" sem nenhum perfil criado** | `target_cfg_path` vazio faz o resolvedor gerar `config/optimzations/json` | Oportunidade Menor | Baixo Risco |

### 3.2. Mitigações Recomendadas para o Milestone 2
- **Cenário de Lançamento sem Perfil:** No callback `on_launch_cs2_clicked` em `ui/src/main.rs`, caso `target_cfg_path.as_os_str().is_empty()`, adicionar uma cláusula de guarda emitindo: `"Crie ou selecione um perfil de otimização antes de iniciar o CS2."`.
- **Sanitização de Extensão Case-Insensitive:** Em `validate_profile_name`, utilizar verificação case-insensitive completa para sufixos `.json`.

---

## 4. Review Summary

**Veredito:** **APPROVE**

### Achados (Findings)

#### [Minor] Oportunidade de Guarda no Lançamento com Diretório Vazio
- **O quê:** Se o usuário clicar em "INICIAR CS2 OTIMIZADO" antes de criar qualquer perfil, o fallback tenta resolver um caminho vazio, resultando em `config/optimzations/json`.
- **Onde:** `ui/src/main.rs:233-248`
- **Por quê:** O fluxo principal esperado pelo usuário é criar um perfil antes de iniciar, mas uma guarda explícita evita a criação acidental de um arquivo chamado `json`.
- **Sugestão:** Implementar verificação de `target_cfg_path.as_os_str().is_empty()` no Milestone 2.

#### [Minor] Tratamento de Extensão com Variações de Caixa
- **O quê:** Inserir `meuperfil.Json` pode gerar `meuperfil.Json.json` pois a sanitização foca em `.json` e `.JSON`.
- **Onde:** `core/src/lib.rs:907-913`
- **Por quê:** Pequena conveniência de validação para o usuário final.
- **Sugestão:** Ajustar para `strip_suffix` com lowercase no Milestone 2.

### Reivindicações Verificadas (Verified Claims)
- [x] Estado inicial sem perfis mantém os 13 switches desativados (`false`) → Verificado no código Slint e Rust.
- [x] Nenhum arquivo é gerado no disco na inicialização quando a pasta está vazia → Verificado em `main()` e na thread de sincronização.
- [x] O botão "+" é exibido centralizado quando não há perfis → Verificado no layout Slint e propriedade `is-empty`.
- [x] O botão "+" posiciona-se imediatamente após o último perfil quando existem perfis → Verificado na estrutura do `HorizontalLayout`.
- [x] O modal in-window permite informar nome, `launch_args` e 13 switches → Verificado em `CreateProfileDialog.slint`.
- [x] O perfil criado é salvo em `config/optimzations/<nome>.json` e ativado imediatamente → Verificado em `on_create_profile_confirmed`.
- [x] O workspace compila com zero erros e zero avisos → Verificado via `cargo check --workspace` (código de saída 0).

---

## 5. Conclusion (Conclusão)

As alterações entregues no **Milestone 1 (R1, R2, R3)** cumprem com excelência todos os requisitos do projeto CounterRush e critérios de aceite estabelecidos no `ORIGINAL_REQUEST.md` e no plano de implementação aprovado.
- Código limpo, desacoplado e aderente aos princípios SOLID e Clean Architecture.
- Zero violações de integridade ou implementações de fachada.
- Compilação 100% aprovada.

A base de código está validada e pronta para avançar para o **Milestone 2 (R4: Testes Automatizados e Resolução de Doctest)**.

---

## 6. Verification Method (Método de Verificação Independente)

Para reproduzir a verificação independente:

1. **Checagem de Compilação do Workspace**:
   ```powershell
   cargo check --workspace
   ```
   *Resultado esperado:* Exit code 0, 0 erros, 0 warnings.

2. **Inspeção Estática dos Arquivos de Código**:
   - `config/src/lib.rs`: método `all_disabled()` e derivação `PartialEq, Eq`.
   - `core/src/lib.rs`: função `validate_profile_name()`.
   - `ui/components/add_preset_button.slint`: componente visual tático do botão "+".
   - `ui/components/create_profile_dialog.slint`: modal overlay in-window.
   - `ui/main.slint`: layout dos botões e propriedades neutras da janela.
   - `ui/src/main.rs`: lógica de inicialização sem injeção fictícia e callback `on_create_profile_confirmed`.
