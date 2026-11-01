# Lista de Tarefas (Task List) — CounterRush UI

- [x] **Tarefa 1: Atualização do botão de adição de perfil (`ui/components/add_preset_button.slint`)**
  - [x] Alterar o texto de `"NOVO PERFIL"` para `"Criar Otimização"` no estado `is-empty: true`.
  - [x] Ajustar largura para 210px quando vazio, mantendo ícone `+` e ação de clique.
- [x] **Tarefa 2: Refatoração do layout da aba de Otimizações (`ui/main.slint`)**
  - [x] Remover o botão `TacticalButton` "OTIMIZAR AUTOMATICAMENTE" da parte inferior da tela (R2).
  - [x] Reposicionar o bloco do input de parâmetros de inicialização para ficar diretamente abaixo das otimizações ativas (13 switches) e antes da seção de perfis (R3).
  - [x] Implementar a exibição condicional exclusiva dos dois botões de ação ("Otimizar Automaticamente" e "Criar Otimização +") no estado `root.presets.length == 0` (R1).
  - [x] Garantir que quando `root.presets.length > 0`, a listagem de perfis e a seleção funcionem normalmente (R4).
- [x] **Tarefa 3: Verificação e Auditoria**
  - [x] Checagem de compilação com `cargo check -p counterrush-ui` (Aprovado, exit code 0).
  - [x] Revisão independente por 2 Reviewers (Ambos APPROVE).
  - [x] Verificação empírica por 2 Challengers (Ambos CONFIRM).
  - [x] Auditoria Forense com Forensic Auditor (`teamwork_auditor` - CLEAN).
- [x] **Tarefa 4: Validação Visual com Hot Reload**
  - [x] Validar visualmente o estado sem arquivos `.json` via `preview.cmd` / Slint Viewer (confirmado via `screenshot_empty_state_tab1.png`).
  - [x] Validar visualmente o estado com arquivos `.json` via `preview.cmd` / Slint Viewer (confirmado via `screenshot_filled_state_tab1.png`).
  - [x] Verificação dos testes unitários do workspace (`cargo test` 13/13 aprovados).
  - [x] Geração do Walkthrough final em Português-BR.
