# Relatório de Handoff — Investigação da Camada de Interface (Slint UI)

**Subagente**: `teamwork_explorer_del_survey_1`  
**Data/Hora**: 2026-11-01T17:43:00Z  
**Contexto**: Investigação para adição de exclusão de perfis de otimização na aba de otimizações com ícone de lixeira, modal de confirmação e transição de estado.

---

## 1. Observation (Observações Diretas)

### 1.1 Renderização dos Perfis de Otimização
* **Arquivo `ui/main.slint` (linhas 9-12)**:
  ```slint
  export struct PresetModel {
      name: string,
      tag: string,
  }
  ```
* **Arquivo `ui/main.slint` (linhas 43-44, 72)**:
  ```slint
  in-out property <string> active-preset: "";
  in-out property <[PresetModel]> presets: [];
  ...
  callback preset-selected(string);
  ```
* **Arquivo `ui/main.slint` (linhas 861-920, dentro de `if root.active-tab == 1`)**:
  ```slint
  // Seção Intermediária (Com Perfis): Perfis de Otimização (config/optimzations) - Seleção Única (R4)
  if root.presets.length > 0 : VerticalLayout {
      spacing: 8px;
      alignment: center;
      ...
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
                  root.is-editing-launch-args = false;
                  root.status-message = "Perfil ativo: " + preset.name;
                  root.preset-selected(preset.name);
              }
          }

          AddPresetButton {
              is-empty: false;
              clicked => {
                  root.create-modal-error = "";
                  root.show-create-modal = true;
              }
          }
      }
  }
  ```
* **Arquivo `ui/main.slint` (linhas 830-858)** — Estado padrão quando não há perfis:
  ```slint
  if root.presets.length == 0 : HorizontalLayout {
      alignment: center;
      spacing: 16px;

      TacticalButton {
          text: "OTIMIZAR AUTOMATICAMENTE";
          badge: "PERFIL RECOMENDADO • BAIXA LATÊNCIA";
          ...
          clicked => { root.auto-optimize-clicked(); }
      }

      AddPresetButton {
          is-empty: true;
          clicked => {
              root.create-modal-error = "";
              root.show-create-modal = true;
          }
      }
  }
  ```
* **Arquivo `ui/main.slint` (linhas 709-827)** — Parâmetros de inicialização associados ao perfil:
  Exibidos somente quando `root.active-preset != ""`.

### 1.2 Estrutura do Componente `PresetButton`
* **Arquivo `ui/components/preset_button.slint` (linhas 3-124)**:
  - Componente: `PresetButton inherits Rectangle`
  - Propriedades de entrada:
    - `in property <string> file-name: "custom_optimization";`
    - `in property <string> tag: "PERFIL JSON";`
    - `in property <bool> is-active: false;`
    - `in property <color> accent-color: CS2Theme.accent-ct-blue;`
    - `in property <color> active-bg: CS2Theme.accent-ct-soft;`
    - `in property <length> button-width: 350px;`
    - `callback selected();`
  - Estrutura interna:
    - `HorizontalLayout` com `spacing: 6px; alignment: space-between;`
    - Lado esquerdo: Indicador circular (beacon 7x7px), ícone SVG (`@image-url("../assets/icons/cfg.svg")`), e `VerticalLayout` com o nome do arquivo (`file-name`) e tag.
    - Lado direito: Pílula HUD de status com texto `"ATIVO"` ou `"USAR"`.
    - Área de clique atual: `touch := TouchArea` que cobre todo o componente no final do arquivo e aciona `root.selected()`.

### 1.3 Ícones e Padrões Visuais
* Diretório `ui/assets/icons/`:
  Contém apenas arquivos SVG vetoriais: `cfg.svg`, `crosshair.svg`, `crown.svg`, `hardware.svg`, `home.svg`, `info.svg`, `map.svg`, `pencil.svg`, `play.svg`, `rocket.svg`, `save.svg`, `settings.svg`.
