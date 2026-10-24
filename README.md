[🇺🇸 Read in English](RDM_EN-US.md)

| **COUNTER RUSH**                                                                        |
| --------------------------------------------------------------------------------------- |
| Jogue CS com mais desempenho, aumente seu FPS, elimine travamentos e quedas de quadros. |

> <span style="color:#e64340">Projeto ainda em etapa inicial de desenvolvimento; todos os módulos ainda estão em teste e podem mudar consideravelmente até a versão de lançamento.</span>

### Objetivo:

- Criar um otimizador de CS que seja gratuito e realmente funcione, sem comandos e "otimizações" placebo.
- Facilitar a criação de arquivos `.cfg` ou o download das configurações dos profissionais. 
- Entender melhor como funciona a otimização dos computadores de campeonato e trazer essas técnicas para os jogadores casuais.
- Trazer maior estabilidade ao jogo que mais joguei na minha vida.

#### O que já foi feito:

- [x] Iniciar o jogo por meio da Steam diretamente no software.
- [x] Iniciar o game com parâmetros atualizados para o CS2 e adequados para o seu hardware.
- [x] Desativar o núcleo 0 do processador, já que ele é utilizado por programas em segundo plano, gerando disputa entre o jogo e os serviços abertos no computador.
- [x] Aumentar a prioridade do processo do jogo, mantendo-o em primeiro plano para o processador.
- [x] Desativar a utilização da GPU por outros programas (Discord, Steam e navegadores), liberando VRAM e recursos com rollback opcional.
- [x] Limpar a memória RAM livre eliminando páginas em standby (clone nativo do Empty Standby List do RAMMap).
- [x] Otimização de DPI e desativação de otimizações de janela do DirectX (DPI_Otimização).
- [x] Otimização na leitura dos dados da RAM desativando a compactação de memória
- [x] Definir a resolução de temporizador do sistema para 0.5ms
- [x] Otimização do agendador multimídia (MMCSS) adaptado por perfil de hardware, priorizando rede, CPU, GPU e I/O de disco para jogos.
- [x] Desativação do Temporizador de eventos de alta precisão (HPET) via BCD.
- [x] Desativação do Modo de Jogo (Game Mode), atalhos do Xbox Game Bar, Modo Xbox e Capturas em segundo plano (GameDVR).
- [x] Ativar o Modo MSI (Message Signaled-Based Interrupts) com Prioridade Alta na GPU, Placa de Rede e Controladoras USB.
- [x] Desativar Economia de Energia PCIe (Link State Power Management) e USB (Suspensão Seletiva e Hubs).
- [x] Otimização da Fila de Quadros Renderizados (Render Queue / Flip Queue) para placas AMD e NVIDIA.

### O que está sendo feito:

- [ ] Função para lançamento do CS com otimizações personalizadas
- [ ] Interface gráfica moderna (Slint) com switches e controles modulares.

### O que ainda vai ser feito:

- Função para criar binds (atalhos dentro do game) para todos os comandos permitidos em servidores oficiais e comandos de treino.
- Interface gráfica moderna (Slint) com switches e controles modulares.
- Função para configurar o próprio `autoexec.cfg` ou baixar a configuração de algum profissional.
- Possibilidade de abrir e, talvez, baixar mapas da Oficina diretamente pelo software.
- Vizualização do Ranking valve diretamente na interface do software.

<p align="center">
  <a href="https://feitonobrasil.dev.br"><img src="https://selo.feitonobrasil.dev.br/en/branco-colorido/1x.svg" alt="Made in Brazil" height="56"></a>
</p>
