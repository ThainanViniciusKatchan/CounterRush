###### Este relatório descreve o status de cada comando executados manualmente.<br/>

Ele servira de comparação à execução da função que cria o autodetect, pois em meus testes mesmo após as otimizações de sistema, aplicativos, CPU e GPU que são permanetes serem aplicadas, ao clicar de otimização automaticamente essas otimizações são executadas de forma redundante pois já foram aplicadas, com isso alguns aplicativos como discord, navegadores e a steam são reniciados sempre que o game é aberto, e esse não deveria ser o comportamento, pois as otimizações deveriam funcionar de forma dinâmica, por exemplo, sempre que o usuário iniciar o game deve er verificado o arquivo selecionado de otimização, realizada uma varedura de status em todas as otimizações permanentes e após isso as otimizações já aplicadas devem ser ignoradas e marcadas no arquivo como false, segue a baixo os comandos utilizados e os retornos.



### Comandos e retornos:

###### A lista funciona da seguinte forma, primeiro o comando logo a baixo o retorno.

- cargo run -p core --example test_autodetect
  

- cargo run -p tweaks --example test_gpu_accel

- 1. Desativar Aceleração de GPU no Discord: **Applied**
  2. Desativar Aceleração de GPU na Steam: **Applied**
  3. Desativar Aceleração de GPU nos Navegadores Chromium: **Applied**
  4. Desativar Aceleração de GPU no Firefox: **Applied**



- cargo run -p tweaks --example test_dpi_optimization -- --status

- === Teste: Otimização de DPI e Modo Janela do CS2 ===
  Nome:        Otimização de DPI e Modo Janela do CS2
  ID:          cs2_dpi_optimization
  Descrição:   Aplica flags de compatibilidade (DISABLEDXMAXIMIZEDWINDOWEDMODE, PERPROCESSSYSTEMDPIFORCEOFF, HIGHDPIAWARE) para mitigar input lag e problemas de escala no CS2.

  **Status atual no Registro: Applied**



- cargo run -p tweaks --example test_memory_compression -- --status

- === Teste: Compactação de Memória do Windows (MMAgent) ===
  Nome:        Desativação de Compactação de Memória (MMAgent)
  ID:          memory_compression
  Descrição:   Desativa a compactação de memória do Windows (Disable-MMAgent -mc) em PCs com mais de 16GB de RAM para reduzir latência de CPU e micro-stutters.
  Requer Admin: true
  RAM Total Detectada: 31.93 GB
    [OK] Memória RAM > 16 GB (31.93 GB). Elegível para desativação da compactação.

  Consultando status atual com Get-MMAgent...
  Status atual: DESATIVADA (Otimizado para >16GB - MemoryCompression = False)



- cargo run -p tweaks --example test_mmcss -- --status

- === Teste: Otimizações de MMCSS (Multimedia Class Scheduler Service) ===
  Nome:         Otimização MMCSS (Prioridade de Rede, CPU, GPU e I/O por Perfil de Hardware)
  ID:           mmcss
  Descrição:    Configura o Multimedia Class Scheduler Service (MMCSS) no Registro adaptando parâmetros para PC Forte (6+ núcleos / 16GB+ RAM) ou PC Fraco (Quad-Core / 8GB RAM).
  Requer Admin: true
  Hardware Detectado: 6 núcleos físicos / 31.93 GB RAM
  Perfil Recomendado para este PC: PC Forte (6+ núcleos / 16GB+ RAM)

  Valores Atuais no Registro:
  ----------------------------------------------------------------------------------

  SystemResponsiveness       : 10 (0xa)
    NetworkThrottlingIndex     : 4294967295 (0xffffffff)
    GPU Priority               : 8 (0x8)
    Priority                   : 6 (0x6)
    Scheduling Category        : High
    SFIO Priority              : High
  ----------------------------------------------------------------------------------

  Conformidade: [PERFIL PC FORTE APLICADO COM SUCESSO]



- cargo run -p tweaks --example test_hpet -- --status

- === Teste: Temporizador de Eventos de Alta Precisão (HPET) ===
  Nome:         Desativação do Temporizador de Alta Precisão (HPET)
  ID:           hpet
  Descrição:    Desativa useplatformclock no BCD para que o Windows utilize os contadores de tempo rápidos da CPU (TSC), reduzindo input lag e stutters no CS2.
  Requer Admin: true

  Consultando status do HPET no BCD...
  Status Atual: Padrão do Windows (useplatformclock não definido no BCD)

  [PADRÃO] useplatformclock não está presente no BCD. O Windows gerencia o relógio nativamente.



