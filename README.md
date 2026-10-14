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

### O que está sendo feito:

- [ ] Desativar a utilização da GPU (placa de vídeo) por outros programas, como Discord, Steam e navegadores, pois a concorrência por recursos pode diminuir o FPS ou causar travamentos.
- [ ] Limpar a memória RAM livre eliminando arquivos em standby (processos não utilizados no momento que ficam salvos na memória), liberando mais recursos para o jogo.

### O que ainda vai ser feito:

- Função para criar binds (atalhos dentro do game) para todos os comandos permitidos em servidores oficiais e comandos de treino.
- Função para configurar o próprio `autoexec.cfg` ou baixar a configuração de algum profissional.
- Possibilidade de abrir e, talvez, baixar mapas da Oficina diretamente pelo software.

<p align="center">
  <a href="https://feitonobrasil.dev.br"><img src="https://selo.feitonobrasil.dev.br/en/branco-colorido/1x.svg" alt="Made in Brazil" height="56"></a>
</p>
