# Relatório de Investigação da Arquitetura Slint UI — Requisitos R2 e R3

**Agente**: `teamwork_explorer_survey_2` (UI Architecture Explorer)  
**Data**: 2026-10-30  
**Contexto**: Análise da interface gráfica Slint UI e pontes com backend Rust para implementação dos requisitos R2 (Botão "+" na listagem de perfis) e R3 (Modal/Diálogo de criação e persistência de perfil `.json`).

---

## 1. Observation (Observações Diretas do Código)

### 1.1. Localização e Arquitetura dos Arquivos Slint
A interface gráfica é gerenciada no crate `counterrush-ui` localizado em `g:\Outros computadores\Meu laptop\Desktop\CounterRush\ui`.
Os arquivos e componentes observados são:

1. **`ui/Cargo.toml`**:
   - Dependências: `slint = "1.8"` e `counterrush_core = { package = "core", path = "../core" }` (linhas 10-11).
   - Build-dependencies: `slint-build = "1.8"` (linha 14).
2. **`ui/build.rs`**:
   - Compilação centralizada via `slint_build::compile("main.slint")` (linha 2). Qualquer arquivo importado em `main.slint` é compilado automaticamente.
3. **`ui/main.slint`**:
   - Componente raiz: `export component MainWindow inherits Window` (linha 12).
   - Dimensões fixas da janela: `min-width: 980px; max-width: 980px; min-height: 620px; max-height: 620px;` (linhas 14-19).
   - Background: `CS2Theme.bg-main` (`#0b0f17`).
4. **`ui/theme.slint`**:
   - Global `CS2Theme`: Cores táticas (`bg-main: #0b0f17`, `bg-surface: #111824`, `bg-card: #162132`, `accent-orange: #ff6a00`, `accent-ct-cyan: #38bdf8`, `status-success: #10b981`, `status-danger: #ef4444`), raios angulares (`radius-xs: 2px`, `radius-sm: 4px`, `radius-md: 6px`) e espaçamentos (linhas 1-62).
5. **`ui/components/sidebar.slint`**:
   - Largura fixa: `width: 148px;` (linha 48).
   - Contém 4 abas de navegação (linhas 93-118):
     - Aba 0: "Painel" (`active-tab == 0`)
     - Aba 1: "Otimizações" (`active-tab == 1`) com ícone `assets/icons/rocket.svg`
     - Aba 2: "Configurações" (`active-tab == 2`)
     - Aba 3: "Ranking CS" (`active-tab == 3`)
6. **`ui/components/preset_button.slint`**:
   - Componente de exibição de cada perfil individual: altura `44px`, borda com raio `CS2Theme.radius-sm`, beacon luminoso, ícone `cfg.svg`, nome do arquivo, tag e pílula lateral HUD ("ATIVO" / "USAR") (linhas 1-125).
7. **`ui/components/tweak_switch.slint`**:
   - Componentes `TweakSwitch` (altura `50px`) e `MasterToggleSwitch` para os 13 módulos de otimização (linhas 1-256).

### 1.2. Localização e Estrutura da Aba de Otimizações
- Em `ui/main.slint`, linhas 411 a 805:
  ```slint
  if root.active-tab == 1 : VerticalLayout {
      padding-left: 19px;
      padding-right: 19px;
      padding-top: 18px;
      padding-bottom: 16px;
      alignment: space-between;
      ...
  ```
- Largura útil da área central na aba: 980px (janela) - 148px (sidebar) - 38px (paddings) = **794px**.
- Estrutura vertical dividida em 3 blocos:
  1. *Linhas 419-699*: Cabeçalho da central e grade dos 13 switches de otimização (4 linhas).
  2. *Linhas 701-752*: **Seção Intermediária: Perfis de Otimização (config/optimzations)**.
  3. *Linhas 754-805*: Botão "OTIMIZAR AUTOMATICAMENTE" e barra de status HUD.

