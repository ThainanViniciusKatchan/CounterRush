# Relatório de Investigação Técnica: Backend Rust & Bridge Rust <-> Slint

## 1. Observação

Foram inspecionados os arquivos de código-fonte e configuração relevantes no repositório:
- `config/src/lib.rs`
- `core/src/lib.rs`
- `ui/src/main.rs`
- `ui/main.slint`
- `ui/components/preset_button.slint`
- `ui/components/create_profile_dialog.slint`
- `PROJECT.md` e `ORIGINAL_REQUEST.md`

### 1.1 Persistência e Listagem de Perfis
- Em `core/src/lib.rs` (linhas 1013–1035), a função `resolve_optimizations_dir()` define o diretório de perfis como `config/optimzations` (observar a grafia exata `optimzations` sem o "i" intermediário):
  ```rust
  pub fn resolve_optimizations_dir() -> PathBuf {
      if let Ok(exe_path) = std::env::current_exe() {
          if let Some(exe_dir) = exe_path.parent() {
              let candidate = exe_dir.join("config").join("optimzations");
              if candidate.is_dir() {
                  return candidate;
              }
          }
      }
      let direct = PathBuf::from("config/optimzations");
      if direct.is_dir() {
          return direct;
      }
      ...
  }
  ```
- Em `core/src/lib.rs` (linhas 1037–1059), as funções `list_optimization_configs_in_dir` e `list_optimization_configs` filtram e ordenam alfabeticamente arquivos com extensão `.json`:
  ```rust
  pub fn list_optimization_configs() -> Vec<PathBuf> {
      list_optimization_configs_in_dir(resolve_optimizations_dir())
  }
  ```