- cargo run -p tweaks --example test_game_bar -- --status

- === Teste: Desativação do Xbox Game Bar, Modo de Jogo e Capturas (GameDVR) ===
  Nome:         Desativação do Xbox Game Bar, Modo de Jogo e Capturas (GameDVR)
  ID:           game_bar
  Descrição:    Desativa o Modo de Jogo, atalhos do Xbox Game Bar, Modo Xbox e gravação em segundo plano (GameDVR) no Registro.
  Requer Admin: true

  Consultando status atual no Registro...

  Valores Atuais no Registro:
  ----------------------------------------------------------------------------------

  AutoGameModeEnabled (Game Mode)      : 0 (0x0) [DESATIVADO (Otimizado)]
    UseNexusForGameBarEnabled (Game Bar) : 0 (0x0) [DESATIVADO (Otimizado)]
    GamingHomeApp (Modo Xbox)            : "" (Vazio) [DESATIVADO (Otimizado)]
    GameDVR_Enabled (GameConfigStore)    : 0 (0x0) [DESATIVADO (Otimizado)]
    AppCaptureEnabled (GameDVR)          : 0 (0x0) [DESATIVADO (Otimizado)]
    AllowGameDVR (Política HKLM)         : 0 (0x0) [BLOQUEADO (Otimizado)]
  ----------------------------------------------------------------------------------

  Conformidade Geral: [TODOS OS RECURSOS XBOX/GAMEDVR ESTÃO DESATIVADOS (100% OTIMIZADO)]



- cargo run -p tweaks --example test_msi -- --status

- === Teste: Modo MSI (Message Signaled-Based Interrupts) ===
  Nome:         Ativação do Modo MSI (Message Signaled-Based Interrupts)
  ID:           msi_mode
  Descrição:    Ativa o Modo MSI na GPU (Prioridade Alta), Placa de Rede e Controladoras USB para eliminar disputas de interrupção (IRQs) e reduzir latência no CS2.
  Requer Admin: true

  Consultando status atual do Modo MSI no Registro...

  Dispositivos PCI Detectados:
  ---------------------------------------------------------------------------------------------------------

  Categoria | Nome do Dispositivo                              | Modo MSI         | Prioridade GPU
  ---------------------------------------------------------------------------------------------------------

  GPU       | AMD Radeon RX 7600                               | ATIVO (1)        | ALTA (3)       
    Network   | Realtek PCIe GbE Family Controller               | ATIVO (1)        | -
    USB       | AMD USB 3.10 eXtensible Host Controller - 1.1... | ATIVO (1)        | -
    USB       | AMD USB 3.10 eXtensible Host Controller - 1.1... | ATIVO (1)        | -
  ---------------------------------------------------------------------------------------------------------

  Conformidade Geral: [TODOS OS DISPOSITIVOS ESTÃO COM MODO MSI ATIVO E OTIMIZADO]



- cargo run -p tweaks --example test_power_saving -- --status

- ================================================================================
          STATUS DA ECONOMIA DE ENERGIA (PCIe & USB) - COUNTER RUSH
  ================================================================================

  1. PCI Express (Link State Power Management - ASPM):

     - Na Tomada (AC): 0 (Desligado - Otimizado para Jogos)
     - Na Bateria (DC): 0 (Desligado - Otimizado para Jogos)
     - Status: [OK] OTIMIZADO (ASPM Desligado)

  2. Suspensão Seletiva USB (Plano de Energia):

     - Na Tomada (AC): 0 (Desabilitado - Otimizado para Jogos)
     - Na Bateria (DC): 0 (Desabilitado - Otimizado para Jogos)
     - Status: [OK] OTIMIZADO (Suspensão Desabilitada)

  3. Hubs e Controladoras USB (Gerenciador de Dispositivos / WMI):
     Dispositivo (InstanceName)                         | Economia de Energia      
     ---------------------------------------------------+--------------------------
     USB\ROOT_HUB30\5&201a262c&0&0_0                    | [OK] Desativada (Otimizado)
     USB\ROOT_HUB30\5&23f8e3f5&0&0_0                    | [OK] Desativada (Otimizado)
     USB\VID_048D&PID_5702\6&28cf390b&0&14_0            | [OK] Desativada (Otimizado)
     USB\VID_2DC8&PID_3109\E417D841E133_0               | [OK] Desativada (Otimizado)
     USB\VID_05E3&PID_0610\6&28cf390b&0&9_0             | [OK] Desativada (Otimizado)
     USB\VID_05E3&PID_0610\6&2838f1dc&0&4_0             | [OK] Desativada (Otimizado)
     USB\VID_03F0&PID_0A8E&MI_03\7&39c2d565&0&0003_0    | [OK] Desativada (Otimizado)
     USB\VID_1D57&PID_FA60&MI_00\7&3161d4c4&0&0000_0    | [OK] Desativada (Otimizado)
     USB\VID_1D57&PID_FA60&MI_01\7&3161d4c4&0&0001_0    | [OK] Desativada (Otimizado)
     USB\VID_1D57&PID_FA60&MI_02\7&3161d4c4&0&0002_0    | [OK] Desativada (Otimizado)
     USB\VID_1D57&PID_FA60&MI_03\7&3161d4c4&0&0003_0    | [OK] Desativada (Otimizado)
     USB\VID_14CD&PID_1212\121220160204_0               | [OK] Desativada (Otimizado)
     USB\VID_258A&PID_0016&MI_00\8&43cc1f4&0&0000_0     | [OK] Desativada (Otimizado)
     USB\VID_258A&PID_0016&MI_01\8&43cc1f4&0&0001_0     | [OK] Desativada (Otimizado)

     - Status Geral USB: [OK] OTIMIZADO (Economia desmarcada em todos)

  --------------------------------------------------------------------------------

  Conformidade Geral do Sistema: TODAS AS OTIMIZAÇÕES DE ENERGIA ESTÃO ATIVADAS! (Latência mínima garantida)
  --------------------------------------------------------------------------------



