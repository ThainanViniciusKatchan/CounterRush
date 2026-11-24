

## 4. Divergências e Pontos de Atenção Encontrados

### 1. Divergência de Hardware em GPU Híbrida (Intel + NVIDIA)

- **O que ocorreu**: [detect_gpu](file:///D:/CounterRush/windows_api/src/hardware.rs#L96-L103) lê as subchaves `000*` da classe Display no Registro e usa `Select-Object -First 1`. Como o primeiro adaptador listado é a GPU integrada `Intel(R) UHD Graphics` (`0.00 GB` VRAM), a dedicada `NVIDIA GeForce RTX 3050 6GB Laptop GPU` (visível em `test_msi --status` e `test_render_queue --status`) foi ignorada.
- **Impactos**:
  - `apply_render_queue` foi bloqueada por hardware em [core/src/lib.rs](file:///D:/CounterRush/core/src/lib.rs#L762-L793), deixando de tentar ativar o modo de persistência na RTX 3050.
  - A VRAM salva em `Hardware.json` ficou zerada (`0.0`).
  - **Sobre a categoria (`classify_pc`)**: em [core/src/classifier.rs](file:///D:/CounterRush/core/src/classifier.rs#L45-L72), `PcCategory::Intermediario` exige `ram_gb >= 15.0` além da VRAM (`>= 3.5 GB`). Como este PC possui `7.71 GB` de RAM, a categoria permaneceria **PC Econômico** mesmo se a RTX 3050 tivesse sido detectada — porém, em sistemas com `16 GB` ou mais de RAM e GPU híbrida, essa falha rebaixaria o perfil para `PC Econômico`.

### 2. Toggles Permanentes na GUI após Auto Otimização

- Ao concluir a Auto Otimização, as otimizações permanentes já aplicadas ou recém-aplicadas recebem `false` no `AutoDetect.json` ([ui/src/main.rs](file:///D:/CounterRush/ui/src/main.rs#L622-L624) e [core/src/lib.rs](file:///D:/CounterRush/core/src/lib.rs#L899-L907)) para não serem reaplicadas a cada inicialização do jogo.
- Na interface, isso faz seus switches ficarem visualmente desligados, podendo dar a impressão ao usuário de que não estão ativos no Windows (diferente do que mostram os comandos `--status`).

### 3. Falhas Exclusivas do Script Auxiliar de Coleta (Não são bugs do CounterRush)

- **Erro `test_gpu_accel .exe`**: causado por um array aninhado no script auxiliar [coleta_status.ps1](file:///C:/Users/Thainan/.gemini/antigravity/brain/46aad192-a555-472a-9f3f-b8cfb5559570/scratch/coleta_status.ps1) (`$cmds += ,@('test_gpu_accel','')`), que concatenou um espaço antes de `.exe`. O binário `test_gpu_accel.exe` do projeto funciona normalmente quando executado diretamente.
- **Cabeçalho `Admin: False` nos logs**: falso negativo no script de coleta por verificar o nome do grupo em inglês (`'Administrators'`) em Windows pt-BR (`'Administradores'`). A execução elevada nos logs do usuário é comprovada porque `test_hpet --status` consultou `bcdedit` sem erro (sem privilégios de Administrador, `bcdedit` falha com *"O tipo de entrada especificado é inválido"*).

---

## 5. Resumo das Alterações no Ambiente Durante o Teste

- **Código-fonte**: nenhuma alteração realizada.
- **Arquivos de configuração**:
  - [config/hardware/Hardware.json](file:///D:/CounterRush/config/hardware/Hardware.json) foi regravado por `test_hardware_detection` com conteúdo idêntico ao cache.
  - [target/debug/config/optimzations/AutoDetect.json](file:///D:/CounterRush/target/debug/config/optimzations/AutoDetect.json) foi criado e [target/debug/config/settings.json](file:///D:/CounterRush/target/debug/config/settings.json) foi atualizado pela GUI.
  - Os arquivos originais anteriores ao teste estão salvos em [scratch/backup](file:///C:/Users/Thainan/.gemini/antigravity/brain/46aad192-a555-472a-9f3f-b8cfb5559570/scratch/backup).
- **Sistema Operacional**: a única alteração permanente aplicada pela Auto Otimização foi a desativação da economia de energia em `USB4\VIRTUAL_POWER_PDO\4&28d2e06c&0&0_0` (`disable_power_saving`, que passou de `Default` para `Applied`), além da limpeza de Standby List e aplicação do timer de `0.5 ms` na sessão.
