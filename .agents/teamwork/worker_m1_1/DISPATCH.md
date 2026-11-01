## 2026-11-01T16:21:42Z
Você é o Worker M1 1 da equipe CounterRush.
Sua missão é implementar as alterações na interface Slint conforme aprovado no Implementation Plan para atender aos requisitos R1, R2, R3 e R4.

Arquivo de requisitos originais:
g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\ORIGINAL_REQUEST.md
Leia este arquivo na íntegra antes de iniciar.

Documentos de arquitetura e plano:
- g:\Outros computadores\Meu laptop\Desktop\CounterRush\PROJECT.md
- g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\orchestrator_1\implementation_plan.md
- g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\orchestrator_1\task.md

Seu diretório de trabalho é:
g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\worker_m1_1\
Escreva seus arquivos de progresso (progress.md) e entrega (handoff.md) neste diretório.

PROPRIEDADE DE ESCRITA EXCLUSIVA:
Você possui posse de escrita estrita sobre os seguintes arquivos:
- `ui/components/add_preset_button.slint`
- `ui/main.slint`
NÃO modifique nenhum outro arquivo de código (especialmente Rust/backend, que já atendem a todos os requisitos).

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

TAREFAS DETALHADAS:
1. Em `ui/components/add_preset_button.slint`:
   - Quando `root.is-empty` for `true`, o texto deve ser `"Criar Otimização"` (em vez de `"NOVO PERFIL"`).
   - O símbolo `+` deve ser mantido.
   - Ajustar a largura quando vazio para `210px` (ex: `width: root.is-empty ? 210px : 44px;`) para comportar o texto sem cortes.
2. Em `ui/main.slint`:
   - [R2] Remover completamente o botão inferior `TacticalButton` "OTIMIZAR AUTOMATICAMENTE" (linhas ~896-917), preservando intacta a Barra de Status HUD Tática logo abaixo (linhas 920-941).
   - [R3] Mover o contêiner do input de parâmetros de inicialização (`if root.active-preset != "" : HorizontalLayout { ... }`, linhas ~770-888) para ficar posicionado diretamente abaixo das 13 otimizações ativas (após a 4ª linha da grade de switches, logo acima da linha 709) e antes do cabeçalho `"PERFIS DE OTIMIZAÇÃO"`.
   - [R1 & R4] Na seção de perfis:
     - Se `root.presets.length == 0`: renderizar exclusivamente os dois botões de ação centralizados:
       1. Botão "Otimizar Automaticamente" (utilizando estilo harmonioso com o tema CS2, acionando `root.auto-optimize-clicked()`).
       2. Botão "Criar Otimização +" (`AddPresetButton { is-empty: true; }` acionando a abertura do modal `root.show-create-modal = true;`).
     - Se `root.presets.length > 0`: renderizar o separador tático `"PERFIS DE OTIMIZAÇÃO"`, a lista horizontal de `PresetButton`s e o botão `AddPresetButton { is-empty: false; }`.
3. Verificação:
   - Execute a verificação de compilação: `cargo check -p counterrush-ui` (use `BypassSandbox: true` caso necessário no PowerShell).
   - Registre os resultados e diffs no seu `handoff.md`.

Ao concluir, envie uma mensagem para o orchestrator (ID: a2e09f4d-39a0-4d52-880b-d65100808272) informando a conclusão.
