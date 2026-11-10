# Project: CounterRush — Exclusão de Perfis de Otimização na Interface

## Architecture
- **Framework de UI:** Slint 1.8 (`ui/main.slint`, `ui/components/delete_profile_dialog.slint`, `ui/components/preset_button.slint`, `ui/assets/icons/trash.svg`).
- **Backend Rust:** `core/src/lib.rs` (funções de exclusão no disco com validação de path traversal), `ui/src/main.rs` (bridge Rust <-> Slint e sincronização de estado).
- **Persistência de Perfis:** Arquivos `.json` em `config/optimzations/*.json`.
- **Deteção de Estado:** Reativo via `root.presets.length == 0` (vazio) vs `root.presets.length > 0` (com perfis).

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | F1: Ícone de lixeira em cada perfil | Adicionar botão/ícone de lixeira tático a cada perfil existente em `PresetButton` | M1 | ORIGINAL_REQUEST § R1 |
| 2 | F2: Modal de confirmação | Modal compacto com texto "Deseja realmente excluir o perfil {Nome do perfil}" e botões "Cancelar" e "Excluir" | M1 | ORIGINAL_REQUEST § R3, R4 |
| 3 | F3: Exclusão persistente segura no backend | Função `delete_optimization_profile` no crate `core` com validação de segurança e exclusão do arquivo `.json` | M1 | ORIGINAL_REQUEST § R2 |
| 4 | F4: Reversão para estado padrão vazio | Quando o perfil excluído for o último, redefinir `presets`, `active-preset`, `launch-args` e switches para o padrão neutro | M1 | ORIGINAL_REQUEST § R5 |
| 5 | F5: Tratamento de exclusão do perfil ativo | Se o perfil ativo for excluído e restarem outros, selecionar o primeiro perfil restante e recarregar switches e args | M1 | Relatório Survey 2/3 |
| 6 | F6: Validação e Auditoria Forense | Verificação estática, compilação Slint/Rust, testes unitários, validação visual e auditoria de integridade | M1 | Regras do Projeto / relus.md |
| 7 | F7: Interceptação do botão "Jogar CS2" na ausência de perfis | Ao clicar em "Jogar CS2" sem perfis existentes, impedir o lançamento e abrir o modal de aviso | M2 | ORIGINAL_REQUEST § Follow-up 2026-11-01T18:39:09Z |
| 8 | F8: Modal "NoProfileDialog" | Modal com texto exato "Nenhum Perfil de Otimização encontrado, Deseja criar um?" e botões "Criar Perfil" e "Sair" | M2 | ORIGINAL_REQUEST § Follow-up 2026-11-01T18:39:09Z |
| 9 | F9: Navegação para a aba Otimizações | Botão "Criar Perfil" fecha o modal e redireciona para a aba Otimizações (`active-tab = 1`) | M2 | ORIGINAL_REQUEST § Follow-up 2026-11-01T18:39:09Z |
| 10 | F10: Salvaguarda defensiva no Rust | Barreira em `on_launch_cs2_clicked` impedindo criação espúria de arquivos no disco quando não houver perfis | M2 | Relatórios Explorers 1 e 2 |
| 11 | F11: Seletor Dinâmico de Idiomas | Listar e alternar dinamicamente entre os idiomas da pasta `locale` (`pt_BR`, `en`, `en_US`, `ru`, `ru_RU`) via `slint::select_bundled_translation` | M3 | ORIGINAL_REQUEST § R1 |
| 12 | F12: Modal "Ainda em Desenvolvimento" | Componente `InDevelopmentDialog` tático CS2 com bloqueador de clique e botão "Entendido" | M3 | ORIGINAL_REQUEST § R6 |
| 13 | F13: Toggles Bloqueados (Autostart e Steam) | Controles para inicialização com sistema e otimização Steam bloqueados com badge "EM BREVE", abrindo modal | M3 | ORIGINAL_REQUEST § R2, R3, R6 |
| 14 | F14: Compartilhamento do Projeto | Botão de compartilhamento que copia texto/link para o clipboard nativo com feedback visual no HUD e botão | M3 | ORIGINAL_REQUEST § R4 |
| 15 | F15: Links Externos (GitHub e Site) | Botões para abrir GitHub oficial e Website do projeto no navegador padrão via `ShellExecuteW` | M3 | ORIGINAL_REQUEST § R5 |
| 16 | F16: Persistência de Configurações | Entidade `AppSettings` em `config` salvando e carregando preferências em `config/settings.json` | M3 | Regras de Arquitetura Limpa |
| 17 | F17: Componente Slint AutoOptimizeDialog | Modal overlay 980x620 com TouchArea bloqueador, layout tático CS2 conciso | M4 | ORIGINAL_REQUEST § Condições |
| 18 | F18: Barra de Progresso e Percentual Reativo | Barra com animação fluida e texto de porcentagem 0% a 100% | M4 | ORIGINAL_REQUEST § Descrição |
| 19 | F19: Indicador da Etapa de Otimização | Exibição em tempo real da otimização / calibração sendo aplicada | M4 | ORIGINAL_REQUEST § Descrição |
| 20 | F20: Cancelamento Funcional e Confirmação | Botão Cancelar e 'X' com confirmação prévia; cancelamento real via token atômico e rollback seguro | M4 | ORIGINAL_REQUEST § Condições |
| 21 | F21: Estado de Sucesso Estático com Sinal Verde | Indicador verde de conclusão e botão 'JOGAR CS2' retidos até ação do usuário | M4 | ORIGINAL_REQUEST § Condições |
| 22 | F22: Ação JOGAR CS2 com Redirecionamento | Fecha modal, altera active-tab = 0 (Painel) e executa lançamento automático do CS2 | M4 | ORIGINAL_REQUEST § Descrição |
| 23 | F23: Fechamento Direto por 'X' Pós-Término | Clique em 'X' após 100% fecha o modal diretamente sem mensagem de alerta | M4 | ORIGINAL_REQUEST § Condições |
| 24 | F24: Persistência de Primeira Execução | Rastreamento em AppSettings (config/settings.json) do estado de primeira otimização | M4 | ORIGINAL_REQUEST § Descrição |