- cargo run -p tweaks --example test_render_queue -- --status

- ================================================================================
       STATUS DA FILA DE QUADROS / RENDER QUEUE (FLIP QUEUE) - COUNTER RUSH
  ================================================================================

  1. Placas de Vídeo AMD (FlipQueueSize no Registro):
     Chave    | Modelo (DriverDesc)            | FlipQueueSize    | Bytes        | Status
     ---------+--------------------------------+------------------+--------------+---------------------
     0000     | AMD Radeon RX 7600             | 1                | 49 0         | [OK] OTIMIZADO (1)  

     - Conformidade AMD: [OK] TODAS AS GPUS AMD ESTÃO COM FlipQueueSize = 1

  2. Placas de Vídeo NVIDIA (Modo de Persistência / nvidia-smi):
     Nenhuma GPU NVIDIA ativa ou utilitário nvidia-smi disponível no sistema.

  --------------------------------------------------------------------------------

  Conformidade Geral do Sistema: FILA DE QUADROS OTIMIZADA PARA RESPOSTA INSTANTÂNEA NO CS2!
  --------------------------------------------------------------------------------



Da para notar que todas as otimizações permanentes, ou seja, aquelas que não precisam serem executadas sempre que o game iniciar ou o computador é desligado\reiniciado, já estão ativadas, porém, quando realizada a otimização automática, elas recebem o valor true mas deveria ser false, as únicas otimizações que devem ser executadas sempre que o game iniciar são: empty_standby_list, cs2_affinity, cs2_priority (opcional, não deve ser ativada por padrão) e apply_timer_resolution, pois quando o computador reiniciar os valores são resetados e precisam mudar novamente para terem efetividade.





## Saída ao executar o comando de criar o arquivo autodetect.json:

