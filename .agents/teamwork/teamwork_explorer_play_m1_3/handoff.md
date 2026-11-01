# Relatório de Handoff — Explorer 3: Modais e Diálogos Slint

**Data/Hora**: 2026-11-01T18:47:00Z  
**Autor**: Explorer 3 (teamwork_explorer)  
**Objetivo**: Investigar componentes de modais, backdrop, estilos e integração do novo modal de ausência de perfis para o botão "Jogar CS2".

---

## 1. Observation (Observações Diretas)

### 1.1 Estrutura dos Diálogos Existentes

#### `ui/components/delete_profile_dialog.slint`:
- **Hierarquia Base**: Herda diretamente de `Rectangle` (`export component DeleteProfileDialog inherits Rectangle`).
- **Dimensões e Backdrop**:
  - Linhas 8-10:
    ```slint
    width: 980px;
    height: 620px;
    background: rgba(11, 15, 23, 0.88);
    ```
  - Cobre exatamente a totalidade da janela da aplicação (`min-width: 980px; min-height: 620px`).
- **Bloqueio de Cliques Subjacentes**:
  - Linhas 12-15:
    ```slint
    // Bloqueador de cliques de fundo para não interagir com a janela principal
    TouchArea {
        mouse-cursor: default;
    }
    ```
    O `TouchArea` consome todos os cliques no canvas 980x620, impedindo que cheguem aos botões ou abas inferiores.
- **Card Central Flutuante**:
  - Linhas 18-27:
    ```slint
    Rectangle {
        x: (parent.width - self.width) / 2;
        y: (parent.height - self.height) / 2;
        width: 440px;
        height: 190px;
        background: CS2Theme.bg-card;
        border-radius: CS2Theme.radius-md;
        border-width: 1px;
        border-color: CS2Theme.status-danger;
        clip: true;
    ```
- **Seções Internas do Card**:
  - Cabeçalho: Indicador circular (8x8px) + Título em caixa alta (`font-size: 13px; font-weight: 900; letter-spacing: 0.8px`) + Botão Fechar (✕ de 24x24px com `TouchArea`).
  - Divisor: Linha horizontal de 1px com cor `CS2Theme.border`.
  - Corpo da Mensagem: `VerticalLayout` com espaçamento de 4px, texto de destaque (`font-size: 12px; font-weight: 700; color: CS2Theme.text-primary`) e instrução secundária (`font-size: 10px; color: CS2Theme.text-dim`).
  - Rodapé de Ações: `HorizontalLayout` com alinhamento à direita (`alignment: end; spacing: 12px;`), botão secundário "Cancelar" (cinza/superfície) e botão primário de ação "Excluir" (vermelho perigo).

#### `ui/components/create_profile_dialog.slint`:
- Segue a mesma arquitetura de overlay: `width: 980px; height: 620px; background: rgba(11, 15, 23, 0.88);` com `TouchArea { mouse-cursor: default; }` na raiz.
- O card central tem `width: 720px; height: 540px;` e borda `CS2Theme.accent-orange`.
- Possui cabeçalho com botão fechar (✕) chamando `root.cancel();` e rodapé com botão Cancelar e botão de ação primária em Laranja CS2 (`CS2Theme.accent-orange` / `accent-orange-bright`).

### 1.2 Mecanismo de Backdrop e Ordem Z em `ui/main.slint`

Em `ui/main.slint`, linhas 1084-1110:
```slint
    // Modal Overlay de Criação de Perfil de Otimização (.json)
    if root.show-create-modal : CreateProfileDialog {
        error-message: root.create-modal-error;
        cancel => {
            root.show-create-modal = false;
            root.create-modal-error = "";
        }
        confirmed(data) => {
            root.create-profile-confirmed(data);
        }
    }

    // Modal Overlay de Confirmação de Exclusão de Perfil (.json)
    if root.show-delete-modal : DeleteProfileDialog {
        profile-name: root.target-delete-profile;
        cancel => {
            root.show-delete-modal = false;
            root.target-delete-profile = "";
        }
        confirmed => {
            root.show-delete-modal = false;
            root.delete-profile-confirmed(root.target-delete-profile);
            root.target-delete-profile = "";
        }
    }
```
- No Slint, elementos declarados no nível da janela após a estrutura de layout principal (como `HorizontalLayout { Sidebar ... }`) são renderizados no topo da pilha visual (maior z-index).
- A exibição é controlada por propriedades booleanas reativas na `MainWindow` (`show-create-modal`, `show-delete-modal`).
- Enquanto a propriedade for `true`, o diálogo renderiza seu backdrop cobrindo os 980x620px da janela e bloqueia todas as interações inferiores.

### 1.3 Botão "JOGAR CS2" em `ui/main.slint`

