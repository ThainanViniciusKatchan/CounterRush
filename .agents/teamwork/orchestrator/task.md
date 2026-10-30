# Lista de Tarefas (Task List) — CounterRush

## Fase 0: Mapeamento e Governança (Concluída)
- [x] Leitura estrita de `ORIGINAL_REQUEST.md` e regras do projeto
- [x] Investigação técnica read-only do backend de configuração (`teamwork_explorer_survey_1`)
- [x] Investigação técnica read-only da arquitetura Slint UI e pontes Rust (`teamwork_explorer_survey_2`)
- [x] Investigação técnica read-only de testes automatizados e runtime (`teamwork_explorer_survey_3`)
- [x] Elaboração do Plano de Implementação (`implementation_plan.md`) e Lista de Tarefas (`task.md`) em Português-BR
- [x] Submissão e APROVAÇÃO FORMAL do Plano e Lista de Tarefas pelo usuário

---

## Fase 1: Backend de Configuração e Estado Inicial Neutro (R1) (Concluída)
- [x] Adicionar método `all_disabled()` e derivação `PartialEq, Eq` em `CustomOptimizationConfig` (`config/src/lib.rs`)
- [x] Remover injeção de perfis fictícios em `scan_presets()` (`ui/src/main.rs`)
- [x] Condicionar inicialização em `main()` para não criar arquivo default em disco quando a pasta estiver vazia (`ui/src/main.rs`)
- [x] Configurar propriedades padrão neutras na interface Slint (`ui/main.slint`)
- [x] Verificar compilação com `cargo check`

---

## Fase 2: Botão "+" e Modal de Criação de Perfil (R2 e R3) (Concluída)
- [x] Criar componente visual `AddPresetButton` (`ui/components/add_preset_button.slint`)
- [x] Criar componente de modal overlay `CreateProfileDialog` (`ui/components/create_profile_dialog.slint`)
- [x] Posicionar o botão "+" imediatamente após o último perfil no `HorizontalLayout` de `ui/main.slint`
- [x] Integrar o modal overlay `CreateProfileDialog` na janela principal `ui/main.slint`
- [x] Implementar função de validação de nome de perfil em `core/src/lib.rs`
- [x] Conectar callback de criação no backend Rust (`ui/src/main.rs`): salvar `.json`, atualizar presets, selecionar novo perfil e sincronizar switches
- [x] Verificar compilação com `cargo check`

---

## Fase 3: Persistência de `.json`, seleção ativa e sincronização de switches (R3) (Concluída)
- [x] Gravação atômica via `save_to_file` em `config/optimzations/<nome>.json`
- [x] Recarregamento reativo da lista de presets e ativação imediata do novo perfil
- [x] Sincronização em tempo real de todos os 13 switches com a UI Slint
- [x] Validação em Gate do Milestone 1 (Reviewer 1 APPROVE, Reviewer 2 APPROVE, Challenger 1 APPROVE, Challenger 2 APPROVE, Auditor CLEAN)

---

## Fase 4: Testes Automatizados e Validação em Tempo de Execução (R4) (Concluída)
- [x] Adicionar dependência `tempfile` nos manifestos do Cargo (`Cargo.toml`, `core/Cargo.toml`, `config/Cargo.toml`)
- [x] Configurar `[lib] doctest = false` em `core/Cargo.toml` para sanar colisão de doctests
- [x] Implementar testes unitários no crate `config` cobrindo estado desligado, serialização/desserialização, UTF-8 BOM e JSON inválido (6 testes)
- [x] Implementar testes unitários no crate `core` cobrindo validação de nomes (incluindo sufixo case-insensitive), listagem em pasta vazia e persistência (3 testes)
- [x] Executar suíte de testes com `cargo test --workspace` (100% aprovado, 9 testes, exit code 0)
- [x] Executar validação funcional de runtime do binário compilado sem crashes ou panics (`counterrush-ui.exe`)

---

## Fase 5: Auditoria de Integridade e Revisão Final (Concluída)
- [x] Revisão de código independente via Reviewers (Reviewer 1 APPROVE, Reviewer 2 APPROVE)
- [x] Desafio adversarial de cobertura e estresse de runtime via Challengers (Challenger 1 APPROVE, Challenger 2 APPROVE)
- [x] Auditoria Forense de Integridade via Auditor independente (`teamwork_auditor` CLEAN)
- [x] Envio do Walkthrough final em Português-BR para o usuário
