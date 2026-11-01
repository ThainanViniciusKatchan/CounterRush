# Plano de Implementação — Comportamento do Botão "Jogar CS2" na Ausência de Perfis

## 1. Visão Geral e Motivação
Quando o usuário clica em "JOGAR CS2" na aba Painel sem que existam perfis de otimização configurados, o comportamento anterior tentava carregar e criar automaticamente um arquivo `custom_optimization.json` no disco e iniciar o jogo.
A nova especificação do usuário exige que:
1. Se não existirem perfis/configurações de otimização, o jogo NÃO deve iniciar;
2. Um modal deve ser exibido com a mensagem exata: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`;
3. Dois botões devem ser exibidos: `"Criar Perfil"` e `"Sair"`;
4. Clicar em `"Criar Perfil"` deve fechar o modal e levar o usuário diretamente para a aba "Otimizações";
5. Clicar em `"Sair"` deve fechar o modal, permanecendo na aba atual.

## 2. Análise de Risco e Impacto
- **Severidade**: Baixa / Não destrutiva.
- **Impacto no Sistema**: Nenhuma alteração destrutiva em arquivos de sistema ou configurações.
- **Atenção**: Garantir que o botão "JOGAR CS2" não mude seu texto para "INICIANDO..." quando o modal for aberto, interceptando antes de disparar o fluxo de lançamento.

## 3. Arquivos Envolvidos e Motivação Detalhada

### 3.1 `ui/components/no_profile_dialog.slint` (Novo Arquivo)
- **Motivação**: Criar um componente modal independente e reutilizável, alinhado à Clean Architecture e ao padrão visual já estabelecido por `delete_profile_dialog.slint` e `create_profile_dialog.slint`.
- **Conteúdo**:
  - Backdrop em `Rectangle` 980x620px com opacidade escura (`rgba(11, 15, 23, 0.88)`) e `TouchArea` bloqueador para isolar cliques de fundo.
  - Card centralizado estilizado com `CS2Theme.bg-card` e borda `CS2Theme.accent-orange`.
  - Cabeçalho com indicador tático laranja e botão fechar `✕`.
  - Mensagem exigida: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`.
  - Botão secundário `"Sair"` disparando callback `close()`.
  - Botão primário `"Criar Perfil"` disparando callback `create-profile()`.

### 3.2 `ui/main.slint` (Modificação)
- **Motivação**: Integrar o novo componente modal na janela principal e interceptar o clique no botão "JOGAR CS2".
- **Alterações**:
  - Importação do componente `NoProfileDialog`.
  - Declaração da propriedade `in-out property <bool> show-no-profile-modal: false;`.
  - No `TacticalButton` "JOGAR CS2" (`ui/main.slint:388`):
    - Se `root.presets.length == 0`: definir `root.show-no-profile-modal = true;` sem alterar `root.is-launching` e sem chamar `root.launch-cs2-clicked()`.
    - Se `root.presets.length > 0`: manter o fluxo normal (`root.is-launching = true; root.launch-cs2-clicked();`).
  - No final do `MainWindow`:
    - Adicionar a renderização condicional do `NoProfileDialog`:
      - `close => { root.show-no-profile-modal = false; }`
      - `create-profile => { root.show-no-profile-modal = false; root.active-tab = 1; root.status-message = "Ajustes Avançados de Hardware"; }`

### 3.3 `ui/src/main.rs` (Modificação Defensiva)
- **Motivação**: Princípio da defesa em profundidade (Clean Architecture). Caso o backend receba uma invocação de `on_launch_cs2_clicked` sem perfis disponíveis, ele deve abortar a execução com segurança em vez de criar arquivos fictícios.
- **Alterações**:
  - No início de `main_window.on_launch_cs2_clicked`, verificar se `scan_presets().is_empty()`. Se vazio, garantir `ui.set_is_launching(false)` e `ui.set_show_no_profile_modal(true)` e retornar precocemente (`return;`).

## 4. Etapas de Implementação
1. **Etapa 1 — Criação do Componente Slint**: Criar `ui/components/no_profile_dialog.slint`.
2. **Etapa 2 — Integração e Interceptação no Slint**: Atualizar `ui/main.slint`.
3. **Etapa 3 — Salvaguarda no Backend Rust**: Atualizar `ui/src/main.rs`.
4. **Etapa 4 — Validação Técnica e Compilação**: Executar `cargo check --workspace` e `cargo test --workspace`.
5. **Etapa 5 — Revisão, Testes Adversariais e Auditoria Forense**: Dispatch de Reviewers, Challengers e Auditor Forense.
