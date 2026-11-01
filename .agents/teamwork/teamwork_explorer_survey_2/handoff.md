# Relatório de Handoff — Explorer 2

**Tipo de Handoff:** Hard (Investigação Concluída)  
**Data:** 2026-11-01  
**Autor:** Explorer 2 (Teamwork CounterRush)  
**Destinatário:** Orchestrator (ID: `a2e09f4d-39a0-4d52-880b-d65100808272`)

---

## 1. Observation (Observações Diretas)

1. **Localização e Listagem das Configurações `.json`:**
   - Em `core/src/lib.rs:1013-1035`, `resolve_optimizations_dir()` define o diretório de arquivos de otimização em `config/optimzations`.
   - Em `core/src/lib.rs:1057-1059`, `list_optimization_configs()` lista os arquivos `.json` existentes em `config/optimzations`.
   - Em `ui/src/main.rs:5-22`, a função `scan_presets()` transforma os arquivos em instâncias de `PresetModel { name, tag }`.
   - Em `config/src/lib.rs:87-109`, `CustomOptimizationConfig::load_from_file()` e `save_to_file()` serializam e desserializam as configurações em JSON com suporte a remoção de BOM UTF-8 e criação de diretórios pai.

2. **Lógica de "Otimizar Automaticamente" Baseada em Hardware:**
   - Em `core/src/lib.rs:745-753`, `OptimizerCore::save_autodetect_config(None)`:
     ```rust
     pub fn save_autodetect_config(
         &self,
         custom_path: Option<&Path>,
     ) -> Result<(PathBuf, config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
         let (config, report) = self.generate_autodetect_config()?;
         let path = resolve_autodetect_config_path(custom_path);
         config.save_to_file(&path)?;
         Ok((path, config, report))
     }
     ```
   - Em `core/src/lib.rs:354-547`, `generate_autodetect_config()` invoca `get_or_detect_hardware()`, obtém a categoria via `classifier::classify_pc()`, obtém argumentos de launch recomendados via `get_default_launch_args_for_category()` e avalia os 13 tweaks considerando limites de CPU (mais de 4 núcleos para afinidade), RAM (mais de 16 GB para compressão de memória), GPU (AMD/NVIDIA para fila de renderização) e economia de energia.
   - Em `core/examples/test_autodetect.rs:26-32`, confirma-se que a gravação do arquivo `AutoDetect.json` segue estritamente este fluxo e persiste no caminho `config/optimzations/AutoDetect.json`.

3. **Callback e Integração Existente no Backend Rust:**
   - Em `ui/src/main.rs:466-517`, o callback `main_window.on_auto_optimize_clicked(move || { ... })` já está totalmente codificado e realiza:
     - Chamada a `core.save_autodetect_config(None)`.
     - Atualização do ponteiro do arquivo ativo `auto_opt_ref`.
     - Leitura atualizada dos perfis via `scan_presets()`.
     - Atualização do modelo Slint `ui.set_presets(...)`.
     - Seleção do perfil ativo `ui.set_active_preset("AutoDetect".into())`.
     - Definição dos argumentos `ui.set_launch_args_text(...)`.
     - Definição do estado de todos os 13 toggles de tweaks via `ui.set_*_tweak_active(...)`.
     - Exibição de mensagem na barra HUD via `ui.set_status_message(...)`.

4. **Estado Atual da Interface em `ui/main.slint`:**
   - Em `ui/main.slint:71`, a MainWindow declara `callback auto-optimize-clicked();`.
   - Em `ui/main.slint:891-917`, o botão "OTIMIZAR AUTOMATICAMENTE" está localizado na parte inferior, imediatamente acima da Barra de Status HUD:
     ```slint
     TacticalButton {
         text: "OTIMIZAR AUTOMATICAMENTE";
         badge: "PERFIL RECOMENDADO • BAIXA LATÊNCIA";
         clicked => {
             root.auto-optimize-clicked();
         }
     }
     ```
   - Em `ui/main.slint:770-888`, o bloco de input de parâmetros de inicialização (`if root.active-preset != "" : HorizontalLayout`) está renderizado **abaixo** da linha de botões de seleção de perfil (`ui/main.slint:741-767`).
   - Em `ui/components/add_preset_button.slint:32-40`, o botão de adicionar exibe `text: "NOVO PERFIL";` quando `root.is-empty` é true.

---

## 2. Logic Chain (Cadeia Lógica de Raciocínio)

