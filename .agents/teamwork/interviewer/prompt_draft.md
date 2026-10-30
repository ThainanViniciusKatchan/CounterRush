Implementação para ambiente de produção do estado inicial sem otimizações ativas, inclusão do botão de criação "+" na área de perfis e janela para criação de novos arquivos de perfil de otimização (.json) com seleção de parâmetros e validação de execução no CounterRush.

Integrity mode: development

## Requirements

### R1. Estado inicial desativado na primeira execução
Quando o programa for iniciado pela primeira vez e nenhum arquivo de configuração de otimização existir no diretório de otimizações, todas as otimizações devem iniciar desligadas (desativadas) e nenhum arquivo de perfil deve ser considerado ativo por padrão.

### R2. Botão "+" na área de perfis de otimização
Adicionar um botão "+" na área onde os arquivos de otimização são exibidos na aba de Otimizações. Caso nenhum perfil exista, o botão "+" deve estar visível para permitir a criação do primeiro arquivo. Caso existam perfis ou um novo perfil seja criado, o botão "+" deve se posicionar imediatamente após o último perfil da listagem.

### R3. Janela e persistência de novo arquivo de otimização (.json)
Adicionar uma nova janela ou interface de diálogo para criação de novos perfis de otimização em formato `.json`, contendo:
- Campo para inserção do nome do arquivo.
- Campo para inserção de parâmetros de inicialização (`launch_args`).
- Seleção das otimizações que devem estar ativadas no perfil.
Ao confirmar, o arquivo deve ser salvo no diretório de otimizações (`config/optimzations`) com o nome informado. Imediatamente após ser salvo, o novo perfil deve ser selecionado e ativado como o perfil corrente na interface.

### R4. Testes automatizados e validação em tempo de execução
Além da compilação bem-sucedida via `cargo check`, a equipe deve implementar testes automatizados (unitários ou de integração) em Rust cobrindo o fluxo de criação, validação do `.json` e o estado inicial sem arquivos. A aplicação recém-modificada deve ser executada para validar funcionalmente se ocorrem erros de execução ou falhas em runtime.

### R5. Governança de planejamento e alterações mínimas
A equipe deve obrigatoriamente submeter o Plano de Implementação (`implementation_plan.md`) e a Lista de Tarefas (`task.md`) em Português-BR para aprovação prévia do usuário antes de realizar qualquer alteração no código-fonte, modificando estritamente os arquivos necessários para atender à solicitação.

## Acceptance Criteria

### Estado Inicial Sem Perfis
- [ ] Ao inicializar a aplicação sem arquivos na pasta `config/optimzations`, todos os switches de otimização iniciam em estado desligado (`false`).
- [ ] Nenhum perfil fictício é ativado indevidamente quando não há arquivos de otimização salvos.
- [ ] O botão "+" é exibido mesmo quando não há nenhum perfil existente.

### Posicionamento e Ação do Botão "+"
- [ ] O botão "+" é posicionado imediatamente à direita (após) o último botão de perfil de otimização existente.
- [ ] Clicar no botão "+" abre a interface de criação de novo perfil.

### Interface de Criação e Salvamento
- [ ] A interface permite ao usuário definir o nome do arquivo, argumentos de inicialização e marcar quais otimizações devem ser ativadas.
- [ ] O arquivo criado é salvo no diretório `config/optimzations/<nome>.json` com estrutura compatível com `CustomOptimizationConfig`.
- [ ] O novo perfil é automaticamente selecionado como ativo na tela principal após ser salvo, e a lista de botões é atualizada mantendo o botão "+" após o novo item.

### Testes e Verificação
- [ ] O workspace compila sem erros com `cargo check`.
- [ ] Testes automatizados em Rust são criados e executados com sucesso (`cargo test`).
- [ ] A aplicação é executada sem panics, travamentos ou falhas de tempo de execução.
- [ ] O plano de implementação e a lista de tarefas em Português-BR são apresentados e aprovados antes de alterar o código.
