Atualizar a interface e o comportamento da aba de Otimizações do CounterRush, tratando o estado sem configurações com botões dedicados e reorganizando os elementos da tela.

Integrity mode: development

## Requirements

### R1. Estado da aba de Otimizações sem arquivos .json
Quando não existir nenhum arquivo de configuração `.json` no projeto, a aba de Otimizações deve exibir exclusivamente dois botões de ação:
- Um botão com o texto "Otimizar Automaticamente", que ao ser clicado cria uma nova configuração automática baseada no hardware do usuário.
- O botão de adicionar pré-existente, agora com o símbolo `+` e o texto "Criar Otimização", que ao ser pressionado abre o modal de criação.

### R2. Remoção do botão inferior de otimização automática
O botão "Otimizar Automaticamente" atualmente localizado na parte inferior da tela deve ser removido.

### R3. Reposicionamento do input de parâmetros de inicialização
O campo de entrada (input) para alterar os parâmetros de inicialização deve ser reposicionado para ficar localizado diretamente abaixo das otimizações ativas e antes da seleção de arquivos de otimização.

### R4. Estado com arquivos de configuração existentes
Quando existirem arquivos de configuração `.json`, a equipe deve manter o fluxo de gerenciamento de configurações existente preservando a consistência visual com os novos controles.

## Acceptance Criteria

### Interface e Funcionalidade
- [ ] No estado sem arquivos de configuração `.json`, são exibidos apenas os dois botões: "Otimizar Automaticamente" e o botão de adicionar com o símbolo `+` e o texto "Criar Otimização".
- [ ] O clique em "Otimizar Automaticamente" no estado vazio cria e aplica a configuração de acordo com o hardware do sistema.
- [ ] O clique em "Criar Otimização +" abre o modal de criação de otimizações.
- [ ] O botão inferior pré-existente "Otimizar Automaticamente" foi completamente removido da parte inferior da tela.
- [ ] O campo de input para parâmetros de inicialização está renderizado logo abaixo das otimizações ativas e antes da seleção de arquivos de otimização.
- [ ] Na presença de arquivos `.json`, a seleção e listagem de otimizações continuam operacionais.
- [ ] O layout e comportamento são validados visualmente através da execução/Hot Reload da aplicação.
