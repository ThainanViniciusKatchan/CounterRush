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

Verifica a contagem de núcleos físicos da CPU, consulta o status atual e aplica prioridade `RealTime` e isolamento do Core 0 (`-band -2`) no processo `cs2.exe` (a afinidade só é aplicada em processadores com mais de 4 núcleos).

```powershell
cargo run -p tweaks --example test_process_tweaks
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