### 1.3. Listagem Atual de Perfis e Alimentação pelo Rust
- **Em `ui/main.slint`**:
  - Definição do modelo de dados (linhas 7-10):
    ```slint
    export struct PresetModel {
        name: string,
        tag: string,
    }
    ```
  - Propriedades reativas padrão na `MainWindow` (linhas 39-43):
    ```slint
    in-out property <string> active-preset: "custom_optimization";
    in-out property <[PresetModel]> presets: [
        { name: "AutoDetect", tag: "HARDWARE ADAPTATIVO" },
        { name: "custom_optimization", tag: "CUSTOMIZADO / MANUAL" },
    ];
    ```
  - Renderização dos botões (linhas 734-751):
    ```slint
    HorizontalLayout {
        alignment: center;
        spacing: root.presets.length > 2 ? 10px : 16px;

        for preset in root.presets : PresetButton {
            file-name: preset.name;
            tag: preset.tag;
            is-active: root.active-preset == preset.name;
            accent-color: preset.name == "AutoDetect" ? CS2Theme.accent-ct-cyan : CS2Theme.accent-orange;
            active-bg: preset.name == "AutoDetect" ? CS2Theme.accent-ct-soft : CS2Theme.accent-orange-soft;
            button-width: root.presets.length > 2 ? 250px : 350px;
            selected => {
                root.active-preset = preset.name;
                root.status-message = "Perfil ativo: " + preset.name;
                root.preset-selected(preset.name);
            }
        }
    }
    ```
- **Em `ui/src/main.rs`**:
  - `scan_presets()` (linhas 5-32):
    Chama `counterrush_core::list_optimization_configs()`. Se vazio, insere artificialmente `AutoDetect` e `custom_optimization`.
  - Inicialização (linhas 38-40):
    `main_window.set_presets(slint::ModelRc::new(slint::VecModel::from(initial_presets.clone())));`
  - Sincronização em tempo real (linhas 78-104):
    Thread monitora a cada 1.5s mudanças nos arquivos de `config/optimzations` e invoca `ui.set_presets(...)` via `slint::invoke_from_event_loop`.
  - Callback de seleção (linhas 454-486):
    `main_window.on_preset_selected(move |preset_name| { ... })` carrega o arquivo JSON via `counterrush_core::resolve_optimization_file_path()` e define os 13 switches na UI.

### 1.4. Modelo de Configuração no Backend (`core` e `config`)
- `config/src/lib.rs`, linhas 20-37:
  Struct `CustomOptimizationConfig` contendo `launch_args: Vec<String>` e 13 flags booleanas (`empty_standby_list`, `cs2_priority`, `cs2_affinity`, `disable_gpu_acceleration`, `apply_dpi_optimization`, `disable_memory_compression`, `apply_timer_resolution`, `apply_mmcss`, `disable_hpet`, `disable_game_bar`, `enable_msi`, `disable_power_saving`, `apply_render_queue`).
- Métodos auxiliares: `load_from_file`, `save_to_file` (cria diretórios pai automaticamente).
- `core/src/lib.rs`, linhas 825-899:
  - `resolve_optimizations_dir()`: Localiza o diretório canônico `config/optimzations`.
  - `list_optimization_configs()`: Retorna `Vec<PathBuf>` de todos os `.json` ordenados.
  - `resolve_optimization_file_path(name)`: Garante extensão `.json` e junta ao caminho do diretório de otimizações.

---

## 2. Logic Chain (Cadeia de Raciocínio Passo a Passo)

### 2.1. Arquitetura do Botão "+" (Requisito R2)

1. **Posicionamento Relativo no Layout Slint**:
   - Em `ui/main.slint`, o container da listagem de perfis é um `HorizontalLayout` com `alignment: center` (linhas 734-751).
   - No Slint, todos os elementos declarados sequencialmente dentro de um `HorizontalLayout` são posicionados lado a lado, da esquerda para a direita.
   - Portanto, ao posicionar o botão de adição (ex: `AddPresetButton` ou componente customizado) imediatamente após a instrução `for preset in root.presets : PresetButton { ... }`, temos duas garantias matemáticas de layout:
     - **Caso A (Nenhum perfil existente, `root.presets.length == 0`)**: O loop `for` não gera nenhum elemento filho. O botão "+" é o único filho presente no `HorizontalLayout`. Devido a `alignment: center`, ele é renderizado diretamente no centro da área de perfis, ficando imediatamente visível no primeiro start da aplicação.
     - **Caso B (Um ou mais perfis existentes)**: O loop `for` gera os `PresetButton` sucessivamente. O botão "+" é renderizado imediatamente à direita do último `PresetButton` gerado, respeitando o espaçamento (`spacing`) definido.