1. A partir da Observação 1, os perfis residem exclusivamente em `config/optimzations/*.json`. O estado sem configurações `.json` ocorre se e somente se `counterrush_core::list_optimization_configs()` retornar uma lista vazia, o que reflete na UI Slint como `root.presets.length == 0` e `root.active-preset == ""`.
2. A partir da Observação 2 e 3, a funcionalidade de otimização automática por hardware (`AutoDetect.json`) já está completamente implementada no backend e associada ao callback `main_window.on_auto_optimize_clicked`. Quando invocado, ele cria o arquivo em disco, recarrega a lista de presets e atualiza todos os campos e switches da UI.
3. Para atender ao requisito **R1**, o novo botão "Otimizar Automaticamente" exibido no estado vazio (`root.presets.length == 0`) só precisa invocar `root.auto-optimize-clicked()`. A execução desse callback fará com que `root.presets.length` passe imediatamente de 0 para 1, acionando a reatividade do Slint que automaticamente transiciona a tela do estado vazio para o estado com perfil carregado.
4. O segundo botão exigido por **R1** ("Criar Otimização +" abrindo o modal de criação) já existe no componente `AddPresetButton` (`ui/components/add_preset_button.slint`). Ele já abre o modal de criação via callback `clicked`. Basta alterar o texto exibido no modo vazio de `"NOVO PERFIL"` para `"Criar Otimização"` e ajustar sua largura para comportar o texto com folga.
5. Para atender ao requisito **R2**, a remoção do botão inferior em `ui/main.slint:891-917` consiste unicamente em retirar o bloco `HorizontalLayout { TacticalButton { ... } }`, preservando a Barra de Status HUD tática no rodapé.
6. Para atender ao requisito **R3**, o bloco do input de parâmetros de inicialização (`if root.active-preset != "" : HorizontalLayout { ... }`) precisa ser movido para cima, ficando posicionado diretamente abaixo dos 13 switches de otimizações ativas e antes do título divisor "PERFIS DE OTIMIZAÇÃO (CONFIG/OPTIMZATIONS)". Como ele depende de `root.active-preset != ""`, quando `root.presets.length == 0` ele permanecerá oculto, garantindo que no estado vazio sejam exibidos exclusivamente os dois botões de ação.
7. Para atender ao requisito **R4**, quando existirem arquivos `.json`, `root.presets.length > 0` renderiza a lista de `PresetButton`s acompanhada do `AddPresetButton` compacto (`+`), preservando o fluxo intacto de seleção, edição e salvamento.

---

## 3. Caveats (Ressalvas e Limitações)

- **Apenas Leitura (Read-Only):** Nenhuma alteração foi realizada em arquivos de código da aplicação (`ui/`, `core/`, `config/`), conforme estrita regra de investigação do Explorer 2.
- **Ambiente de Build/Teste no Sandbox:** O comando do PowerShell no ambiente sandbox do Windows não possui acesso padrão de escrita ao drive `G:\` sem elevação ou `BypassSandbox: true`. Portanto, os testes automatizados via `cargo test` devem ser executados pelo implementador/arquiteto em ambiente apropriado.
- **Hot Reload com Slint Viewer:** A validação visual final pode ser executada rapidamente com o script já existente no projeto: `preview.cmd` (`slint-viewer ui/main.slint -I ui --auto-reload`).

---

## 4. Conclusion (Conclusão e Diagnóstico)

A implementação dos requisitos é direta, limpa e de baixo risco estrutural:
1. **No backend Rust (`ui/src/main.rs`):** Nenhuma alteração de lógica ou novos callbacks são necessários. A infraestrutura de `on_auto_optimize_clicked`, `on_create_profile_confirmed` e sincronização reativa já atende perfeitamente ao comportamento esperado.
2. **No componente `AddPresetButton` (`ui/components/add_preset_button.slint`):**
   - Alterar `text: "NOVO PERFIL";` para `text: "Criar Otimização";`.
   - Ajustar `width: root.is-empty ? 190px : 44px;`.
3. **No arquivo principal Slint (`ui/main.slint`):**
   - Remover o botão `TacticalButton` "OTIMIZAR AUTOMATICAMENTE" da seção inferior (mantendo apenas a barra de status HUD).
   - Mover o bloco de parâmetros de inicialização (`if root.active-preset != "" : HorizontalLayout`) para antes do cabeçalho "PERFIS DE OTIMIZAÇÃO".
   - Na seção de perfis, criar a ramificação reativa:
     - Se `root.presets.length == 0`: exibir em `HorizontalLayout` exclusivamente os dois botões ("Otimizar Automaticamente" chamando `root.auto-optimize-clicked()` e `AddPresetButton` com `is-empty: true`).
     - Se `root.presets.length > 0`: exibir a lista de `PresetButton`s mais o `AddPresetButton { is-empty: false; }`.

---

## 5. Verification Method (Método de Verificação Independente)

Para o implementador e o validador verificarem de forma autônoma:

1. **Inspeção de Código:**
   - Validar que o botão inferior de otimização em `ui/main.slint` foi removido.
   - Validar que o input de parâmetros de inicialização está entre a grade de switches e a lista de perfis.
   - Validar que em `ui/components/add_preset_button.slint`, o texto exibido quando vazio é `"Criar Otimização"`.
2. **Verificação de Compilação e Testes:**
   - Executar `cargo check` no workspace para garantir que a sintaxe Slint e a compilação do crate `ui` passam sem erros.
   - Executar `cargo test` no crate `config` e `core`.
3. **Validação Visual com Hot Reload:**
   - Executar `preview.cmd` ou rodar a aplicação via `cargo run --bin counterrush-ui`.
   - **Cenário Sem Perfis:** Renomear ou mover temporariamente os arquivos de `config/optimzations/`. Validar que:
     - Apenas os dois botões ("Otimizar Automaticamente" e "Criar Otimização +") aparecem centralizados.
     - O botão inferior não existe.
     - O campo de parâmetros de inicialização não aparece.
     - Clicar em "Criar Otimização +" abre o modal de criação.
     - Clicar em "Otimizar Automaticamente" gera o `AutoDetect.json` e a tela atualiza instantaneamente para o estado preenchido com todos os switches calibrados.
   - **Cenário Com Perfis:** Validar que ao existir arquivos `.json`, os botões de seleção de perfil aparecem, o input de launch args aparece acima deles e abaixo dos switches, e a seleção entre perfis funciona normalmente.