* Não existem arquivos de fontes TTF/WOFF no projeto. Todos os ícones seguem a especificação Lucide/Feather:
  - `viewBox="0 0 24 24"`
  - `fill="none"`
  - `stroke="currentColor"`
  - `stroke-width="2"`
  - `stroke-linecap="round"`
  - `stroke-linejoin="round"`
* Não existe atualmente um ícone de lixeira (`trash.svg`) no diretório `ui/assets/icons/`.

### 1.4 Padrão de Construção de Modais no Slint
* **Arquivo `ui/components/create_profile_dialog.slint` (linhas 77-535)** e **`ui/main.slint` (linhas 1078-1087)**:
  - O modal é instanciado na raiz de `MainWindow` fora dos layouts de abas, renderizando-se sobre todo o conteúdo (Z-index superior):
    ```slint
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
    ```
  - Dimensões do overlay: `width: 980px; height: 620px; background: rgba(11, 15, 23, 0.88);`
  - Bloqueio de cliques: Contém um `TouchArea { mouse-cursor: default; }` como primeiro filho, que consome todos os cliques no backdrop para não vazar para os botões da janela principal.
  - Card centralizado: Um `Rectangle` posicionado com `x: (parent.width - self.width) / 2; y: (parent.height - self.height) / 2;` com `background: CS2Theme.bg-card; border-radius: CS2Theme.radius-md; border-width: 1px;`.
  - Botões de ação no rodapé: Botão secundário de Cancelar e botão primário de Confirmação com `TouchArea` individual.

---

## 2. Logic Chain (Cadeia Lógica de Raciocínio)

1. **Da renderização dos perfis (Obs 1.1)**:
   Os perfis são renderizados dinamicamente via `for preset in root.presets : PresetButton` na aba 1 (`Central de Otimizações`). Portanto, qualquer novo elemento que deva pertencer a cada perfil (como o ícone de lixeira) deve ser incorporado diretamente dentro do componente `PresetButton` em `ui/components/preset_button.slint`.

2. **Do ícone e padrão visual (Obs 1.3)**:
   Como todo o design system utiliza SVGs Feather/Lucide 24x24 com `stroke="currentColor"` e estilização via `colorize` no Slint, criar `ui/assets/icons/trash.svg` com a especificação padrão Lucide (`polyline points="3 6 5 6 21 6"`, `path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"`, `line x1="10" y1="11" x2="10" y2="17"`, `line x1="14" y1="11" x2="14" y2="17"`) garante coerência visual de 100% com os ícones já existentes (`pencil.svg`, `save.svg`).

3. **Da interatividade e Z-order no `PresetButton` (Obs 1.2)**:
   Atualmente, `PresetButton` possui um `touch := TouchArea` no final do arquivo que cobre o retângulo inteiro para selecionar o perfil (`root.selected()`). Se adicionarmos o botão de lixeira dentro do `HorizontalLayout`, mas mantivermos o `touch` cobrindo tudo por cima, o clique na lixeira seria interceptado pela seleção do perfil.
   Para resolver isso de forma elegante no Slint:
   - O `touch := TouchArea` base de seleção deve ser posicionado antes do conteúdo que tem interações específicas, ou a lixeira deve possuir seu próprio `TouchArea` (`del-touch := TouchArea`) declarado em nível superior de Z-order.
   - Declarando o `touch` de seleção antes do `HorizontalLayout`, qualquer elemento filho dentro de `HorizontalLayout` com um `TouchArea` próprio (`del-touch`) receberá os cliques de forma exclusiva, enquanto cliques nas demais áreas do card continuam acionando `touch.clicked` (`root.selected()`).

