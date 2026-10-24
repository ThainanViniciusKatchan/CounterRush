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

## 6.1. Inicialização com Otimizações Personalizadas (JSON)

Inicia o CS2 com parâmetros da Steam e otimizações de sistema customizados a partir de um arquivo config/custom_optimization.json. Caso o arquivo não exista, ele é criado automaticamente com base no perfil de hardware detectado (Econômico, Intermediário ou Potente).

`powershell
cargo run -p core --example test_custom_launch
`

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




---

## 15. Ativação do Modo MSI (Message Signaled-Based Interrupts)

Ativa o modo de interrupção baseado em mensagens (MSI) diretamente pelo barramento PCIe, eliminando o compartilhamento de linhas IRQ (INTx) e reduzindo a latência DPC/ISR do sistema:
- **Placa de Vídeo (Display):** Ativa o MSI (`MSISupported = 1`) com Prioridade Alta (`DevicePriority = 3`) para que o agendador de interrupções priorize a renderização de frames do CS2.
- **Placas de Rede (Ethernet / Wi-Fi):** Ativa o MSI (`MSISupported = 1`) para mitigar atrasos de buffer de rede e micro-congelamentos (*packet jitter*).
- **Controladoras Host USB (PCI):** Ativa o MSI (`MSISupported = 1`) para periféricos de alto polling rate (mouses de 1000Hz a 8000Hz e teclados mecânicos).

> [!IMPORTANT]
> A manipulação das chaves em `HKLM:\SYSTEM\CurrentControlSet\Enum\PCI\...\Device Parameters` exige privilégios de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 15.1. Ativar Modo MSI em todos os dispositivos elegíveis (GPU com Prioridade Alta + Rede + USB)
cargo run -p tweaks --example test_msi -- --enable

# 15.2. Ativar Modo MSI apenas na GPU (com Prioridade Alta: DevicePriority = 3)
cargo run -p tweaks --example test_msi -- --enable-gpu

# 15.3. Ativar Modo MSI apenas na Placa de Rede
cargo run -p tweaks --example test_msi -- --enable-network

# 15.4. Ativar Modo MSI apenas nas Controladoras Host USB
cargo run -p tweaks --example test_msi -- --enable-usb

# 15.5. Rollback completo (remove MessageSignaledInterruptProperties e Affinity Policy de todos os dispositivos)
cargo run -p tweaks --example test_msi -- --rollback

# 15.6. Rollback apenas na GPU
cargo run -p tweaks --example test_msi -- --rollback-gpu

# 15.7. Rollback apenas na Placa de Rede
cargo run -p tweaks --example test_msi -- --rollback-network

# 15.8. Rollback apenas nas Controladoras Host USB
cargo run -p tweaks --example test_msi -- --rollback-usb

# 15.9. Consultar status detalhado de todos os dispositivos PCI encontrados
cargo run -p tweaks --example test_msi -- --status
```

---

## 16. Desativação da Economia de Energia PCIe e USB

Desativa o gerenciamento de economia de energia no barramento PCI Express (**Link State Power Management - ASPM**), a **Suspensão Seletiva USB** no plano de energia e a permissão do Windows para desligar **Hubs USB** no Gerenciador de Dispositivos (WMI). Isso evita que o hardware entre em estados de baixo consumo (L0s/L1 e suspensão de portas), eliminando *micro-stutters*, picos de *frametime* e latência de entrada (*input lag*) em mouses e teclados no CS2.

> [!IMPORTANT]
> A alteração do plano de energia e a manipulação das instâncias CIM no namespace `root\wmi` exigem permissões de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 16.1. Desativar todas as economias de energia (PCIe ASPM + Suspensão Seletiva USB + Hubs USB)
cargo run -p tweaks --example test_power_saving -- --disable

# 16.2. Desativar apenas a economia PCIe (Link State Power Management - ASPM)
cargo run -p tweaks --example test_power_saving -- --disable-pcie

# 16.3. Desativar apenas a Suspensão Seletiva USB no plano de energia
cargo run -p tweaks --example test_power_saving -- --disable-usb-suspend

# 16.4. Desativar apenas a economia de energia em Hubs USB (Gerenciador de Dispositivos)
cargo run -p tweaks --example test_power_saving -- --disable-usb-hubs

# 16.5. Rollback completo (restaura todas as configurações para o padrão de fábrica do Windows)
cargo run -p tweaks --example test_power_saving -- --rollback

# 16.6. Rollback apenas no PCIe (restaura para Economia Moderada de Energia - padrão Windows)
cargo run -p tweaks --example test_power_saving -- --rollback-pcie

# 16.7. Rollback apenas na Suspensão Seletiva USB (restaura para Habilitado)
cargo run -p tweaks --example test_power_saving -- --rollback-usb-suspend

# 16.8. Rollback apenas nos Hubs USB (devolve permissão de economia de energia ao Windows)
cargo run -p tweaks --example test_power_saving -- --rollback-usb-hubs

# 16.9. Consultar status atual detalhado de todas as configurações de energia
cargo run -p tweaks --example test_power_saving -- --status
```

---

## 17. Otimizações na Render Queue / Flip Queue Configuration (AMD & NVIDIA)

Configura a fila de renderização de quadros (*pre-rendered frames*) para reduzir a latência de clique e resposta de mira (*input-to-display latency*) no Counter-Strike 2:
- **AMD:** Define `FlipQueueSize = 1` (em formato binário `[0x31, 0x00]`) sob `HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\000*`, limitando o driver a apenas 1 quadro pré-renderizado.
- **NVIDIA:** Executa `nvidia-smi -pm 1` (Modo de Persistência), mantendo o driver da GPU ativo no kernel para eliminar latências de transição de estado de energia.

> [!IMPORTANT]
> A gravação na chave da classe de vídeo no Registro e a execução de comandos `nvidia-smi` exigem permissões de Administrador. Execute o terminal PowerShell como Administrador.

```powershell
# 17.1. Aplicar otimização em todas as placas de vídeo detectadas (AMD e/ou NVIDIA)
cargo run -p tweaks --example test_render_queue -- --apply

# 17.2. Aplicar FlipQueueSize = 1 apenas para placas AMD
cargo run -p tweaks --example test_render_queue -- --apply-amd

# 17.3. Aplicar Modo de Persistência apenas para placas NVIDIA (nvidia-smi -pm 1)
cargo run -p tweaks --example test_render_queue -- --apply-nvidia

# 17.4. Rollback completo (restaura o padrão do fabricante em todas as GPUs)
cargo run -p tweaks --example test_render_queue -- --rollback

# 17.5. Rollback apenas para placas AMD (remove FlipQueueSize do Registro)
cargo run -p tweaks --example test_render_queue -- --rollback-amd

# 17.6. Rollback apenas para placas NVIDIA (nvidia-smi -pm 0)
cargo run -p tweaks --example test_render_queue -- --rollback-nvidia

# 17.7. Consultar status atual da fila de renderização em todas as placas
cargo run -p tweaks --example test_render_queue -- --status
```