Em `ui/main.slint`, linhas 377-394:
```slint
TacticalButton {
    text: root.is-launching ? "INICIANDO..." : "JOGAR CS2";
    badge: root.is-launching ? "APLICANDO TWEAKS & STEAM" : "INICIAR COM ALTA PERFORMANCE";
    enabled: !root.is-launching;
    background-color: CS2Theme.accent-orange;
    hover-color: #ff771a;
    custom-border-color: #c2410c;
    custom-border-hover-color: #ff851b;
    custom-border-width: 3px;
    text-color: #ffffff;
    icon-src: @image-url("assets/icons/play.svg");
    clicked => {
        root.is-launching = true;
        root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
        root.launch-cs2-clicked();
    }
}
```
- O botão encontra-se na aba Painel (`active-tab == 0`).
- Atualmente, ao ser clicado, ele dispara incondicionalmente `root.is-launching = true` e `root.launch-cs2-clicked()`.
- A propriedade `root.presets` em `MainWindow` contém o vetor reativo de perfis (`in-out property <[PresetModel]> presets: [];`).
- Quando não há perfis salvos no disco, `root.presets.length == 0` (utilizado na linha 834 de `ui/main.slint`: `if root.presets.length == 0 : HorizontalLayout { ... }`).

### 1.4 Estilos e Tokens Globais (`ui/theme.slint`)
- **Fundo do Backdrop**: `rgba(11, 15, 23, 0.88)` (mesma base cromática do `CS2Theme.bg-main: #0b0f17`).
- **Card**: Fundo `CS2Theme.bg-card` (`#162132`), borda `1px` em `CS2Theme.accent-orange` (`#ff6a00`), raio `CS2Theme.radius-md` (`6px`).
- **Botão Primário ("Criar Perfil")**:
  - Fundo normal: `CS2Theme.accent-orange` (`#ff6a00`).
  - Fundo hover: `CS2Theme.accent-orange-bright` (`#ff851b`).
  - Borda: `1px solid #c2410c`.
  - Raio: `CS2Theme.radius-sm` (`4px`).
  - Texto: `font-size: 11px; font-weight: 900; letter-spacing: 0.5px; color: #ffffff;`.
- **Botão Secundário ("Sair")**:
  - Fundo normal: `CS2Theme.bg-surface` (`#111824`).
  - Fundo hover: `CS2Theme.bg-card-hover` (`#1e2c42`).
  - Borda: `1px solid CS2Theme.border` (`#24334a`), hover `CS2Theme.border-hover` (`#3b5275`).
  - Raio: `CS2Theme.radius-sm` (`4px`).
  - Texto: `font-size: 11px; font-weight: 800; letter-spacing: 0.5px; color: CS2Theme.text-muted` (hover `CS2Theme.text-primary`).

---

## 2. Logic Chain (Cadeia de Raciocínio Lógico)

1. **Premissa de Arquitetura**: O projeto adota componentes isolados por responsabilidade em `ui/components/` (ex: `delete_profile_dialog.slint`, `create_profile_dialog.slint`, `preset_button.slint`). Modais são independentes e comunicam intenções através de callbacks para o componente pai (`MainWindow`).
2. **Derivação do Componente**: O novo modal deve ser um novo arquivo `ui/components/no_profile_dialog.slint`, exportando `NoProfileDialog`. Isso respeita Clean Architecture, SOLID e a regra do projeto de não poluir arquivos monolíticos.
3. **Padrão de Backdrop**: Replicar a técnica de `width: 980px; height: 620px; background: rgba(11, 15, 23, 0.88);` com `TouchArea { mouse-cursor: default; }` garante isolamento perfeito, impedindo qualquer clique na interface inferior durante a exibição do diálogo.
4. **Tratamento de Evento no Botão "JOGAR CS2"**:
   - Em `ui/main.slint`, dentro do `clicked => { ... }` do `TacticalButton`, insere-se a verificação condicional:
     ```slint
     if root.presets.length == 0 {
         root.show-no-profile-modal = true;
     } else {
         root.is-launching = true;
         root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
         root.launch-cs2-clicked();
     }
     ```
   - Isso impede o início indevido da Steam/processo e abre imediatamente o diálogo quando não há perfis.
5. **Comportamento dos Callbacks Solicitados**:
   - **Botão "Criar Perfil"**: Requisito: *"O botão criar perfil leva o usuário para a aba otimizações."*
     - No handler do modal em `ui/main.slint`: fecha o modal (`root.show-no-profile-modal = false`), muda a aba ativa para 1 (`root.active-tab = 1`) e atualiza `root.status-message = "Visualizando Ajustes Avançados de Hardware"`.
   - **Botão "Sair"**: Requisito: *"O botão sair fecha o modal."*
     - No handler do modal em `ui/main.slint`: define `root.show-no-profile-modal = false`.
   - **Botão Fechar (✕)**: Como em todos os outros diálogos da aplicação, aciona o mesmo callback de fechamento do modal (`root.close()`).

