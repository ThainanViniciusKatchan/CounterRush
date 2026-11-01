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

## Code Layout
- `core/src/lib.rs`: Implementação de `delete_optimization_profile` e `delete_optimization_profile_in_dir`, testes unitários.
- `ui/assets/icons/trash.svg`: Ícone vetorial Lucide Feather em SVG.
- `ui/components/delete_profile_dialog.slint`: Componente do modal compacto de confirmação de exclusão.
- `ui/components/no_profile_dialog.slint`: Componente do modal de aviso de ausência de perfis para o botão "Jogar CS2".
- `ui/components/preset_button.slint`: Layout do perfil com botão de lixeira.
- `ui/main.slint`: Integração do diálogo `NoProfileDialog`, interceptação no botão "JOGAR CS2", navegação para a aba Otimizações e controle de `show-no-profile-modal`.
- `ui/src/main.rs`: Salvaguarda defensiva em `on_launch_cs2_clicked`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Implementação da Exclusão de Perfis de Otimização | Core Backend (`core/src/lib.rs`), UI Slint (`trash.svg`, `delete_profile_dialog.slint`, `preset_button.slint`, `main.slint`), Bridge Rust (`ui/src/main.rs`) e testes | none | DONE |
| 2 | M2: Comportamento do botão "Jogar CS2" sem perfis | UI Slint (`no_profile_dialog.slint`, `main.slint`), Bridge Rust (`ui/src/main.rs`), testes e auditoria forense | M1 | DONE |

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

### `ui/main.slint` ↔ `ui/src/main.rs`
- `in-out property <bool> show-no-profile-modal: false;`: Visibilidade do modal de ausência de perfis.
- `callback delete-profile-confirmed(string)`: Envia o nome do perfil confirmado para exclusão no backend.
- `ui.set_presets(ModelRc::new(VecModel::from(scan_presets())))`: Atualiza a lista reativa de perfis.
- Em `on_launch_cs2_clicked`: Se `presets.is_empty()`, cancela o lançamento, reseta `is_launching = false` e abre `show-no-profile-modal = true`.
