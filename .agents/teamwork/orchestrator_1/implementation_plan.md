# Plano de Implementação — Atualização da Aba de Otimizações do CounterRush

## Visão Geral
Este plano descreve as modificações necessárias na interface e comportamento da aba de Otimizações do CounterRush para atender aos requisitos R1, R2, R3 e R4, garantindo Clean Architecture, padrão SOLID, zero efeitos colaterais e validação visual via Hot Reload (Slint Viewer).

---

## 1. Motivação e Escopo das Mudanças

| Requisito | Descrição | Motivação |
|---|---|---|
| **R1** | Estado da aba de Otimizações sem arquivos `.json` | Exibir exclusivamente dois botões de ação ("Otimizar Automaticamente" e "+ Criar Otimização") para guiar o usuário na criação do primeiro perfil. |
| **R2** | Remoção do botão inferior de otimização automática | Eliminar a redundância do botão inferior, liberando ~60px verticais e mantendo a barra HUD limpa. |
| **R3** | Reposicionamento do input de parâmetros de inicialização | Posicionar o input de launch args logo abaixo das otimizações ativas (13 switches) e antes da seleção de perfis, criando fluxo de leitura natural. |
| **R4** | Manutenção do fluxo com arquivos existentes | Assegurar que a listagem, seleção e persistência de perfis permaneçam 100% operacionais quando existirem arquivos `.json`. |

---

## 2. Arquivos Impactados

### Arquivos Modificados (Apenas 2 arquivos de UI):
1. `ui/components/add_preset_button.slint`
   - **Mudança:** Atualizar o texto exibido quando `root.is-empty` for `true` de `"NOVO PERFIL"` para `"Criar Otimização"`. Ajustar a largura para 210px quando vazio para comportar a nova legenda com folga.
   - **Motivação:** Cumprir o requisito R1 preservando o ícone `+` e o disparo do modal de criação.

2. `ui/main.slint`
   - **Mudança 1 (R2):** Remoção do botão inferior `TacticalButton` "OTIMIZAR AUTOMATICAMENTE" (linhas 896-917), preservando intacta a Barra de Status HUD (linhas 920-941).
   - **Mudança 2 (R3):** Transferência do contêiner do input de parâmetros de inicialização (`if root.active-preset != "" : HorizontalLayout`) das linhas 770-888 para a linha 707 (diretamente abaixo da grade de 13 switches e antes da seção de perfis).
   - **Mudança 3 (R1 & R4):** Na seção de perfis/ações, implementar a ramificação condicional reativa:
     - `if root.presets.length == 0`: exibir contêiner com exclusivamente os 2 botões de ação ("Otimizar Automaticamente" com badge e clique chamando `root.auto-optimize-clicked()`, e `AddPresetButton { is-empty: true; }` abrindo o modal).
     - `if root.presets.length > 0`: exibir o separador `"PERFIS DE OTIMIZAÇÃO"`, a lista horizontal de `PresetButton`s e o `AddPresetButton` compacto.

### Arquivos NÃO Modificados (Zero Alteração em Rust):
- `ui/src/main.rs`: Os callbacks `on_auto_optimize_clicked`, `on_save_launch_args_clicked` e `on_create_profile_confirmed` já implementam 100% da lógica necessária.
- `core/` e `config/`: Nenhuma alteração necessária.

---

## 3. Pontos de Atenção (Avisos de Risco e Segurança)

> ⚠️ **ATENÇÃO — Comandos Destrutivos:** Nenhum comando destrutivo será executado. Nenhum arquivo `.json` de usuário será deletado. Para simular o estado vazio em testes, os arquivos serão apenas renomeados temporariamente para backup e restaurados em seguida.
>
> ⚠️ **ATENÇÃO — Git e Branches:** Nenhum comando git de escrita ou push para branch remota será executado.
>
> ⚠️ **ATENÇÃO — Altura Fixa da Janela (620px):** O reposicionamento do input foi minuciosamente calculado pelos Explorers: a soma de todos os elementos na aba é de 409px, deixando 177px de margem livre na janela de 620px, garantindo que não haverá corte visual ou overflow.

---

## 4. Etapas de Execução

1. **Etapa 1:** Despachar Worker para aplicar alterações em `ui/components/add_preset_button.slint`.
2. **Etapa 2:** Despachar Worker para aplicar alterações em `ui/main.slint` (remoção do botão inferior, reposicionamento do input de launch args e renderização exclusiva dos 2 botões no estado vazio).
3. **Etapa 3:** Validação com Reviewers e Challengers para verificar compilação (`cargo check -p counterrush-ui`) e integridade.
4. **Etapa 4:** Verificação de integridade forense com Forensic Auditor (`teamwork_auditor`).
5. **Etapa 5:** Validação visual com Slint Viewer / Hot Reload (`preview.cmd`) nos dois estados (sem perfis e com perfis).
