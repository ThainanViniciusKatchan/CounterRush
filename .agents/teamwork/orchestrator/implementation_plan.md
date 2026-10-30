# Plano de Implementação (Implementation Plan) — Gestão de Perfis de Otimização e Estado Inicial

**Projeto:** CounterRush  
**Data:** 2026-10-30  
**Versão:** 1.0  
**Status:** Aguardando Aprovação do Usuário (Nenhuma alteração de código realizada até aprovação)  

---

## 1. Visão Geral e Motivações

Este plano estabelece a implementação completa e estrita para os requisitos do CounterRush:
1. **R1 — Estado inicial desativado na primeira execução:** Garantir que quando o programa for iniciado pela primeira vez e nenhum arquivo `.json` existir em `config/optimzations`, todas as otimizações iniciem desligadas (`false`), nenhum perfil seja considerado ativo por padrão, e nenhum arquivo fictício seja gravado indevidamente no disco.
2. **R2 — Botão "+" na área de perfis:** Adicionar um botão tático "+" na listagem de perfis da aba Otimizações. Se não houver perfis, o botão "+" deve aparecer centralizado permitindo criar o primeiro arquivo. Se houver perfis, ele deve se posicionar imediatamente à direita (após) o último perfil existente.
3. **R3 — Janela/Diálogo e persistência de novo perfil (.json):** Implementar interface de diálogo (modal in-window no padrão Slint e CS2Theme) contendo:
   - Campo para nome do arquivo `.json`;
   - Campo para argumentos de inicialização Steam (`launch_args`);
   - Seleção das 13 otimizações ativadas no perfil;
   - Ao confirmar: salvar em `config/optimzations/<nome>.json` compatível com `CustomOptimizationConfig`, selecionar imediatamente como ativo na interface, sincronizar switches e manter o botão "+" posicionado após o novo perfil.
4. **R4 — Testes automatizados e validação em tempo de execução:**
   - Adicionar testes automatizados em Rust cobrindo o estado desativado, serialização/desserialização JSON, validação de nomes e persistência.
   - Resolver o conflito de doctest do crate `core` no `cargo test`.
   - Validar compilação (`cargo check`) e execução controlada da aplicação em runtime sem panics ou travamentos.
5. **R5 — Governança de planejamento e alterações mínimas:** Submissão deste plano e da lista de tarefas para aprovação prévia, alterando estritamente os arquivos necessários.

---

## 2. Análise de Risco e Segurança

> **[ATENÇÃO - COMPORTAMENTO DO SISTEMA DE ARQUIVOS]**: Atualmente, a função `main()` grava prematuramente um arquivo `custom_optimization.json` no disco na primeira execução. A remoção desse comportamento garante o cumprimento estrito do requisito R1, prevenindo escrita não autorizada no disco do usuário antes de uma ação explícita. Não há comandos destrutivos envolvidos.

---

## 3. Arquivos Afetados e Motivações Detalhadas

| Arquivo | Natureza da Alteração | Motivação Explícita da Alteração |
| :--- | :--- | :--- |
| `Cargo.toml` (raiz) | Modificação | Adicionar `tempfile = "3"` na seção `[workspace.dependencies]` para permitir testes de I/O em diretórios temporários isolados sem afetar os arquivos reais do usuário. |
| `core/Cargo.toml` | Modificação | Adicionar `[lib] doctest = false` (evita conflito interno de compilação entre o crate local `core` e `::core` do Rust std) e habilitar `tempfile.workspace = true` para testes. |
| `config/Cargo.toml` | Modificação | Habilitar `tempfile.workspace = true` em `[dev-dependencies]` para testes unitários de serialização em disco. |
| `config/src/lib.rs` | Modificação | 1. Adicionar `#[derive(PartialEq, Eq)]` em `CustomOptimizationConfig` para permitir asserções idiomáticas de igualdade.<br>2. Implementar construtor `CustomOptimizationConfig::all_disabled()` com todas as 13 flags em `false` e `launch_args` vazio.<br>3. Adicionar suíte de testes unitários automatizados (R4). |
| `core/src/lib.rs` | Modificação | 1. Adicionar função de validação e sanitização `validate_profile_name(name: &str) -> Result<String, String>` para impedir caracteres ilegais no Windows (`\ / : * ? " < > \|`) e strings vazias.<br>2. Adicionar suíte de testes unitários automatizados para listagem em diretório vazio e criação de perfil (R4). |
| `ui/components/add_preset_button.slint` | **Criação de novo arquivo** | Componente visual tático isolado para o botão "+" de altura 44px (igual ao `PresetButton`), com estilo CS2Theme, hover states e suporte a largura adaptativa. |
| `ui/components/create_profile_dialog.slint` | **Criação de novo arquivo** | Componente de modal overlay in-window com backdrop translúcido tático, bloqueador de cliques de fundo, campo de texto para nome, campo para `launch_args`, grade tática dos 13 switches compactos, botões Marcar/Desmarcar todos e ações Confirmar/Cancelar. |
| `ui/main.slint` | Modificação | 1. Importar `AddPresetButton` e `CreateProfileDialog`.<br>2. Definir valores default neutros para primeira execução: `active-preset: ""`, `presets: []`, e os 13 switches como `false`.<br>3. Integrar `AddPresetButton` no `HorizontalLayout` logo após o loop `for preset in root.presets`.<br>4. Inserir o overlay `CreateProfileDialog` na raiz da janela.<br>5. Declarar callbacks `create-profile-confirmed` e propriedades de controle do modal. |
| `ui/src/main.rs` | Modificação | 1. Em `scan_presets()`: remover a injeção artificial de perfis falsos (`AutoDetect` e `custom_optimization`) quando o diretório estiver vazio.<br>2. Na inicialização de `main()`: se `initial_presets.is_empty()`, não gravar arquivo default no disco, manter `active-preset` vazio e os switches em `false`.<br>3. Implementar o callback `on_create_profile_confirmed`: sanitizar nome, montar `CustomOptimizationConfig`, gravar arquivo em `config/optimzations/<nome>.json`, recarregar lista, selecionar novo perfil como ativo e sincronizar os 13 switches da interface.<br>4. No callback de toggle individual de switch: tratar o caso de nenhum perfil ativo para feedback claro na barra de status. |

