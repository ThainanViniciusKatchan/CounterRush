# Handoff - Ajuste da Função de Auto-detecção (AutoDetect.json)

## 1. O que foi alterado

### `core/src/lib.rs`
- **Função `generate_autodetect_config`**:
  - Mantida a varredura e diagnóstico do hardware através de `get_or_detect_hardware(None)` e a obtenção de parâmetros recomendados de inicialização via `get_default_launch_args_for_category(category)`.
  - Integrada a invocação de todos os verificadores reais existentes no módulo `tweaks` (`Cs2PriorityTweak`, `Cs2AffinityTweak`, `EmptyStandbyListTweak`, `TimerResolutionTweak`, `is_all_gpu_accel_disabled`, `Cs2DpiOptimizationTweak`, `MemoryCompressionTweak`, `is_mmcss_applied_for_current_hardware`, `HpetTweak`, `GameBarTweak`, `is_all_msi_enabled`, `PcieUsbPowerSavingTweak` e `RenderQueueTweak`).
  - Definidos compulsoriamente como `true` os campos de otimização em tempo de execução (runtime): `cs2_affinity`, `apply_timer_resolution` e `empty_standby_list`.
  - Mapeados os demais campos de otimizações para valores estritamente booleanos correspondentes ao status real retornado pelos respectivos verificadores.
  - Atualizada a lista de avaliações (`evaluations`) no relatório de restrições (`HardwareRestrictionsReport`) refletindo o status real verificado e os critérios de runtime.
- **Teste Automatizado `test_autodetect_config_generation_and_no_custom_optimization`**:
  - Adicionado teste de unidade cobrindo programaticamente a execução de `save_autodetect_config`, a criação exclusiva de `AutoDetect.json`, os valores booleanos esperados (com campos de runtime em `true` e demais campos correspondentes aos verificadores), e garantindo programaticamente a ausência do arquivo `custom_optimization.json`.

### `ui/src/main.rs`
- Corrigido o carregamento de inicialização dos presets na UI: ao inicializar com perfis existentes em disco (como `AutoDetect.json`), o sistema carrega o perfil já salvo em vez de disparar a criação forçada e inadvertida de `custom_optimization.json`.

### `core/examples/test_autodetect.rs`
- Atualizada a rotina de exemplo para invocar diretamente `core.save_autodetect_config(None)`, utilizando o fluxo completo de auto-detecção com verificadores e gravação do `AutoDetect.json`.

## 2. Por quê
- Em conformidade com os requisitos R1, R2, R3 e R4 da solicitação original, garantindo que o arquivo `AutoDetect.json` seja gerado unicamente a partir de inspeção real do sistema e que o arquivo `custom_optimization.json` não seja gerado sob nenhuma hipótese durante a rotina de autodetecção.

## 3. Registro de Verificação
- **Testes executados**:
  - `cargo check --workspace`: compilação limpa sem erros nem avisos.
  - `cargo test -p core`: 6 testes unitários executados e aprovados com sucesso, incluindo `test_autodetect_config_generation_and_no_custom_optimization`.
  - `cargo test --workspace`: 12 testes aprovados em todo o workspace (`config` e `core`).
- **Verificação de Regressão**:
  - Todos os testes preexistentes (`test_profile_name_validation`, `test_list_optimization_configs_empty_directory`, `test_create_and_persist_new_profile`, `test_autodetect_evaluation_respects_hardware_limits`, `test_isolated_build_dir_does_not_escape_to_project_dir`) continuam passando.
