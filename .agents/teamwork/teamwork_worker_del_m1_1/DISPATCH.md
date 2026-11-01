## 2026-11-01T17:44:52Z
Você é o subagente teamwork_worker_del_m1_1.
Sua identidade: teamwork_worker.
Seu diretório de trabalho: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_worker_del_m1_1

LEITURA OBRIGATÓRIA:
Leia o arquivo de solicitação original antes de começar:
g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\ORIGINAL_REQUEST.md

DOCUMENTOS DE ESCOPO E ARQUITETURA:
- g:\Outros computadores\Meu laptop\Desktop\CounterRush\PROJECT.md
- Relatório de UI: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_explorer_del_survey_1\handoff.md
- Relatório de Backend: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_explorer_del_survey_2\handoff.md
- Relatório de Casos de Borda e Build: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_explorer_del_survey_3\handoff.md

REGRAS DO PROJETO E POLÍTICAS:
- Responda em PORTUGUÊS-BR.
- Respeite .agent/rules/relus.md e os princípios de Clean Architecture / SOLID.
- Altere APENAS os arquivos de sua propriedade exclusiva. Não altere nada fora do escopo.
- Arquivos de sua propriedade exclusiva:
  1. core/src/lib.rs
  2. ui/assets/icons/trash.svg
  3. ui/components/delete_profile_dialog.slint
  4. ui/components/preset_button.slint
  5. ui/main.slint
  6. ui/src/main.rs

AVISO OBRIGATÓRIO DE INTEGRIDADE:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

SUA MISSÃO — Milestone 1: Implementação da Exclusão de Perfis de Otimização:
1. Backend Core (`core/src/lib.rs`):
   - Implementar `delete_optimization_profile_in_dir<P: AsRef<Path>>(dir: P, name: &str) -> Result<(), CoreError>` e `delete_optimization_profile(name: &str) -> Result<(), CoreError>`.
   - Validar o nome através de `validate_profile_name(name)` para mitigar ataques de Path Traversal.
   - Adicionar teste unitário robusto em `core/src/lib.rs` criando arquivo em pasta temporária (`tempfile`) e confirmando sua exclusão e comportamento contra arquivos inexistentes.
2. Assets de UI:
   - Criar `ui/assets/icons/trash.svg` no padrão Lucide / Feather do projeto (conforme detalhado no handoff do Explorer 1).
3. Componente Slint Modal (`ui/components/delete_profile_dialog.slint`):
   - Criar o componente compacto com fundo escuro semi-transparente que bloqueia cliques traseiros.
   - Exibir o texto: `"Deseja realmente excluir o perfil " + root.profile-name + "?"`.
   - Incluir os dois botões: "Cancelar" (fecha modal) e "Excluir" (dispara confirmação).
4. Botão de Perfil (`ui/components/preset_button.slint`):
   - Incorporar o botão de lixeira no bloco direito com `del-touch := TouchArea` isolado para que o clique na lixeira dispare `root.delete()` sem selecionar acidentalmente o card.
5. Janela Principal Slint (`ui/main.slint`):
   - Importar `DeleteProfileDialog`.
   - Declarar propriedades `show-delete-modal` e `target-delete-profile`.
   - Declarar `callback delete-profile-confirmed(string);`.
   - Conectar o evento `delete` do `PresetButton` para preencher `target-delete-profile` e abrir o modal.
   - Renderizar `DeleteProfileDialog` quando `show-delete-modal` for verdadeiro.
6. Backend Slint Bridge (`ui/src/main.rs`):
   - Registrar `main_window.on_delete_profile_confirmed`.
   - Executar a exclusão chamando `counterrush_core::delete_optimization_profile(&target_name)`.
   - Atualizar a lista de perfis via `scan_presets()` e `ui.set_presets(...)`.
   - Se a lista estiver vazia:
     - `ui.set_active_preset("".into())`
     - `ui.set_launch_args_text("".into())`
     - `ui.set_is_editing_launch_args(false)`
     - Setar os 13 switches para `false` (`CustomOptimizationConfig::all_disabled()`)
     - Limpar `active_config_path` (`PathBuf::new()`)
   - Se o perfil excluído for o ativo e restarem outros:
     - Selecionar o primeiro perfil restante (`remaining_presets[0]`), carregar sua configuração (`CustomOptimizationConfig::load_from_file`), atualizar `active-preset`, `launch-args-text`, `active_config_path` e os switches correspondentes.
   - Se o perfil excluído for inativo:
     - Manter o perfil ativo intacto.
   - Exibir mensagem na status bar informando sucesso ou erro na exclusão.
7. Verificação:
   - Executar `cargo check --workspace` para validar integridade da compilação.
   - Executar `cargo test -p core` para validar testes unitários.
   - Documentar os resultados no handoff.
