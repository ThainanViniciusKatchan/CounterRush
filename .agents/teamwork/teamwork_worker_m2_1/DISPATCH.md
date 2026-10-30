## 2026-10-30T20:25:37Z
Você é teamwork_worker_m2_1, atuando como Test Automation and Runtime Validation Worker.
Seu diretório de trabalho é: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_worker_m2_1
Seu orquestrador pai é: 57a334a3-ffa9-4780-b779-a82434f687fa

LEITURA OBRIGATÓRIA:
1. Requisitos originais: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\ORIGINAL_REQUEST.md
2. Plano de Implementação Aprovado: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\orchestrator\implementation_plan.md
3. Escopo do Projeto: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\orchestrator\PROJECT.md
4. Relatório do Explorer 3 (Testes & Runtime): g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_explorer_survey_3\handoff.md
5. Relatório do Worker m1: g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_worker_m1_1\handoff.md

AVISO OBRIGATÓRIO DE INTEGRIDADE (MANDATORY INTEGRITY WARNING):
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A teamwork_auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

ARQUIVOS DE SUA PROPRIEDADE EXCLUSIVA (WRITE OWNERSHIP):
- Cargo.toml (raiz)
- core/Cargo.toml
- config/Cargo.toml
- config/src/lib.rs
- core/src/lib.rs

TAREFA - MILESTONE 2: Testes Automatizados e Validação em Tempo de Execução (R4)
1. Manifestos do Cargo:
   - Em `Cargo.toml` (raiz): adicionar `tempfile = "3"` na seção `[workspace.dependencies]`.
   - Em `core/Cargo.toml`: adicionar `[lib] doctest = false` (para eliminar a colisão interna de doctests com `::core`) e adicionar `tempfile = { workspace = true }` em `[dev-dependencies]`.
   - Em `config/Cargo.toml`: adicionar `tempfile = { workspace = true }` em `[dev-dependencies]`.
2. Refinamento em `core/src/lib.rs`:
   - Em `validate_profile_name(name: &str)`: aprimorar para remover `.json` de forma case-insensitive (ex: tratando `.json`, `.JSON`, `.Json`, etc.).
3. Suíte de Testes Automatizados no crate `config` (`config/src/lib.rs`):
   - Adicionar módulo `#[cfg(test)] mod tests` cobrindo:
     * `test_custom_optimization_config_all_disabled`: verifica que todas as 13 flags são `false` e `launch_args` é vazio.
     * `test_json_roundtrip_integrity`: serializa uma struct customizada com `serde_json::to_string_pretty` e desserializa confirmando igualdade total via `PartialEq`.
     * `test_save_and_load_file`: cria diretório temporário com `tempfile::tempdir()`, salva com `save_to_file` e recarrega com `load_from_file`, verificando persistência e criação automática de pastas.
     * `test_load_with_bom`: testa desserialização de JSON contendo BOM UTF-8 (`\u{feff}`).
     * `test_malformed_json_handling`: testa rejeição limpa (`ConfigError::Json`) de JSON corrompido sem pânico.
     * `test_serde_default_omitted_fields`: testa que JSON sem alguns campos preenche os defaults via `#[serde(default)]`.
4. Suíte de Testes Automatizados no crate `core` (`core/src/lib.rs`):
   - Adicionar módulo `#[cfg(test)] mod tests` cobrindo:
     * `test_profile_name_validation`: testa nomes válidos, nomes vazios, nomes apenas com espaços, caracteres ilegais NTFS (`\ / : * ? " < > |`), nomes de dispositivos reservados (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`) e sufixo `.json` case-insensitive.
     * `test_list_optimization_configs_empty_directory`: testa com diretório temporário vazio e confirma que `list_optimization_configs` retorna lista vazia sem erros e sem criar arquivos.
     * `test_create_and_persist_new_profile`: simula o fluxo do R3 gravando uma `CustomOptimizationConfig` em diretório temporário e verificando a integridade.
5. Execução de Verificação Obrigatória:
   - Execute `cargo check --workspace` no PowerShell (com `BypassSandbox: true`).
   - Execute `cargo test --workspace` no PowerShell (com `BypassSandbox: true`). Certifique-se de que 100% dos testes passam com exit code 0.
   - Execute a validação em runtime do executável `counterrush-ui.exe` com inicialização e encerramento limpo via script PowerShell:
     ```powershell
     $proc = Start-Process -FilePath "C:\cargo-target\CounterRush\debug\counterrush-ui.exe" -PassThru
     Start-Sleep -Seconds 2
     $running = -not $proc.HasExited
     Stop-Process -Id $proc.Id -Force
     Write-Host "Processo executou com estabilidade: $running"
     ```

ENTREGA:
Escreva seu relatório estruturado completo de handoff em:
g:\Outros computadores\Meu laptop\Desktop\CounterRush\.agents\teamwork\teamwork_worker_m2_1\handoff.md
Ao concluir, envie uma mensagem para o orquestrador (57a334a3-ffa9-4780-b779-a82434f687fa) com o resumo dos testes implementados, comandos executados e resultados de compilação/execução.