2. **Dimensionamento e Estética Visual Tática**:
   - Altura: Deve ser rigorosamente `44px`, igual à altura de `PresetButton` (`preset_button.slint:13`), garantindo alinhamento horizontal harmônico.
   - Largura:
     - Quando `presets.length > 0`: Formato compacto quadrado de `44px` (ou `48px`), contendo o símbolo "+" em destaque com `CS2Theme.accent-orange` e cantos arredondados `CS2Theme.radius-sm`.
     - Quando `presets.length == 0`: Pode assumir largura adaptativa (ex: `160px` ou `180px`) exibindo o texto "+ CRIAR NOVO PERFIL" para máxima clareza e affordance visual, ou permanecer como um botão tático minimalista centralizado com tooltip explicativo.
   - Cálculo de Largura no Container de 794px:
     - 0 perfis: 1 botão de 44px (ou 180px) -> Ocupa 44px a 180px de 794px (perfeitamente centralizado).
     - 1 perfil: 1x 350px + 16px (spacing) + 44px = 410px <= 794px.
     - 2 perfis: 2x 280px + 2x 14px + 44px = 632px <= 794px.
     - 3 perfis: 3x 210px + 3x 12px + 44px = 710px <= 794px.
3. **Disparo de Eventos e Controle de Interface**:
   - O clique no botão "+" deve ativar o modal.
   - Isso pode ser feito puramente na UI de forma instantânea através de uma propriedade reativa (ex: `root.show-create-modal = true;`), emitindo opcionalmente o callback `create-profile-requested()` para o backend Rust para pré-preenchimento ou limpeza de estados residuais.

### 2.2. Arquitetura da Janela / Diálogo de Criação (Requisito R3)

1. **Escolha Arquitetural: Modal Overlay vs Janela Secundária**:
   - Em Slint, instanciar uma segunda janela com `inherits Window` em runtime pode causar comportamentos complexos com o loop de eventos no Windows (problemas de z-order, foco, perda de estilo CS2Theme nativo, e criação de múltiplos ícones na barra de tarefas do Windows).
   - A solução canônica e de maior fidelidade visual no Slint é o **Modal Overlay In-Window**:
     - Um container `Rectangle` de sobreposição cobrindo `width: 100%; height: 100%;` (980x620px), posicionado no topo da árvore de elementos de `MainWindow`.
     - Fundo com transparência tática escura (`rgba(11, 15, 23, 0.88)`).
     - Um `TouchArea` cobrindo o fundo para capturar e consumir cliques, bloqueando qualquer interação acidental com a tela principal enquanto o modal estiver aberto.
     - Um card centralizado de `700px` de largura por `510px` a `530px` de altura, com borda tática (`CS2Theme.accent-orange`), cabeçalho, formulário e botões de ação.
2. **Componentes Necessários no Diálogo**:
   - **Cabeçalho**: Título "NOVO PERFIL DE OTIMIZAÇÃO", subtítulo tático e botão fechar ("✕").
   - **Campo 1 (Nome do Arquivo)**:
     - Label: "NOME DO ARQUIVO (.JSON)"
     - Elemento: `TextInput` embutido em um container `Rectangle` estilizado (altura `34px`, borda que se ilumina em laranja ao receber foco com `has-focus`).
     - Feedback de caminho: Texto auxiliar mostrando que será salvo em `config/optimzations/<nome>.json`.
   - **Campo 2 (Argumentos de Inicialização - `launch_args`)**:
     - Label: "PARÂMETROS DE INICIALIZAÇÃO STEAM (LAUNCH ARGS)"
     - Elemento: `TextInput` (altura `34px`) para inserção de flags (ex: `-novid -tickrate 128 +fps_max 0`).
   - **Campo 3 (Seleção dos 13 Módulos de Otimização)**:
     - Seção modular compacta organizada em 3 colunas, utilizando componentes de checkbox tático (`CompactTweakCheck`), cada um com altura de `30px` a `32px` e badge tático "✓".
     - Botões de conveniência rápidos: "Marcar Todos" e "Desmarcar Todos".
   - **Rodapé / Ações**:
     - Texto de mensagem de erro/validação reativa (vermelho `CS2Theme.status-danger`).
     - Botão "CANCELAR" (`TacticalButton` secundário ou `Rectangle` neutro).
     - Botão "CRIAR E ATIVAR PERFIL" (`TacticalButton` primário em laranja).