---

## 3. Caveats (Ressalvas e Observações)

1. **Abertura Direta do Diálogo de Criação vs Navegação de Aba**:
   - O requisito declara: *"O botão criar perfil leva o usuário para a aba otimizações."*
   - Na aba otimizações (`active-tab == 1`), quando não há perfis, a interface exibe os botões `"OTIMIZAR AUTOMATICAMENTE"` e `"Criar Otimização"` (`AddPresetButton`).
   - Se for desejado que além de navegar para a aba 1 o modal de criação de perfil (`show-create-modal`) seja aberto automaticamente, bastaria adicionar `root.show-create-modal = true;`. No entanto, para cumprir estritamente a especificação do usuário, a navegação de aba com fechamento do diálogo é o comportamento exato exigido.
2. **Defesa em Profundidade no Backend Rust**:
   - Embora a checagem em Slint `root.presets.length == 0` bloqueie o disparo de `launch-cs2-clicked()`, é recomendável que o handler Rust `on_launch_cs2_clicked` em `ui/src/main.rs` também verifique se `presets.is_empty()` ou se o caminho está vazio, abrindo o modal reativo caso ocorra alguma inconsistência.
3. **Arquivo de Template Disponibilizado**:
   - O código completo do novo componente foi gerado em `.agents/teamwork/teamwork_explorer_play_m1_3/proposed_no_profile_dialog.slint` para servir de base direta e sem ambiguidades para a fase de implementação.

---

## 4. Conclusion (Conclusão e Proposta de Implementação)

### 4.1 Novo Arquivo: `ui/components/no_profile_dialog.slint`
Criar o arquivo com o conteúdo idêntico a `proposed_no_profile_dialog.slint`, contendo:
- Mensagem exata: `"Nenhum Perfil de Otimização encontrado, Deseja criar um?"`
- Botão secundário: `"Sair"`
- Botão primário: `"Criar Perfil"`
- Botão de fechar: `"✕"`
- Callbacks: `callback create-profile();` e `callback close();`

### 4.2 Alterações em `ui/main.slint`
1. **Importação** (linha 9):
   ```slint
   import { NoProfileDialog } from "components/no_profile_dialog.slint";
   ```
2. **Propriedade** (bloco de propriedades da `MainWindow`):
   ```slint
   in-out property <bool> show-no-profile-modal: false;
   ```
3. **Condicional no Botão "JOGAR CS2"** (linhas 388-393):
   ```slint
   clicked => {
       if root.presets.length == 0 {
           root.show-no-profile-modal = true;
       } else {
           root.is-launching = true;
           root.status-message = "Iniciando Counter-Strike 2 pela Steam com perfil otimizado...";
           root.launch-cs2-clicked();
       }
   }
   ```
4. **Instanciação do Diálogo** (ao final de `MainWindow`, junto aos outros modais):
   ```slint
   // Modal Overlay de Aviso: Ausência de Perfis ao Jogar CS2
   if root.show-no-profile-modal : NoProfileDialog {
       create-profile => {
           root.show-no-profile-modal = false;
           root.active-tab = 1;
           root.status-message = "Ajustes Avançados de Hardware";
       }
       close => {
           root.show-no-profile-modal = false;
       }
   }
   ```

---

## 5. Verification Method (Método de Verificação Independente)

Para validar a implementação de forma independente:
1. **Verificação de Compilação Slint/Rust**:
   Executar a checagem do compilador Slint e Rust no pacote de UI:
   `cargo check -p counterrush-ui`
   *Condição de invalidação*: Qualquer erro de sintaxe, tipo de callback ou referência de arquivo no Slint ou Rust.
2. **Inspeção Estática de Arquivos**:
   - Verificar presença de `ui/components/no_profile_dialog.slint`.
   - Verificar importação e instanciação em `ui/main.slint`.
3. **Cenários Visuais e Comportamentais (HotReload ou Execução)**:
   - **Cenário 1 (Sem perfis)**: Sem arquivos em `config/optimzations/`. Ao iniciar o app e clicar em "JOGAR CS2" na aba Painel:
     - O modal "AVISO DE PERFIL" deve aparecer centralizado com fundo escurecido.
     - A interface de trás não deve responder a cliques.
     - Clicar em "Sair" ou no "✕" deve fechar o modal e permanecer na aba Painel.
     - Clicar em "Criar Perfil" deve fechar o modal e alternar para a aba "Otimizações" (`active-tab = 1`).
   - **Cenário 2 (Com perfis)**: Com ao menos 1 perfil em `config/optimzations/`. Ao clicar em "JOGAR CS2":
     - O modal NÃO deve abrir. O fluxo de lançamento do jogo deve ocorrer normalmente.
