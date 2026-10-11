# Guia de Projeto — CS2 Optimizer (Rust + Slint)

# Projeto: CounterRush



## 1. Visão Geral do Projeto

**Objetivo:** Criar um otimizador/booster para Counter-Strike 2 focado exclusivamente em ajustes do sistema operacional Windows, sem qualquer manipulação de processos, memória ou arquivos do jogo.

**Escopo permitido:**

- Ajustes em configurações do Windows (registro, serviços, agendador, plano de energia, prioridades, etc.).

- Tweaks que **eu pessoalmente testei e validei** no CS2 antes de serem incorporados.

**Fora do escopo (proibido):**

- Qualquer interação com o processo do CS2 (leitura/escrita de memória, injeção, hooks, DLLs).

- Overlays, drivers, ou alterações que possam gerar falsos positivos no VAC.

- Modificação de arquivos do jogo ou da Steam.

> **Princípio norteador:** toda otimização deve ser reversível, auditável e restrita ao ambiente Windows.

---

## 2. Linguagem e Arquitetura

- **Linguagem principal:** Rust (edition estável mais recente).

- **Estrutura:** organizada em **crates e módulos separados**, cada um com responsabilidade única, para facilitar manutenção e testes isolados.

- **Sugestão de organização inicial:**

  - `core/` — lógica central e orquestração de tweaks.

  - `tweaks/` — um módulo por tweak (ex.: `power_plan.rs`, `game_mode.rs`, `network.rs`).

  - `windows_api/` — wrappers seguros para chamadas à API do Windows.

  - `config/` — leitura/gravação de configurações do usuário.

  - `ui/` — interface Slint (adicionada apenas na etapa correspondente).

  - `tests/` — testes de integração manuais.

---

## 3. Regras de Desenvolvimento (obrigatórias)

1. **Validação manual obrigatória:** cada módulo/crate criado deve ser testado por mim manualmente e de forma individual antes de ser integrado.

2. **Aviso prévio de risco:** qualquer mudança que possa quebrar o projeto ou desabilitar funcionalidades existentes deve ser **alertada antes** de ser executada, com descrição do impacto e alternativas.

3. **Escopo controlado:** só serão adicionadas funções que eu solicitar explicitamente. Nada de "extras" ou melhorias não pedidas.

4. **Comandos exatos:** quando uma função executar um comando (shell, PowerShell, registro, etc.), deve ser utilizado **exatamente o comando que eu enviei na sessão** — sem reformatar, otimizar ou substituir por equivalente.

5. **Sem suposições:** em caso de dúvida sobre comportamento, escopo ou comando, perguntar antes de implementar.

---

## 4. Etapa de Interface Gráfica (Slint)

- Só será iniciada após a lógica de tweaks estar estável e testada.

- Desenvolvimento **incremental, componente por componente**:

  1. Botão

  2. Switch

  3. Checkbox

  4. Dropdown

  5. ... (e assim por diante)

- Cada componente deve ser:

  - Implementado isoladamente.

  - Testado manualmente por mim.

  - **Aprovado antes** de passar para o próximo.

> Nenhum componente novo deve ser adicionado sem que o anterior esteja validado.

---

## 5. Git — Regras de Uso

**Permitido sem autorização (somente leitura):**

- `git status`

- `git log`

- `git diff`

- `git branch` (listagem)

- `git remote -v`

**Requer execução manual minha (nunca automatizar):**

- `git add`

- `git commit`

- `git push`

- `git pull` / `git fetch` com merge

- Qualquer comando que altere o repositório local ou remoto.

> O agente pode **sugerir** comandos de escrita, mas **nunca executá-los**.

---

## 6. Fluxo de Trabalho com o Agente (Antigravity)

1. Eu descrevo a tarefa (módulo, tweak ou componente).

2. O agente propõe o plano e **alerta riscos** antes de codar.

3. O agente implementa **apenas** o que foi pedido.

4. Eu testo manualmente o módulo isolado.

5. Se aprovado, seguimos; se não, iteramos.

6. Ao final de cada etapa estável, **eu** executo os comandos de Git.

---

## 7. Critérios de Aceite por Etapa

- Compila sem warnings críticos.

- Função executa apenas o que foi especificado.

- Nenhuma interação com o processo do CS2.

- Testado manualmente por mim.

- Aprovado explicitamente para prosseguir.

---

## 8. Restrições de Segurança (VAC)

- Proibido: read/write em memória de processos, `OpenProcess` sobre o CS2, hooks, injeção, drivers kernel-mode.

- Permitido: comandos administrativos do Windows, edição de registro, gerenciamento de serviços, planos de energia, prioridades de processo via API oficial do Windows **quando explicitamente solicitado por mim**.

- Toda tweak deve ser **reversível** (com função de rollback).

---

## 9. Observações Finais

- Documentar cada tweak com: nome, o que faz, comando usado, como reverter, e resultado observado no CS2.

- Manter um changelog manual do projeto.

- Em caso de conflito entre este guia e uma instrução pontual minha na sessão, **a instrução da sessão prevalece**, mas o guia deve ser atualizado depois.