3. **Ponte de Comunicação com o Backend Rust**:
   - Definição de struct Slint para transporte seguro dos dados digitados:
     ```slint
     export struct NewProfileData {
         name: string,
         launch-args: string,
         empty-standby-list: bool,
         cs2-priority: bool,
         cs2-affinity: bool,
         disable-gpu-acceleration: bool,
         apply-dpi-optimization: bool,
         disable-memory-compression: bool,
         apply-timer-resolution: bool,
         apply-mmcss: bool,
         disable-hpet: bool,
         disable-game-bar: bool,
         enable-msi: bool,
         disable-power-saving: bool,
         apply-render-queue: bool,
     }
     ```
   - Callback na `MainWindow`: `callback create-profile-confirmed(NewProfileData);`.
   - Lógica de tratamento em `ui/src/main.rs`:
     1. Validação estrita do nome: não vazio, remoção de `.json` redundante no input, proibição de caracteres especiais do Windows (`/ \ : * ? " < > |`), verificação se o arquivo já existe. Se inválido, define propriedade `create-modal-error` e aborta sem fechar o modal.
     2. Tokenização de `launch_args`: `data.launch_args.split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>()`.
     3. Instanciação de `CustomOptimizationConfig` com os 13 booleanos e salvamento via `.save_to_file(&target_path)`.
     4. Atualização imediata da UI:
        - `ui.set_presets(slint::ModelRc::new(slint::VecModel::from(scan_presets())));`
        - `ui.set_active_preset(clean_name.into());`
        - Atualização do `Arc<RwLock<PathBuf>>` (`active_config_path`).
        - Atualização dos 13 switches de tela (`ui.set_cpu_tweak_active(...)`, etc.).
        - Limpeza do erro e fechamento do modal (`ui.set_show_create_modal(false)`).
        - Mensagem na barra de status HUD: `"Perfil '<nome>' criado e ativado com sucesso."`.

### 2.3. Integração com o Requisito R1 (Estado Inicial sem Perfis)
- Atualmente em `ui/src/main.rs` (linhas 21-30), o código injeta artificialmente `AutoDetect` e `custom_optimization` quando `config/optimzations` está vazio.
- Além disso, `ui/main.slint` inicia as propriedades com valores `true` e `active-preset: "custom_optimization"`.
- Para harmonizar R1, R2 e R3:
  1. `scan_presets()` deve retornar `Vec::new()` quando nenhum arquivo existir.
  2. As propriedades em `ui/main.slint` devem ter como valor padrão `presets: []`, `active-preset: ""`, e os 13 switches como `false`.
  3. No `ui/src/main.rs`, ao detectar que `initial_presets` está vazio, o programa não deve invocar `load_or_create_custom_config(None)` (o que geraria arquivo em disco prematuramente), mantendo os switches desligados e exibindo no HUD `"Nenhum perfil existente. Clique no botão '+' para criar um novo perfil."`.
  4. Dessa forma, na primeira execução, o botão "+" aparecerá sozinho e centralizado. Ao ser clicado, abrirá o modal (R3). Ao salvar, criará o primeiro arquivo, que se tornará imediatamente o ativo e o botão "+" passará a ficar à direita dele!

---

## 3. Caveats (Ressalvas e Limitações Identificadas)

