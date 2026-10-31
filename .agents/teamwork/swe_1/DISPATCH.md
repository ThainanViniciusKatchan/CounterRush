## 2026-10-31T17:14:11Z
This is a single self-contained fix; keep it small and focused. Implementação para ambiente de produção da exibição e edição dos parâmetros de inicialização do arquivo de perfil de otimização selecionado na aba de otimizações do CounterRush, com controle de bloqueio do campo de entrada e alternância entre os ícones de lápis e salvar.

Integrity mode: development

## Requirements

### R1. Exibição e controle de visibilidade dos parâmetros de inicialização
Adicionar um campo de input na aba de otimizações que exiba os parâmetros de inicialização do arquivo de perfil de otimização selecionado.
- Por padrão, o campo deve iniciar bloqueado para edição.
- Quando nenhum arquivo de perfil de otimização estiver selecionado, o campo de input e o botão de edição associado devem ficar ocultos. Ao selecionar um perfil, ambos devem ser exibidos com os parâmetros correspondentes ao arquivo.

### R2. Botão de edição e salvamento com alternância de ícones
Adicionar à frente do campo de input um botão com estilo idêntico ao botão de adicionar configuração:
- Em estado bloqueado, o botão deve exibir o ícone de lápis. Ao ser clicado, deve desbloquear o campo de input para edição.
- Com o campo desbloqueado, o ícone do botão deve alternar para um ícone de salvamento. Ao ser clicado, deve persistir as alterações dos parâmetros no arquivo de otimização selecionado e bloquear novamente o campo de input.

### R3. Consistência visual e de layout
Garantir que o novo campo de input e o botão mantenham alinhamento e consistência visual com a interface da aba de otimizações, respeitando o distanciamento correto e as dimensões dos botões já estabelecidos no projeto.

### R4. Governança e regras do projeto
Cumprir estritamente as diretrizes de `.agent/rules/relus.md`:
- Elaborar e submeter o plano de implementação e a lista de tarefas em Português-BR para aprovação prévia do usuário antes de realizar alterações no código-fonte.
- Alterar estritamente os arquivos necessários para atender à solicitação, sem duplicar classes, funções ou arquivos existentes.
- Revisar tudo antes de entregar e após cada edição.

## Acceptance Criteria

### Visibilidade e Seleção de Perfil
- [ ] Quando nenhum perfil de otimização está selecionado na aba, o campo de parâmetros e o botão de edição/salvamento permanecem ocultos.
- [ ] Ao selecionar um perfil de otimização, o campo de parâmetros e o botão tornam-se visíveis.
- [ ] O campo exibe os parâmetros de inicialização configurados no arquivo de perfil selecionado.

### Comportamento de Edição e Salvamento
- [ ] O campo de input inicia bloqueado para edição por padrão.
- [ ] O botão à frente exibe o ícone de lápis enquanto o campo estiver bloqueado.
- [ ] Clicar no botão com ícone de lápis desbloqueia o campo de input para digitação e altera o ícone do botão para salvamento.
- [ ] Clicar no botão com ícone de salvamento grava os parâmetros no arquivo do perfil selecionado e bloqueia novamente o campo de input, retornando o ícone para lápis.
- [ ] O distanciamento, dimensões e alinhamento do campo e botão são consistentes com os padrões visuais existentes da aba.

### Governança e Validação
- [ ] O plano de implementação e lista de tarefas em Português-BR são apresentados e aprovados antes de alterar o código.
- [ ] O projeto compila com sucesso via `cargo check`.
- [ ] A aplicação executa sem travamentos ou erros funcionais ao selecionar perfis, alternar a edição e salvar novos parâmetros.


## 2026-10-31T19:56:53Z
This is a single self-contained fix; keep it small and focused. Ajustar a função de auto-detecção responsável pela criação do arquivo AutoDetect.json no CounterRush para que utilize os verificadores das otimizações existentes e gere unicamente esse arquivo com os valores booleanos correspondentes.

Integrity mode: development

## Requirements

### R1. Varredura de hardware e checagem de otimizações
A função de auto-detecção deve continuar realizando a varredura no hardware para identificar os componentes do sistema e deve invocar todos os verificadores existentes nas otimizações para identificar quais otimizações já estão aplicadas e quais ainda não estão.

### R2. Geração exclusiva do arquivo AutoDetect.json
A função deve gerar unicamente o arquivo AutoDetect.json, preenchendo as entradas de otimizações com valores booleanos (true e false) conforme o retorno da respectiva verificação. O arquivo custom_optimization.json não deve ser criado em nenhuma circunstância.

### R3. Valores fixos para otimizações de runtime
As seguintes otimizações, que atuam somente quando o jogo está rodando, devem ser registradas obrigatoriamente com o valor booleano true no AutoDetect.json:
- Afinidade
- Timer 0.5ms
- Limpeza da Standby List

### R4. Padrões do projeto
O código modificado deve seguir os padrões do repositório (Clean Architecture, princípios SOLID) e respeitar as diretrizes de `.agent/rules/relus.md`, incluindo a apresentação prévia de plano de implementação.

## Acceptance Criteria

### Geração e Validação do AutoDetect.json
- [ ] A função de auto-detecção executa a varredura de hardware e consulta os verificadores de cada otimização disponível.
- [ ] O arquivo AutoDetect.json é gerado com valores booleanos correspondentes ao estado real retornado pelos verificadores.
- [ ] As entradas de Afinidade, Timer 0.5ms e Limpeza da Standby List possuem valor true no AutoDetect.json gerado.
- [ ] A execução da função resulta exclusivamente na criação de AutoDetect.json, sem criar o arquivo custom_optimization.json.

### Verificação Automatizada
- [ ] Um teste automatizado executa a rotina de auto-detecção e verifica programaticamente que AutoDetect.json é gerado com os valores booleanos esperados e campos fixos em true.
- [ ] O teste automatizado confirma programaticamente que o arquivo custom_optimization.json não foi gerado.