## Code Layout
- `config/src/lib.rs`: Campo `initial_optimization_done: bool` em `AppSettings`.
- `core/src/lib.rs`: Implementação de `delete_optimization_profile` e suporte a calibração com cancelamento.
- `ui/assets/icons/trash.svg`: Ícone vetorial Lucide Feather em SVG.
- `ui/components/delete_profile_dialog.slint`: Componente do modal compacto de confirmação de exclusão.
- `ui/components/no_profile_dialog.slint`: Componente do modal de aviso de ausência de perfis para o botão "Jogar CS2".
- `ui/components/in_development_dialog.slint`: Diálogo de recurso em desenvolvimento.
- `ui/components/preset_button.slint`: Layout do perfil com botão de lixeira.
- `ui/components/auto_optimize_dialog.slint`: Componente modal de animação da otimização automática.
- `ui/main.slint`: Integração do `AutoOptimizeDialog`, propriedades reativas de progresso e estado, callback de cancelamento e redirecionamento para o Painel.
- `ui/src/main.rs`: Rotina de execução assíncrona da calibração com feedback de progresso, cancelamento cooperativo via `AtomicBool`, rollback e disparo de `on_launch_cs2_clicked`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Implementação da Exclusão de Perfis de Otimização | Core Backend (`core/src/lib.rs`), UI Slint (`trash.svg`, `delete_profile_dialog.slint`, `preset_button.slint`, `main.slint`), Bridge Rust (`ui/src/main.rs`) e testes | none | DONE |
| 2 | M2: Comportamento do botão "Jogar CS2" sem perfis | UI Slint (`no_profile_dialog.slint`, `main.slint`), Bridge Rust (`ui/src/main.rs`), testes e auditoria forense | M1 | DONE |
| 3 | M3: Aba de Configurações (Settings) | UI Slint (`in_development_dialog.slint`, `main.slint`), i18n (`locale/`), persistência (`config/src/lib.rs`), core backend (`core/src/lib.rs`), Win32 clipboard (`windows_api`), bridge Rust (`ui/src/main.rs`) | none | DONE |
| 4 | M4: Modal de Animação de Otimização Automática e Lançamento CS2 | UI Slint (`auto_optimize_dialog.slint`, `main.slint`), Config (`config/src/lib.rs`), Bridge Rust (`ui/src/main.rs`) com cancelamento atômico, rollback e E2E tests | M1, M2, M3 | DONE |

## Interface Contracts
### `ui/components/preset_button.slint` ↔ `ui/main.slint`
- `callback delete();`: Disparado ao clicar na lixeira do perfil.

### `ui/components/delete_profile_dialog.slint` ↔ `ui/main.slint`
- `in property <string> profile-name`: Nome do perfil a ser exibido na mensagem.
- `callback cancel();`: Fecha o modal sem excluir.
- `callback confirmed();`: Confirma a exclusão do perfil.

### `ui/components/no_profile_dialog.slint` ↔ `ui/main.slint`
- `callback create-profile();`: Fecha o modal e muda `active-tab = 1;`.
- `callback close();`: Fecha o modal (`show-no-profile-modal = false`).

### `ui/components/auto_optimize_dialog.slint` ↔ `ui/main.slint`
- `in property <float> progress`: Valor normalizado de 0.0 a 1.0 (barra e porcentagem).
- `in property <string> current-step-name`: Descrição da otimização sendo aplicada no instante.
- `in property <bool> is-finished`: Estado indicando 100% concluído.
- `in-out property <bool> show-cancel-confirm`: Sub-alerta de confirmação de cancelamento.
- `callback request-cancel();`: Acionado pelo botão Cancelar ou 'X' durante a execução para abrir sub-alerta.
- `callback confirm-cancel();`: Acionado pelo usuário ao confirmar cancelamento, fecha modal e aborta no backend.
- `callback dismiss-cancel-confirm();`: Fecha apenas o sub-alerta de confirmação mantendo a otimização em andamento.
- `callback play-cs2();`: Acionado no estado final pelo botão "JOGAR CS2", fecha modal, navega para Painel e inicia CS2.
- `callback close();`: Acionado pelo 'X' no estado final, fechando imediatamente sem confirmação.

### `ui/main.slint` ↔ `ui/src/main.rs`
- `in-out property <bool> show-no-profile-modal: false`: Visibilidade do modal de ausência de perfis.
- `in-out property <bool> show-auto-optimize-modal: false`: Visibilidade do modal de otimização automática.
- `in-out property <float> auto-optimize-progress: 0.0`: Progresso reativo da calibração (0.0 a 1.0).
- `in-out property <string> auto-optimize-step-name: ""`: Descrição textual da etapa corrente.
- `in-out property <bool> auto-optimize-finished: false`: Flag de conclusão da calibração.
- `callback auto-optimize-clicked()`: Inicia o processo assíncrono em thread com token de cancelamento.
- `callback cancel-auto-optimize-clicked()`: Sinaliza `cancel_token.store(true)` para abortar e reverter tweaks.
- `ui.set_presets(...)`: Atualiza lista de perfis gerada pelo `AutoDetect.json`.

