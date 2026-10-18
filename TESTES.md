# Comandos de Teste do CounterRush

Guia rápido com todos os comandos de teste manual executáveis via terminal PowerShell.

---

## 1. Verificação e Compilação Geral

Comandos para checagem rápida de integridade e compilação de todo o projeto:

```powershell
# Verifica sintaxe e tipagem de todos os crates e exemplos em segundos
cargo check --workspace --examples

# Compila todos os crates e gera todos os binários executáveis de teste
cargo build --workspace --examples
```

---

## 2. Detecção e Classificação de Hardware

Coleta informações detalhadas de CPU, GPU e memória RAM e classifica o computador nos perfis **Econômico**, **Intermediário** ou **Potente** com base nos requisitos do CS2.

```powershell
cargo run -p core --example test_hardware_detection
```

---

## 3. Inicialização do CS2 por Perfil de Hardware

Detecta o hardware do computador, determina a categoria ideal e inicia o Counter-Strike 2 pela Steam com os parâmetros de otimização exatos definidos no `Parameters.md`.

```powershell
cargo run -p core --example test_launch_by_category
```

---

## 4. Launcher Básico do CS2

Testa a abertura direta do jogo através do protocolo oficial da Steam (`steam://rungameid/730`), contando com fallback automático para o executável `steam.exe`.

```powershell
cargo run -p windows_api --example test_launcher
```

---

## 5. Tweaks de Processo (Prioridade e Afinidade)

Inicia o Counter-Strike 2 via Steam automaticamente (caso ainda não esteja em execução), aguarda a detecção do processo `cs2.exe` e aplica os ajustes de processo diretamente, podendo executar cada um separadamente ou ambos juntos:

```powershell
# 5.1. Abrir jogo (se fechado) e aplicar apenas a Prioridade RealTime
cargo run -p tweaks --example test_process_tweaks -- --priority

# 5.2. Abrir jogo (se fechado) e aplicar apenas a Afinidade de CPU (sem Core 0 em CPUs > 4 núcleos)
cargo run -p tweaks --example test_process_tweaks -- --affinity

# 5.3. Abrir jogo (se fechado) e aplicar ambos os tweaks em conjunto
cargo run -p tweaks --example test_process_tweaks -- --all
# (ou simplesmente: cargo run -p tweaks --example test_process_tweaks)
```

---

## 6. Fluxo Integrado (Limpeza de RAM + Launcher + Espera + Otimizações)

Executa o fluxo completo do otimizador de ponta a ponta com feedback detalhado em tempo real:
1. Esvazia a Standby List da memória RAM (RAMMap Clone) para liberar páginas de cache antes de abrir o jogo.
2. Aciona a Steam para iniciar o Counter-Strike 2.
3. Aguarda ativamente até 60 segundos para o processo `cs2.exe` ser instanciado.
4. Aplica automaticamente a prioridade RealTime e a afinidade de processador (sem Core 0 em CPUs > 4 núcleos).

> [!NOTE]
> Para que a limpeza da Standby List e as prioridades do processo sejam aplicadas com permissões completas, execute o terminal PowerShell como Administrador.

```powershell
cargo run -p core --example test_launch_and_optimize
```

---

## 7. Gerenciamento de Aceleração de GPU por Aplicativo

Permite desativar a aceleração de hardware por GPU de aplicativos em segundo plano para liberar VRAM e recursos gráficos para o CS2, incluindo o **rollback opcional** para reativar a qualquer momento.

### 7.1. Consulta de Status
Lista o estado atual da aceleração (Ativada ou Desativada) em cada aplicativo:

```powershell
cargo run -p tweaks --example test_gpu_accel
```

### 7.2. Discord
```powershell
# Desativar aceleração de GPU no Discord
cargo run -p tweaks --example test_gpu_accel -- --disable-discord

# Reativar aceleração de GPU no Discord (Rollback)
cargo run -p tweaks --example test_gpu_accel -- --enable-discord
```

### 7.3. Steam
```powershell
# Desativar aceleração de GPU na Steam (Interface Web)
cargo run -p tweaks --example test_gpu_accel -- --disable-steam

# Reativar aceleração de GPU na Steam (Rollback)
cargo run -p tweaks --example test_gpu_accel -- --enable-steam
```