- ================================================================================
       CounterRush — Detecção de Hardware e Geração do AutoDetect.json
  ================================================================================

  [1] Verificando arquivo de cache de hardware: config\hardware\Hardware.json
    -> [CACHE IDENTIFICADO] Arquivo 'Hardware.json' encontrado. Dados carregados do arquivo.

  [2] Especificações de Hardware Utilizadas:

  - Processador (CPU):  AMD Ryzen 5 3600 6-Core Processor
    Núcleos / Threads:  6 físicos / 12 lógicos (Frequência: 3600 MHz)
  - Placa de Vídeo (GPU): AMD Radeon RX 7600
    VRAM Dedicada:      7.98 GB
  - Memória RAM:        31.93 GB
    Velocidade da RAM:  2666 MHz
  - Perfil Classificado: PC Intermediário

  [3] Avaliação de Hardware e Status Atual das Otimizações:
    [OPCIONAL / PADRÃO OFF] Prioridade em Tempo Real do CS2
      ID no JSON:  cs2_priority (false)
      Status SO:   Inativo
      Critério:    Desativada por padrão conforme diretriz do sistema. Mantida como otimização opcional para controle do usuário. | Status SO: Não aplicada.

    [PENDENTE PARA OTIMIZAR] Afinidade de Processador do CS2 (sem Core 0)
      ID no JSON:  cs2_affinity (true)
      Status SO:   Pendente (será aplicado)
      Critério:    Compatível: CPU com 6 núcleos físicos permite isolar o Core 0 sem sobrecarregar as threads do CS2. | Status SO: Otimização de execução (aplicada ao iniciar o CS2).

    [JÁ OTIMIZADO NO SO] Desativação da Compactação de Memória
      ID no JSON:  disable_memory_compression (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Sistema possui 31.93 GB de RAM (> 16 GB). Desativar compactação reduz latência de CPU sem risco de exaustão de memória. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [JÁ OTIMIZADO NO SO] Fila de Quadros Renderizados (Render Queue)
      ID no JSON:  apply_render_queue (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: GPU AMD detectada. FlipQueueSize = 1 suportado no Registro. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [JÁ OTIMIZADO NO SO] Desativar Aceleração de GPU em Apps
      ID no JSON:  disable_gpu_acceleration (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Ativado: Recomendado para perfil PC Intermediário para poupar VRAM dedicada e ciclos de decodificação. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [PENDENTE PARA OTIMIZAR] Limpeza da Standby List (RAM)
      ID no JSON:  empty_standby_list (true)
      Status SO:   Pendente (será aplicado)
      Critério:    Compatível: Universalmente suportado. Libera memória cache antes da inicialização. | Status SO: Otimização de execução (aplicada ao iniciar o CS2).

    [JÁ OTIMIZADO NO SO] Otimização de DPI e Modo Janela
      ID no JSON:  apply_dpi_optimization (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Universalmente suportado. Elimina latência de composição de janelas do DirectX. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [JÁ OTIMIZADO NO SO] Resolução do Temporizador do Sistema (0.5ms)
      ID no JSON:  apply_timer_resolution (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Universalmente suportado. Crava o timer do kernel em 0.5ms para menor input lag. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [JÁ OTIMIZADO NO SO] Otimização do Agendador Multimídia (MMCSS)
      ID no JSON:  apply_mmcss (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Universalmente suportado com calibração automática conforme núcleos e RAM. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [PENDENTE PARA OTIMIZAR] Desativação do HPET
      ID no JSON:  disable_hpet (true)
      Status SO:   Pendente (será aplicado)
      Critério:    Compatível: Universalmente suportado para uso de contadores TSC de alta frequência da CPU. | Status SO: Pendente (será aplicado na otimização automática).

    [JÁ OTIMIZADO NO SO] Desativação do Xbox Game Bar e GameDVR
      ID no JSON:  disable_game_bar (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Universalmente suportado. Elimina overlays e gravação em segundo plano. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

    [JÁ OTIMIZADO NO SO] Ativação do Modo MSI
      ID no JSON:  enable_msi (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Ativa interrupções baseadas em mensagem para barramentos PCI. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).  

    [JÁ OTIMIZADO NO SO] Desativação de Economia de Energia PCIe e USB
      ID no JSON:  disable_power_saving (true)
      Status SO:   Já otimizado (preservado sem reaplicar)
      Critério:    Compatível: Previne estados de suspensão em dispositivos PCIe e USB durante o jogo. | Status SO: Já otimizado no sistema (será preservado sem reaplicação).

  [4] Argumentos de Inicialização do CS2 (Launch Args):

  - -fullscreen
  - -console
  - -nojoy
  - +engine_low_latency_sleep_after_client_tick
  - true
  - +fps_max
  - 240
  - +mat_queue_mode

  [5] Persistência dos Arquivos de Configuração:

  - Hardware:    config\hardware\Hardware.json
  - AutoDetect:  config\optimzations\AutoDetect.json
  - Tamanho:     567 bytes
  - Status:      Arquivo 'AutoDetect.json' gravado e verificado com sucesso no disco.

  ================================================================================  
    Resumo da Autodetecção:
      - Otimizações recomendadas no AutoDetect.json:    12
      - Já otimizadas atualmente no sistema:            9
      - Pendentes (ainda precisam ser otimizadas):      3
      - Opcionais mantidas desativadas:                 1
      - Bloqueadas por limite de hardware:              0
  ================================================================================

  Já otimizadas no sistema: disable_memory_compression, apply_render_queue, disable_gpu_acceleration, apply_dpi_optimization, apply_timer_resolution, apply_mmcss, disable_game_bar, enable_msi, disable_power_saving
  Pendentes para otimizar: cs2_affinity, empty_standby_list, disable_hpet

  [SUCESSO] Configuração 'AutoDetect.json' gerada conforme hardware e estado atual do sistema.
