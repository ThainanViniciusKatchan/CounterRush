# Task List — Comportamento do Botão "Jogar CS2"

- [x] 1. Criar componente `ui/components/no_profile_dialog.slint`
  - [x] Backdrop bloqueador (980x620px, rgba(11, 15, 23, 0.88))
  - [x] Mensagem: "Nenhum Perfil de Otimização encontrado, Deseja criar um?"
  - [x] Botão "Sair" (callback `close`)
  - [x] Botão "Criar Perfil" (callback `create-profile`)
- [x] 2. Integrar modal e lógica em `ui/main.slint`
  - [x] Importar `NoProfileDialog`
  - [x] Adicionar propriedade `show-no-profile-modal`
  - [x] Interceptar clique do botão "JOGAR CS2" quando `presets.length == 0`
  - [x] Implementar transição de aba (`active-tab = 1`) no callback `create-profile`
- [x] 3. Adicionar salvaguarda defensiva em `ui/src/main.rs`
  - [x] Bloquear lançamento em `on_launch_cs2_clicked` se `scan_presets().is_empty()`
- [x] 4. Verificação de compilação e testes (`cargo check`, `cargo test`)
- [x] 5. Revisão por pares independentes (2 Reviewers — APPROVE)
- [x] 6. Testes adversariais e de borda (2 Challengers — APPROVE)
- [x] 7. Auditoria Forense de Integridade (teamwork_auditor — CLEAN)
- [x] 8. Avaliação do Gate e Relatório Final (PASS)
