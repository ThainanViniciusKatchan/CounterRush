# Project: CounterRush - Gestão de Perfis de Otimização e Estado Inicial

## Architecture
- **Padrão Arquitetural**: Clean Architecture / SOLID com separação estrita de responsabilidades:
  - `config`: Modelo de dados `CustomOptimizationConfig`, serialização/desserialização JSON com `serde_json`.
  - `core`: Resolução de caminhos no sistema de arquivos (`config/optimzations`), validação de nomes de arquivos e regras de negócio.
  - `ui`: Interface gráfica em Slint UI (`main.slint`, `add_preset_button.slint`, `create_profile_dialog.slint`) e runtime/callbacks em Rust (`main.rs`).

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Estado Inicial Desativado (R1) | Iniciar com switches desligados (`false`), lista vazia e sem perfil ativo quando não houver arquivos em `config/optimzations` | M1 | ORIGINAL_REQUEST § R1 |
| 2 | Botão "+" na listagem de perfis (R2) | Exibir botão "+" centralizado quando vazio ou imediatamente após o último perfil | M1 | ORIGINAL_REQUEST § R2 |
| 3 | Modal de Criação de Perfil (R3) | Diálogo in-window Slint com nome, launch_args e seleção dos 13 switches | M1 | ORIGINAL_REQUEST § R3 |
| 4 | Persistência e Ativação Imediata (R3) | Gravação de `<nome>.json` em `config/optimzations`, atualização da lista e seleção imediata com sincronização dos switches | M1 | ORIGINAL_REQUEST § R3 |
| 5 | Testes Automatizados Rust (R4) | Testes unitários cobrindo estado inicial, validação de JSON, serialização e persistência | M2 | ORIGINAL_REQUEST § R4 |
| 6 | Validação em Runtime (R4) | Verificação de compilação `cargo check`, testes `cargo test` e execução da GUI sem crash | M2 | ORIGINAL_REQUEST § R4 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Implementação Funcional Completa (R1, R2, R3) | `config/src/lib.rs`, `core/src/lib.rs`, `ui/components/add_preset_button.slint`, `ui/components/create_profile_dialog.slint`, `ui/main.slint`, `ui/src/main.rs` | none | DONE |
| 2 | M2: Testes Automatizados e Validação em Runtime (R4) | `Cargo.toml`, `core/Cargo.toml`, `config/Cargo.toml`, testes unitários em `config` e `core`, `cargo test`, validação de execução | M1 | DONE |

## Code Layout
- `config/src/lib.rs`: `CustomOptimizationConfig::all_disabled()`, derivação `PartialEq, Eq`, testes unitários de serialização/desserialização/BOM/defaults.
- `core/src/lib.rs`: `validate_profile_name()` com case-insensitive, testes de listagem e criação.
- `ui/components/add_preset_button.slint`: Componente tático para o botão "+".
- `ui/components/create_profile_dialog.slint`: Componente modal overlay para criação de perfil.
- `ui/main.slint`: Integração do botão "+", modal overlay e propriedades default neutras.
- `ui/src/main.rs`: Inicialização sem arquivos/perfis fictícios e callback de criação/ativação de perfil.

## Interface Contracts
### `ui/main.slint` ↔ `ui/src/main.rs`
- `callback create_profile_confirmed(NewProfileData)`
- `in-out property <bool> show_create_modal: false;`
- `in-out property <string> create_modal_error: "";`
- `in-out property <string> active_preset: "";`
- `in-out property <[PresetModel]> presets: [];`