---

## 4. Etapas de Implementação

### Etapa 1: Backend de Configuração e Estado Inicial Neutro (R1)
- Em `config/src/lib.rs`: implementar `all_disabled()` e derivação `PartialEq, Eq`.
- Em `ui/src/main.rs`: limpar injeção artificial de `scan_presets()`.
- Em `ui/src/main.rs`: condicionar a inicialização para não criar arquivo físico nem ativar switches caso não existam perfis.
- Em `ui/main.slint`: ajustar defaults para `false` e `active-preset: ""`.

### Etapa 2: Componentes Slint — Botão "+" e Modal de Criação (R2 e R3)
- Criar `ui/components/add_preset_button.slint`.
- Criar `ui/components/create_profile_dialog.slint`.
- Atualizar `ui/main.slint` importando os componentes e inserindo o botão "+" após o loop de perfis.
- Inserir o modal overlay com controle de exibição `show-create-modal`.

### Etapa 3: Ponte Rust e Persistência do Novo Perfil (R3)
- Em `core/src/lib.rs`: adicionar função auxiliar `validate_profile_name`.
- Em `ui/src/main.rs`: conectar o callback `on_create_profile_confirmed`.
- Gravar o arquivo com `new_config.save_to_file(&target_path)`.
- Atualizar o modelo de presets, selecionar o novo perfil como ativo, atualizar switches da interface e fechar o modal.

### Etapa 4: Testes Automatizados e Validação em Runtime (R4)
- Atualizar manifestos `Cargo.toml`, `core/Cargo.toml` e `config/Cargo.toml`.
- Implementar testes unitários em `config/src/lib.rs`:
  - `test_custom_optimization_config_all_disabled`
  - `test_json_roundtrip_integrity`
  - `test_save_and_load_file`
  - `test_load_with_bom`
  - `test_malformed_json_handling`
- Implementar testes unitários em `core/src/lib.rs`:
  - `test_profile_name_validation`
  - `test_list_optimization_configs_empty_directory`
  - `test_create_and_persist_new_profile`
- Executar `cargo check --workspace` e `cargo test --workspace --lib --tests`.
- Executar validação de runtime da aplicação via script de inicialização controlada.

---

## 5. Critérios de Aceite e Validação

1. **Estado Inicial Sem Perfis:**
   - Com `config/optimzations` vazia: todos os 13 switches desligados (`false`), `active-preset` vazio, nenhum arquivo gerado no disco e botão "+" visível centralizado.
2. **Posicionamento do Botão "+":**
   - Vazio: botão "+" centralizado.
   - Com perfis: botão "+" posicionado imediatamente após o último perfil.
   - Clique no "+" abre o diálogo de criação.
3. **Criação e Salvamento:**
   - Permite preencher nome, `launch_args` e marcar otimizações.
   - Salva em `config/optimzations/<nome>.json`.
   - Novo perfil é selecionado e ativado imediatamente, e os switches da tela principal refletem a seleção.
   - O botão "+" continua posicionado após o novo perfil.
4. **Testes e Verificação:**
   - `cargo check --workspace` com exit code 0.
   - `cargo test --workspace --lib --tests` com 100% dos testes aprovados.
   - Execução estável da aplicação sem panics.