### 7.4. Navegadores Chromium (Chrome, Edge, Brave)
```powershell
# Desativar aceleração de GPU nos navegadores Chromium
cargo run -p tweaks --example test_gpu_accel -- --disable-chromium

# Reativar aceleração de GPU nos navegadores Chromium (Rollback)
cargo run -p tweaks --example test_gpu_accel -- --enable-chromium
```

### 7.5. Firefox
```powershell
# Desativar aceleração de GPU no Firefox
cargo run -p tweaks --example test_gpu_accel -- --disable-firefox

# Reativar aceleração de GPU no Firefox (Rollback)
cargo run -p tweaks --example test_gpu_accel -- --enable-firefox
```

### 7.6. Operações em Lote (Todos os Aplicativos)
```powershell
# Desativar aceleração de GPU em todos os aplicativos de uma vez
cargo run -p tweaks --example test_gpu_accel -- --disable-all

# Reativar aceleração de GPU em todos os aplicativos de uma vez (Rollback Geral)
cargo run -p tweaks --example test_gpu_accel -- --enable-all
```

---

## 8. Limpeza da Standby List (RAMMap Clone)

Esvazia a lista de memória em standby do Windows através da Native API (`NtSetSystemInformation` com classe `80` e comando `4`), liberando memória RAM ocupada por páginas de cache antes de abrir o jogo para evitar micro-travamentos e paginação pelo kernel.

> [!IMPORTANT]
> A limpeza da Standby List exige privilégios de Administrador (`SeProfileSingleProcessPrivilege`). Execute o terminal PowerShell como Administrador para testar.

```powershell
cargo run -p tweaks --example test_empty_standby
```

---

## 9. Otimização de DPI e Modo Janela do CS2

Configura flags de compatibilidade do Windows (`~ DISABLEDXMAXIMIZEDWINDOWEDMODE PERPROCESSSYSTEMDPIFORCEOFF HIGHDPIAWARE`) na chave de registro do CS2 para mitigar latência de entrada (input lag), desativar otimizações de janela do DirectX e desligar o escalonamento incorreto de DPI.

```powershell
# 9.1. Aplicar a otimização de DPI no Registro e disparar o CS2 via Steam
cargo run -p tweaks --example test_dpi_optimization -- --apply

# 9.2. Executar o Fallback (Rollback) para remover a chave do Registro e restaurar o padrão
cargo run -p tweaks --example test_dpi_optimization -- --rollback

# 9.3. Consultar o status atual da chave no Registro
cargo run -p tweaks --example test_dpi_optimization -- --status
```

---

## 10. Desativação de Compactação de Memória (MMAgent)

Controla o recurso de compactação de páginas de memória do Windows (`Disable-MMAgent -mc` e `Enable-MMAgent -mc`), reduzindo latência de CPU e micro-travamentos em computadores com mais de 16 GB de RAM (> 16 GB).

> [!IMPORTANT]
> A desativação só é permitida se o computador possuir mais de 16 GB de RAM. Os comandos `MMAgent` exigem privilégios de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 10.1. Desativar compactação de memória (com verificação prévia de status e trava > 16GB)
cargo run -p tweaks --example test_memory_compression -- --disable

# 10.2. Reativar compactação de memória (Rollback com Enable-MMAgent -mc)
cargo run -p tweaks --example test_memory_compression -- --enable

# 10.3. Consultar status atual no MMAgent e RAM detectada
cargo run -p tweaks --example test_memory_compression -- --status
```

---

## 11. Resolução de Temporizador do Sistema (0.5ms)

Gerencia a resolução do temporizador do Windows cravando-a em 0.5 ms através de chamadas nativas `NtSetTimerResolution`, habilitando a diretiva global `GlobalTimerResolutionRequests` no Registro e medindo a precisão das pausas com `NtDelayExecution`.

> [!IMPORTANT]
> A configuração do Registro e os ajustes de timer exigem privilégios de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 11.1. Aplicar timer de 0.5ms (testa comando 1 Timer05; se não reduzir, usa fallback KernelTimer) e testa 10 pausas
cargo run -p tweaks --example test_timer_resolution -- --apply

# 11.2. Apenas testar precisão do timer atual (10 pausas de 0.5ms com NtDelayExecution)
cargo run -p tweaks --example test_timer_resolution -- --test

# 11.3. Rollback completo (restaura timer com NtSetTimerResolution Set=false e remove chave do Registro)
cargo run -p tweaks --example test_timer_resolution -- --rollback

# 11.4. Rollback apenas do timer
cargo run -p tweaks --example test_timer_resolution -- --rollback-timer

# 11.5. Rollback apenas do registro
cargo run -p tweaks --example test_timer_resolution -- --rollback-reg

# 11.6. Consultar status do registro e informações do tweak
cargo run -p tweaks --example test_timer_resolution -- --status
```

