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