- Em `core/src/lib.rs` (linhas 1061–1069), `resolve_optimization_file_path(name: &str)` anexa a extensão `.json` se ausente e concatena ao diretório de otimizações.
- Em `core/src/lib.rs` (linhas 1071–1102), `validate_profile_name(name: &str)` valida o nome contra strings vazias, caracteres ilegais (`\ / : * ? " < > |`) e dispositivos reservados do Windows (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`).
- Em `config/src/lib.rs` (linhas 87–109), `CustomOptimizationConfig::load_from_file` e `CustomOptimizationConfig::save_to_file` utilizam `serde_json::to_string_pretty` e `std::fs::write` / `std::fs::read_to_string`, tratando BOM (`\u{feff}`).
- Em `ui/src/main.rs` (linhas 5–22), `scan_presets()` converte a lista retornada por `counterrush_core::list_optimization_configs()` em um `Vec<PresetModel>` contendo `name` e `tag` ("HARDWARE ADAPTATIVO" para `AutoDetect`, "CUSTOMIZADO / MANUAL" para `custom_optimization` e "PERFIL DO USUÁRIO" para os demais).

### 1.2 Verificação de Métodos Existentes de Exclusão
- A busca por `remove_file` ou `delete` nos crates `config`, `core` e `ui` retornou **0 resultados**.
- Nenhuma função ou método de exclusão ou deleção de arquivo existe no repositório.

### 1.3 Registro de Callbacks e Gerenciamento de Estado no Slint
- Em `ui/main.slint` (linhas 66–75), os callbacks da interface são declarados no corpo de `MainWindow`:
  ```slint
  callback launch-cs2-clicked();
  callback card-action-clicked(string);
  callback tweak-toggled(string, bool);
  callback bulk-tweaks-action(string);
  callback auto-optimize-clicked();
  callback preset-selected(string);
  callback create-profile-confirmed(NewProfileData);
  callback save-launch-args-clicked(string);
  ```
- Em `ui/src/main.rs`, os callbacks são registrados através dos métodos gerados `on_<nome_em_snake_case>`:
  - Linha 522: `main_window.on_preset_selected(move |preset_name| { ... })`
  - Linha 561: `main_window.on_create_profile_confirmed(move |data| { ... })`
- Em `ui/src/main.rs` (linhas 88–142), o estado ativo em memória do backend é armazenado em `active_config_path: Arc<RwLock<PathBuf>>`.
- Em `ui/src/main.rs` (linhas 93–142), há uma thread contínua que roda a cada 500ms e checa a cada 1.5s (`if loop_count % 3 == 0`) se arquivos foram adicionados ou excluídos na pasta `config/optimzations`, atualizando a interface Slint via `slint::invoke_from_event_loop`.
- Em `ui/src/main.rs` (linhas 31–50), o estado neutro/desligado (quando não há perfis salvos) é configurado definindo `active_preset` como `""`, limpando `launch_args_text`, desmarcando `is_editing_launch_args` e zerando todos os 13 booleanos de tweaks (`set_cpu_tweak_active(false)`, etc.).

---

## 2. Cadeia Lógica (Logic Chain)

1. **Localização e formato de persistência:**
   - Com base na observação 1.1, todos os perfis de otimização residem em `config/optimzations/*.json`. O caminho canônico é sempre resolvido por `counterrush_core::resolve_optimization_file_path(name)`.

2. **Inexistência de função de exclusão e conformidade arquitetural (Clean Architecture / SOLID):**
   - Com base na observação 1.2, não há API de remoção de perfil.
   - Pelo Princípio da Responsabilidade Única (SRP) e Arquitetura Limpa:
     - O crate `config` lida exclusivamente com serialização/desserialização da struct `CustomOptimizationConfig` e sua integridade de campos.
     - O crate `core` gerencia as entidades e casos de uso de perfis (resolução de diretório, validação de regras de nomenclatura do Windows e listagem).
     - Portanto, a função de exclusão de perfis deve ser implementada em `core/src/lib.rs`, onde já residem `resolve_optimization_file_path`, `validate_profile_name` e `list_optimization_configs`.
   - A assinatura recomendada no `core/src/lib.rs` é:
     ```rust
     pub fn delete_optimization_profile(name: &str) -> Result<(), CoreError>
     ```
     Acompanhada da variante parametrizável por diretório para facilitar testes unitários com diretórios temporários (`tempfile`):
     ```rust
     pub fn delete_optimization_profile_in_dir<P: AsRef<Path>>(dir: P, name: &str) -> Result<(), CoreError>
     ```
   - O fluxo dessa função deve:
     1. Validar o nome através de `validate_profile_name(name)` para mitigar ataques de Path Traversal (evitando exclusão de arquivos fora do diretório de perfis, ex: `../../hardware/Hardware.json`).
     2. Resolver o caminho absoluto/relativo no diretório de perfis (`resolve_optimizations_dir().join(format!("{clean_name}.json"))`).
     3. Verificar a existência do arquivo com `path.exists()` (retornando erro caso o arquivo não exista).
     4. Chamar `std::fs::remove_file(path)` mapeando erros para `CoreError::Io`.

3. **Bridge Rust <-> Slint e Callback de Exclusão:**
   - Com base na observação 1.3, o Slint deve expor um callback para confirmação de exclusão em `ui/main.slint`:
     ```slint
     callback delete-profile-confirmed(string);
     ```
   - No backend `ui/src/main.rs`, ele deve ser registrado como:
     ```rust
     let window_handle = main_window.as_weak();
     let delete_config_ref = active_config_path.clone();
     main_window.on_delete_profile_confirmed(move |preset_name| {
         // Execução da exclusão no core, atualização da lista de perfis e sincronização de estado
     });
     ```

4. **Impactos em Arquivos, Memória e Caches:**
   - **Sistema de Arquivos:** O arquivo `.json` selecionado é excluído permanentemente de `config/optimzations/`. A pasta permanece preservada.
   - **Memória do Rust (`active_config_path: Arc<RwLock<PathBuf>>`):** Se o perfil ativo no momento for o perfil excluído, o `active_config_path` apontará para um caminho inexistente. Consequentemente:
     - Se restarem outros perfis: deve-se selecionar o primeiro perfil remanescente (`updated_presets[0]`), carregar suas configurações e gravar o novo `PathBuf` em `active_config_path.write()`.
     - Se o perfil excluído for o **único existente** (lista agora vazia): `active_config_path` deve receber `PathBuf::new()`, e a interface deve ser resetada aos padrões neutros idênticos às linhas 31–50 de `ui/src/main.rs` (switches desativados, `active_preset` em `""`, sem launch args).
   - **Interface Slint:**
     - Quando `presets.length == 0`, a interface reage automaticamente ativando o bloco `if root.presets.length == 0` em `ui/main.slint:830`, que renderiza exclusivamente os botões "OTIMIZAR AUTOMATICAMENTE" e "Criar Otimização +" e oculta a barra de parâmetros de inicialização (`if root.active-preset != ""`).

---

## 3. Ressalvas e Limitações (Caveats)

- **Permissões de Sistema de Arquivos no Windows:** Caso o arquivo JSON esteja com permissão de somente leitura ou bloqueado por outro processo (ex: antivírus com lock de arquivo), `std::fs::remove_file` retornará erro de I/O. A mensagem de erro deve ser capturada e exibida na HUD status bar (`ui.set_status_message`).
- **Pasta `optimzations` sem a letra 'i':** A pasta física e os caminhos literais no código chamam-se `optimzations`. Qualquer novo código deve estritamente usar `resolve_optimizations_dir()` e evitar hardcoding incorreto de `optimizations`.

---

## 4. Conclusão

1. **Persistência:** Perfis são salvos em `config/optimzations/*.json`, listados por `counterrush_core::list_optimization_configs()` e convertidos para UI em `scan_presets()`.
2. **Método de Exclusão:** Não existe atualmente. Deve ser criado no crate `core` (`core/src/lib.rs`) sob o nome `delete_optimization_profile(name: &str) -> Result<(), CoreError>` e `delete_optimization_profile_in_dir`, respeitando SRP, Clean Architecture e validação de segurança via `validate_profile_name`.
3. **Bridge Slint:** Registrado em `ui/src/main.rs` via `main_window.on_delete_profile_confirmed(move |preset_name| { ... })`, acionado após a confirmação no modal Slint.
4. **Impacto e Sincronização:** É mandatório atualizar o `active_config_path` (`Arc<RwLock<PathBuf>>`) e os estados do Slint (`presets`, `active_preset`, switches e `launch_args_text`) no próprio callback para que o app não opere com referências mortas e retorne ao estado neutro caso nenhum perfil reste.

---

## 5. Método de Verificação Independente

1. **Inspeção de Código Estática:**
   - Conferir em `core/src/lib.rs` as linhas 1013–1069 (`resolve_optimizations_dir`, `list_optimization_configs`, `resolve_optimization_file_path`).
   - Conferir em `ui/src/main.rs` as linhas 5–22 (`scan_presets`), 31–86 (inicialização com e sem perfis) e 90–142 (thread de sincronização).
2. **Verificação de Compilação e Testes Unitários:**
   - Executar `cargo check --workspace` no diretório raiz do projeto para validar integridade sintática.
   - O implementador poderá adicionar um teste unitário em `core/src/lib.rs` (seguindo os moldes do teste `test_create_and_persist_new_profile` em `core/src/lib.rs:1347`) criando um arquivo em `tempfile::tempdir()`, chamando `delete_optimization_profile_in_dir` e verificando com `assert!(!path.exists())`.