---

## 12. Otimização MMCSS (Multimedia Class Scheduler Service)

Gerencia a prioridade de rede, CPU, GPU e I/O de disco do Windows no `SystemProfile` e no perfil `Tasks\Games`, adaptando os valores automaticamente ao hardware da máquina (PC Forte vs PC Fraco / Quad-Core).

> [!IMPORTANT]
> A gravação de chaves sob `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile` exige privilégios de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 12.1. Aplicar automaticamente conforme o hardware detectado (PC Forte vs PC Fraco)
cargo run -p tweaks --example test_mmcss -- --apply

# 12.2. Forçar perfil PC Forte (6+ núcleos / 16GB+ RAM)
cargo run -p tweaks --example test_mmcss -- --apply-forte

# 12.3. Forçar perfil PC Fraco / Quad-Core / 8GB RAM
cargo run -p tweaks --example test_mmcss -- --apply-fraco

# 12.4. Rollback (restaurar todos os valores para o padrão de fábrica do Windows)
cargo run -p tweaks --example test_mmcss -- --rollback

# 12.5. Consultar status atual de cada chave no Registro
cargo run -p tweaks --example test_mmcss -- --status
```

---

## 13. Desativação do Temporizador de Alta Precisão (HPET)

Controla o parâmetro `useplatformclock` do BCD (*Boot Configuration Data*). Desativar o HPET força o Windows a usar os contadores de tempo rápidos da própria CPU (TSC), diminuindo o *input lag* e os micro-stutters no CS2.

> [!IMPORTANT]
> A execução de comandos `bcdedit` exige privilégios de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 13.1. Desativar HPET (bcdedit /set useplatformclock false)
cargo run -p tweaks --example test_hpet -- --disable

# 13.2. Rollback do HPET (bcdedit /deletevalue useplatformclock)
cargo run -p tweaks --example test_hpet -- --rollback

# 13.3. Consultar status atual do useplatformclock no BCD
cargo run -p tweaks --example test_hpet -- --status
```

---

## 14. Desativação do Xbox Game Bar, Modo de Jogo e Capturas (GameDVR)

Desativa as sobreposições em segundo plano do ecossistema Xbox (Modo de Jogo, atalhos da Game Bar, Modo Xbox e gravação em segundo plano GameDVR) para eliminar micro-stutters, liberar VRAM e evitar sobreposições no CS2.

> [!IMPORTANT]
> A remoção e gravação da política `AllowGameDVR` sob `HKLM` exige privilégios de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 14.1. Desativar todos os recursos (Modo de Jogo + Game Bar + Modo Xbox + Capturas GameDVR)
cargo run -p tweaks --example test_game_bar -- --disable

# 14.2. Desativar apenas o Modo de Jogo (AutoGameModeEnabled = 0)
cargo run -p tweaks --example test_game_bar -- --disable-game-mode

# 14.3. Desativar apenas os atalhos do Xbox Game Bar (UseNexusForGameBarEnabled = 0)
cargo run -p tweaks --example test_game_bar -- --disable-game-bar

# 14.4. Desativar apenas o Modo Xbox (GamingHomeApp = "")
cargo run -p tweaks --example test_game_bar -- --disable-xbox-mode

# 14.5. Desativar apenas as Capturas e GameDVR (GameDVR_Enabled = 0, AppCaptureEnabled = 0, AllowGameDVR = 0)
cargo run -p tweaks --example test_game_bar -- --disable-captures

# 14.6. Rollback completo (restaurar todos os valores para o padrão de fábrica do Windows)
cargo run -p tweaks --example test_game_bar -- --rollback

# 14.7. Consultar status atual de cada chave no Registro
cargo run -p tweaks --example test_game_bar -- --status
```