1. **Overflow com muitos perfis**:
   - A largura disponível na área central é de 794px.
   - Com até 3 perfis e o botão "+", todos os botões cabem confortavelmente em linha única.
   - Se o usuário criar 4 ou mais perfis, pode haver aperto ou truncamento do texto caso a largura de cada botão não seja reduzida proporcionalmente ou se não for utilizado um container com rolagem horizontal (`Flickable`).
   - *Recomendação*: Na implementação, ajustar a fórmula de largura (`button-width: root.presets.length > 2 ? 210px : 280px`) e, se necessário, envelopar o container de botões em um `Flickable { viewport-height: 44px; interactive: true; }`.
2. **Sanitização de Nomes no Windows**:
   - Nomes reservados do Windows (ex: `CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`) devem ser tratados na validação Rust para evitar falhas silenciosas na criação do arquivo.
3. **Escopo do Agente**:
   - Este agente é READ-ONLY. Nenhuma alteração foi realizada nos arquivos de código-fonte (`main.slint`, `main.rs`, etc.).
   - As propostas acima foram estruturadas em formato de especificação de engenharia pronta para ser incorporada no Plano de Implementação (`implementation_plan.md`).

---

## 4. Conclusion (Conclusão e Recomendações Técnicas)

A arquitetura do CounterRush está perfeitamente preparada para a inclusão dos requisitos R2 e R3 com mínimo impacto nos arquivos existentes:

1. **Arquivos a Criar**:
   - `ui/components/add_preset_button.slint`: Componente dedicado para o botão "+", encapsulando o hover, dimensões (44px) e affordance visual tática.
   - `ui/components/create_profile_dialog.slint`: Componente dedicado para o modal de criação de perfil, encapsulando os inputs (`TextInput`), a grade dos 13 switches compactos e as validações visuais.
2. **Arquivos a Modificar**:
   - `ui/main.slint`:
     - Importar `AddPresetButton` e `CreateProfileDialog`.
     - Iniciar propriedades com estado neutro (vazio/false) para compatibilidade com R1.
     - Inserir o botão `AddPresetButton` no `HorizontalLayout` de perfis após o loop `for`.
     - Inserir o modal `CreateProfileDialog` como overlay na raiz da janela.
     - Adicionar o callback `create-profile-confirmed(NewProfileData)` e a propriedade `show-create-modal`.
   - `ui/src/main.rs`:
     - Remover a injeção artificial de perfis falsos em `scan_presets()`.
     - Implementar o tratamento de primeira execução sem arquivos (desativar switches e não criar arquivo fantasma).
     - Implementar o callback `on_create_profile_confirmed` com validação de nome, gravação do arquivo `.json` via `CustomOptimizationConfig::save_to_file`, atualização da lista e ativação imediata do novo perfil na interface.

---

## 5. Verification Method (Método de Verificação Independente)

Para validar de forma independente a arquitetura e viabilidade:

1. **Inspeção de Código**:
   - Verificar `ui/main.slint` linhas 734-751 para confirmar o posicionamento relativo no `HorizontalLayout`.
   - Verificar `ui/src/main.rs` linhas 5-32 para constatar a injeção artificial de perfis no fallback atual.
   - Verificar `config/src/lib.rs` linhas 20-37 para confirmar os 13 campos booleanos e `launch_args`.
2. **Checagem de Compilação com Slint**:
   - Após a escrita dos novos arquivos `.slint`, executar:
     ```powershell
     cargo check -p counterrush-ui
     ```
   - O `build.rs` deve compilar `main.slint` e todos os componentes importados sem avisos de tipagem ou sintaxe.
3. **Verificação de Runtime**:
   - Iniciar com diretório `config/optimzations` vazio: o botão "+" deve aparecer centralizado; switches desligados.
   - Clicar no botão "+": o modal deve abrir sobre a tela com foco no campo de nome.
   - Preencher nome `teste_perfil`, launch_args `-novid`, marcar módulos e clicar em "Criar e Ativar":
     - O arquivo `config/optimzations/teste_perfil.json` deve ser criado.
     - O botão `teste_perfil` deve aparecer na tela principal marcado como "ATIVO".
     - O botão "+" deve aparecer imediatamente à direita do botão `teste_perfil`.