4. **Da estrutura do modal de confirmação (Obs 1.4 e Requisitos do Usuário)**:
   O usuário solicitou:
   - "Deve existir uma confirmação se o usuário realmente deseja excluir o perfil."
   - "Essa mensagem de confirmação deve ser um pequeno modal com o texto 'Deseja realmente excluir o perfil {Nome do perfil}' com dois botões 'Cancelar', 'Excluir'."
   Seguindo o padrão do `CreateProfileDialog`:
   - Criar `ui/components/delete_profile_dialog.slint` (ou `confirm_delete_dialog.slint`).
   - Dimensões compactas ("pequeno modal"): largura 440px, altura 190px (focado, sem rolagem, direto).
   - Card com borda tática de perigo `border-color: CS2Theme.status-danger` (#ef4444) e fundo `CS2Theme.bg-card`.
   - Mensagem: `"Deseja realmente excluir o perfil " + root.profile-name` (com tipografia de destaque).
   - Dois botões:
     - Botão "Cancelar" (estilo neutro, `CS2Theme.bg-surface`, fecha modal).
     - Botão "Excluir" (estilo perigo/destrutivo, fundo `CS2Theme.status-danger`, confirma exclusão).

5. **Do fluxo de eventos e callbacks (Obs 1.1, Obs 1.4)**:
   - No `PresetButton`: expor `callback delete();`.
   - Em `ui/main.slint`:
     - Adicionar propriedades:
       - `in-out property <bool> show-delete-modal: false;`
       - `in-out property <string> target-delete-profile: "";`
     - No loop `for preset in root.presets : PresetButton`:
       ```slint
       delete => {
           root.target-delete-profile = preset.name;
           root.show-delete-modal = true;
       }
       ```
     - Adicionar o overlay condicional no final de `MainWindow`:
       ```slint
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
     - Expor callback para o backend Rust:
       `callback delete-profile-confirmed(string);`

6. **Da reversão ao estado padrão quando o último perfil é excluído (Obs 1.1)**:
   O Slint já possui a condição `if root.presets.length == 0` que renderiza o botão "OTIMIZAR AUTOMATICAMENTE" e o `AddPresetButton` em modo inicial (`is-empty: true`), ocultando os seletores e os campos de parâmetros (`if root.active-preset != ""`).
   Portanto, quando o backend Rust remover o arquivo e atualizar `presets` para um array vazio e `active-preset` para `""`, a interface Slint reverterá instantânea e automaticamente para o estado inicial padrão sem necessidade de lógica visual complexa adicional!

---

## 3. Caveats (Ressalvas e Limitações)

1. **Escopo Read-Only**: Esta investigação apenas mapeou e desenhou a solução da interface, sem modificar nenhum arquivo fonte ou criar os componentes finais.
2. **Exclusão de arquivos no disco**: A exclusão real do arquivo `.json` em `config/optimzations/` deverá ser realizada pelo backend Rust através do callback `on_delete_profile_confirmed`.
3. **Perfil ativo excluído**: Caso o perfil excluído não seja o único, mas seja o que estava atualmente selecionado, o backend Rust deverá selecionar automaticamente o primeiro perfil remanescente e carregar suas configurações.

---

## 4. Conclusion (Conclusão e Proposta de Implementação)

Para atender a todos os requisitos da solicitação original, a implementação na camada UI divide-se em 4 partes bem definidas:

### A. Novo Asset de Ícone: `ui/assets/icons/trash.svg`
Criar o arquivo SVG Lucide no formato exato do projeto:
```xml
<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">
  <polyline points="3 6 5 6 21 6"/>
  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
  <line x1="10" y1="11" x2="10" y2="17"/>
  <line x1="14" y1="11" x2="14" y2="17"/>
</svg>
```

### B. Novo Componente: `ui/components/delete_profile_dialog.slint`
```slint
import { CS2Theme } from "../theme.slint";

export component DeleteProfileDialog inherits Rectangle {
    in property <string> profile-name: "";
    callback cancel();
    callback confirmed();

    width: 980px;
    height: 620px;
    background: rgba(11, 15, 23, 0.88);

    // Bloqueador de cliques de fundo para não vazar à MainWindow
    TouchArea {
        mouse-cursor: default;
    }

    // Card Centralizado Compacto
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

        VerticalLayout {
            padding: 20px;
            spacing: 14px;
            alignment: space-between;

            // Cabeçalho com Título de Alerta e Botão Fechar
            HorizontalLayout {
                alignment: space-between;

                HorizontalLayout {
                    spacing: 8px;
                    alignment: start;

                    Rectangle {
                        width: 8px;
                        height: 8px;
                        border-radius: 4px;
                        background: CS2Theme.status-danger;
                        y: (parent.height - self.height) / 2;
                    }

                    Text {
                        text: "EXCLUIR PERFIL";
                        font-size: 13px;
                        font-weight: 900;
                        letter-spacing: 0.8px;
                        color: CS2Theme.status-danger;
                        vertical-alignment: center;
                    }
                }

                // Botão Fechar (✕)
                Rectangle {
                    width: 24px;
                    height: 24px;
                    border-radius: CS2Theme.radius-xs;
                    background: close-touch.has-hover ? CS2Theme.bg-card-hover : CS2Theme.bg-surface;
                    border-width: 1px;
                    border-color: close-touch.has-hover ? CS2Theme.border-hover : CS2Theme.border;

                    Text {
                        text: "✕";
                        font-size: 11px;
                        font-weight: 800;
                        color: close-touch.has-hover ? #ffffff : CS2Theme.text-dim;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }

                    close-touch := TouchArea {
                        mouse-cursor: pointer;
                        clicked => {
                            root.cancel();
                        }
                    }
                }
            }

            Rectangle {
                height: 1px;
                background: CS2Theme.border;
            }

            // Mensagem de Confirmação Obrigatória
            VerticalLayout {
                spacing: 4px;

                Text {
                    text: "Deseja realmente excluir o perfil " + root.profile-name + "?";
                    font-size: 12px;
                    font-weight: 700;
                    color: CS2Theme.text-primary;
                    wrap: word-wrap;
                }

                Text {
                    text: "Esta ação não pode ser desfeita e removerá o arquivo .json do disco.";
                    font-size: 10px;
                    color: CS2Theme.text-dim;
                }
            }

            // Rodapé com os dois botões: Cancelar e Excluir
            HorizontalLayout {
                alignment: end;
                spacing: 12px;

                // Botão Cancelar
                Rectangle {
                    width: 100px;
                    height: 34px;
                    background: cancel-touch.has-hover ? CS2Theme.bg-card-hover : CS2Theme.bg-surface;
                    border-radius: CS2Theme.radius-sm;
                    border-width: 1px;
                    border-color: cancel-touch.has-hover ? CS2Theme.border-hover : CS2Theme.border;

                    Text {
                        text: "Cancelar";
                        font-size: 11px;
                        font-weight: 800;
                        letter-spacing: 0.5px;
                        color: cancel-touch.has-hover ? CS2Theme.text-primary : CS2Theme.text-muted;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }

                    cancel-touch := TouchArea {
                        mouse-cursor: pointer;
                        clicked => {
                            root.cancel();
                        }
                    }
                }

                // Botão Excluir
                Rectangle {
                    width: 100px;
                    height: 34px;
                    background: confirm-touch.has-hover ? #dc2626 : CS2Theme.status-danger;
                    border-radius: CS2Theme.radius-sm;
                    border-width: 1px;
                    border-color: #991b1b;

                    Text {
                        text: "Excluir";
                        font-size: 11px;
                        font-weight: 900;
                        letter-spacing: 0.5px;
                        color: #ffffff;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }

                    confirm-touch := TouchArea {
                        mouse-cursor: pointer;
                        clicked => {
                            root.confirmed();
                        }
                    }
                }
            }
        }
    }
}
```

### C. Alterações no `ui/components/preset_button.slint`
1. Adicionar o callback: `callback delete();`
2. No bloco direito do layout (ao lado da pílula HUD), inserir o botão de lixeira:
```slint
// Bloco Direito: Pílula HUD + Botão Lixeira
HorizontalLayout {
    spacing: 6px;
    alignment: end;

    // Pílula Lateral Direita de Estado HUD (ATIVO / USAR)
    Rectangle {
        width: root.is-active ? 62px : 56px;
        height: 22px;
        background: root.is-active ? root.active-bg : CS2Theme.bg-surface;
        border-radius: CS2Theme.radius-xs;
        border-width: 1px;
        border-color: root.is-active ? root.accent-color : CS2Theme.border;
        y: (parent.height - self.height) / 2;
        ...
    }

    // Botão Lixeira Tático
    Rectangle {
        width: 24px;
        height: 22px;
        y: (parent.height - self.height) / 2;
        background: del-touch.has-hover ? rgba(239, 68, 68, 0.20) : transparent;
        border-radius: CS2Theme.radius-xs;
        border-width: 1px;
        border-color: del-touch.has-hover ? CS2Theme.status-danger : (root.is-active ? root.accent-color.transparentize(60%) : CS2Theme.border);
        animate background { duration: 120ms; }
        animate border-color { duration: 120ms; }

        Image {
            source: @image-url("../assets/icons/trash.svg");
            width: 12px;
            height: 12px;
            colorize: del-touch.has-hover ? CS2Theme.status-danger : CS2Theme.text-dim;
            x: (parent.width - self.width) / 2;
            y: (parent.height - self.height) / 2;
            animate colorize { duration: 120ms; }
        }

        del-touch := TouchArea {
            mouse-cursor: pointer;
            clicked => {
                root.delete();
            }
        }
    }
}
```
3. O `touch := TouchArea` que seleciona o perfil deve ser declarado antes de `HorizontalLayout` para garantir que `del-touch` tenha prioridade de clique.

### D. Alterações em `ui/main.slint`
1. Importar o diálogo:
   `import { DeleteProfileDialog } from "components/delete_profile_dialog.slint";`
2. Adicionar propriedades reativas em `MainWindow`:
   ```slint
   in-out property <bool> show-delete-modal: false;
   in-out property <string> target-delete-profile: "";
   ```
3. Adicionar callback do backend:
   ```slint
   callback delete-profile-confirmed(string);
   ```
4. Conectar o evento `delete` no loop de `PresetButton`:
   ```slint
   for preset in root.presets : PresetButton {
       ...
       delete => {
           root.target-delete-profile = preset.name;
           root.show-delete-modal = true;
       }
   }
   ```
5. Adicionar a sobreposição no final de `MainWindow`:
   ```slint
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

---

## 5. Verification Method (Método de Verificação Independente)

Para que qualquer agente ou desenvolvedor valide de forma independente esta análise:
1. **Inspeção de Arquivos de UI**:
   - Inspecionar `ui/main.slint` linhas 830–920 para confirmar como `root.presets` e `root.presets.length == 0` funcionam.
   - Inspecionar `ui/components/preset_button.slint` para confirmar dimensões (altura 44px, largura 250px ou 350px) e o layout com a pílula HUD.
   - Inspecionar `ui/components/create_profile_dialog.slint` para confirmar o mecanismo de overlay, bloqueio com `TouchArea` e centralização do card.
2. **Inspeção de Assets**:
   - Executar `find_by_name` em `ui/assets/icons` para comprovar que não existe arquivo de fonte e que todos os ícones são SVGs individuais.
3. **Validação do Compilador Slint / Cargo**:
   - Quando os arquivos forem adicionados/alterados na fase de implementação, compilar com:
     `cargo check --manifest-path ui/Cargo.toml`
     e certificar-se de que a compilação do Slint e Rust passe sem erros ou avisos.
