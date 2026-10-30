## 2026-10-30T20:03:48Z

Você é teamwork_worker_m1_1, atuando como Full Stack Implementation Worker.
Seu diretório de trabalho é: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_worker_m1_1
Seu orquestrador pai é: 57a334a3-ffa9-4780-b779-a82434f687fa

LEITURA OBRIGATÓRIA:
1. Requisitos originais: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\ORIGINAL_REQUEST.md
2. Plano de Implementação Aprovado: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\orchestrator\implementation_plan.md
3. Escopo do Projeto: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\orchestrator\PROJECT.md
4. Relatório Backend Explorer: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_explorer_survey_1\handoff.md
5. Relatório UI Explorer: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_explorer_survey_2\handoff.md

AVISO OBRIGATÓRIO DE INTEGRIDADE (MANDATORY INTEGRITY WARNING):
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

ARQUIVOS DE SUA PROPRIEDADE EXCLUSIVA (WRITE OWNERSHIP):
Você possui autorização estrita para modificar ou criar APENAS os seguintes arquivos:
- config/src/lib.rs
- core/src/lib.rs
- ui/components/add_preset_button.slint (novo arquivo)
- ui/components/create_profile_dialog.slint (novo arquivo)
- ui/main.slint
- ui/src/main.rs

TAREFA - MILESTONE 1: Implementação Funcional Completa (R1, R2, R3)
1. R1 (Estado Inicial Desativado):
   - Em `config/src/lib.rs`: Adicionar derivação `PartialEq, Eq` na struct `CustomOptimizationConfig`. Adicionar método `pub fn all_disabled() -> Self` retornando todas as 13 flags como `false` e `launch_args` vazio.
   - Em `ui/src/main.rs`: Em `scan_presets()`, remover a injeção artificial de `AutoDetect` e `custom_optimization` quando a lista de arquivos estiver vazia (retornar lista vazia).
   - Em `ui/src/main.rs`: Na inicialização em `main()`, se `initial_presets.is_empty()`, NÃO chamar `load_or_create_custom_config(None)` (para não criar arquivo fictício no disco prematuramente), definir `active-preset` como `""` e configurar todos os 13 switches da interface para `false`.
   - Em `ui/main.slint`: Configurar valores padrão neutros das propriedades da `MainWindow` (`active-preset: ""`, `presets: []`, e os 13 switches iniciando em `false`).
2. R2 (Botão "+" na listagem de perfis):
   - Criar `ui/components/add_preset_button.slint` com altura de 44px (igual ao `PresetButton`), estilo CS2Theme, hover visual e callback de clique.
   - Em `ui/main.slint`: Posicionar o `AddPresetButton` imediatamente após o loop `for preset in root.presets` dentro do `HorizontalLayout` de perfis. Dessa forma, se a lista estiver vazia, ele será exibido centralizado e visível; se houver perfis, ele ficará posicionado imediatamente à direita do último perfil.
   - Conectar o clique do botão "+" para abrir o modal de criação (`root.show-create-modal = true;`).
3. R3 (Modal de Criação e Persistência de Perfil .json):
   - Criar `ui/components/create_profile_dialog.slint`: Modal overlay in-window cobrindo a tela (980x620px), fundo translúcido tático com `TouchArea` bloqueando cliques de fundo, card estilizado contendo:
     * Campo para nome do arquivo (.json);
     * Campo para argumentos de inicialização Steam (`launch_args`);
     * Grade com seleção das 13 otimizações (com botões de conveniência Marcar/Desmarcar Todos);
     * Mensagem de erro de validação;
     * Botões Cancelar e Criar/Ativar Perfil.
   - Em `core/src/lib.rs`: Adicionar função pública `pub fn validate_profile_name(name: &str) -> Result<String, String>` que rejeita nomes vazios, caracteres proibidos do Windows (`\ / : * ? " < > |`) e remove extensão `.json` duplicada se presente.
   - Em `ui/main.slint`: Integrar o `CreateProfileDialog` na raiz da `MainWindow` controlado pela propriedade `show-create-modal`, com callback `create-profile-confirmed(NewProfileData)`.
   - Em `ui/src/main.rs`: Implementar o handler do callback `create-profile-confirmed`:
     * Validar e sanitizar o nome via `validate_profile_name`. Se inválido, setar `create-modal-error` na UI e não fechar o modal.
     * Tokenizar `launch_args` (`split_whitespace()`).
     * Instanciar `CustomOptimizationConfig` com as opções escolhidas.
     * Salvar em `config/optimzations/<nome>.json` usando `new_config.save_to_file(&target_path)`.
     * Atualizar a lista de perfis via `scan_presets()`.
     * Atualizar `active_config_path` e setar `active-preset` na UI com o nome do perfil recém-criado.
     * Sincronizar os 13 switches da interface principal conforme a configuração criada.
     * Fechar o modal (`show-create-modal = false`).
     * Exibir feedback positivo na barra de status HUD.
   - Em `ui/src/main.rs`: No callback `on_tweak_toggled`, se nenhum perfil estiver ativo, emitir feedback informativo ("Crie ou selecione um perfil para salvar alterações").

VERIFICAÇÃO OBRIGATÓRIA:
Você DEVE executar `cargo check --workspace` no terminal do Windows PowerShell (usando `BypassSandbox: true` se necessário para acessar a unidade G:) e certificar-se de que a compilação passa com 0 erros.

ENTREGA:
Escreva seu relatório estruturado completo de handoff em:
g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_worker_m1_1\handoff.md
Ao concluir, envie uma mensagem para o orquestrador (57a334a3-ffa9-4780-b779-a82434f687fa) informando o resultado da compilação e resumo das alterações.
