#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindCommandEntry {
    pub category: &'static str,
    pub category_note: &'static str,
    pub subcategory: &'static str,
    pub display_command: &'static str,
    pub cfg_command: &'static str,
    pub description: &'static str,
    pub featured: bool,
}

macro_rules! cmd {
    ($cat:expr, $note:expr, $sub:expr, $cmd:expr, $desc:expr) => {
        BindCommandEntry {
            category: $cat,
            category_note: $note,
            subcategory: $sub,
            display_command: $cmd,
            cfg_command: $cmd,
            description: $desc,
            featured: false,
        }
    };
    ($cat:expr, $note:expr, $sub:expr, $disp:expr => $cfg:expr, $desc:expr) => {
        BindCommandEntry {
            category: $cat,
            category_note: $note,
            subcategory: $sub,
            display_command: $disp,
            cfg_command: $cfg,
            description: $desc,
            featured: false,
        }
    };
}

macro_rules! featured_cmd {
    ($cat:expr, $note:expr, $sub:expr, $cmd:expr, $desc:expr) => {
        BindCommandEntry {
            category: $cat,
            category_note: $note,
            subcategory: $sub,
            display_command: $cmd,
            cfg_command: $cmd,
            description: $desc,
            featured: true,
        }
    };
}

const CAT_1: &str = "1. Movimentação";
const NOTE_1: &str = "Deslocamento, saltos, postura e controle de câmera (Source 2)";
const SUB_1_1: &str = "Comandos Principais";

const CAT_2: &str = "2. Treinamento";
const NOTE_2: &str = "Exigem sv_cheats 1";
const SUB_2_0: &str = "Pré-requisito de Treino (sv_cheats)";
const SUB_2_1: &str = "Movimentação Livre e Teleporte de Precisão";
const SUB_2_2: &str = "Treino Avançado de Granadas";
const SUB_2_3: &str = "Controle de Bots";
const SUB_2_4: &str = "Combate, Hitboxes e Partida";

const CAT_3: &str = "3. Jogabilidade";
const NOTE_3: &str = "Permitidos em partidas competitivas";
const SUB_3_1: &str = "Interação, Visão e Bomba (C4)";
const SUB_3_2: &str = "Radar Inteligente";
const SUB_3_3: &str = "Mira Dinâmica e Alinhamento (Crosshair)";

const CAT_4: &str = "4. Comunicação";
const NOTE_4: &str = "Voz, clutch binds, pings 3D, rodas e rádio";
const SUB_4_1: &str = "Microfone e \"Clutch Binds\" (Voz)";
const SUB_4_2: &str = "Pings e Rodas de Chat (Chat Wheels)";
const SUB_4_3: &str = "Menus de Rádio e Falas Diretas de Personagem";
const SUB_4_4: &str = "Chat de Texto";

const CAT_5: &str = "5. Armas e Granadas";
const NOTE_5: &str = "Seleção via slots e compras rápidas Source 2";
const SUB_5_1: &str = "Slots de Armas e Equipamentos";
const SUB_5_2: &str = "Seleção Direta de Granadas (Altamente Recomendado)";
const SUB_5_3: &str = "Comandos de Compra Rápida (buy e Reembolso)";

const CAT_6: &str = "6. Monitoramento (FPS, Telemetria e Rede)";
const NOTE_6: &str = "Telemetria nativa do HUD CS2 e diagnósticos";
const SUB_6_1: &str = "Telemetria Nativa do CS2 (0 = Nunca, 1 = Problema, 2 = Sempre)";
const SUB_6_2: &str = "Diagnósticos Clássicos e Placar";

const CAT_7: &str = "7. Outros (Áudio, Demos/Replays, HUD e Sistema)";
const NOTE_7: &str = "Som, análise de demos GOTV, HUD e manutenção";
const SUB_7_1: &str = "Controle de Volume e Áudio";
const SUB_7_2: &str = "Replays e Análise de Demos (GOTV / Suas Partidas)";
const SUB_7_3: &str = "Personalização de HUD e Capturas";
const SUB_7_4: &str = "Console, Sistema e Gerenciamento de Binds";

static CS2_BIND_CATALOG: &[BindCommandEntry] = &[
    // 1. Movimentação (15)
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+forward", "Move para frente (W)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+back", "Move para trás (S)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+left", "Move lateralmente / strafe para a esquerda (A)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+right", "Move lateralmente / strafe para a direita (D)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+jump", "Executa o pulo (muito utilizado no scroll do mouse para facilitar o Bunnyhop)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+duck", "Agacha o personagem (CTRL)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+sprint", "Caminha devagar e sem emitir som de passos (SHIFT)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "option_duck_method 0", "Agachar segurando a tecla (0): mantém o personagem agachado apenas enquanto a tecla estiver pressionada."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "option_duck_method 1", "Agachar alternando com um toque (1): pressiona uma vez para agachar e novamente para levantar."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "option_speed_method 0", "Caminhar silencioso segurando a tecla (0): anda devagar apenas enquanto a tecla estiver pressionada."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "option_speed_method 1", "Caminhar silencioso alternando com um toque (1): pressiona uma vez para ativar e novamente para desativar."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+turnleft", "Gira a câmera horizontalmente para a esquerda (Surf, KZ e Bhop)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+turnright", "Gira a câmera horizontalmente para a direita (Surf, KZ e Bhop)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+lookup", "Move o ângulo de visão verticalmente para cima via teclado."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+lookdown", "Move o ângulo de visão verticalmente para baixo via teclado."),

    // 2. Treinamento (37) — Pré-requisito destacado + comandos de treino
    featured_cmd!(CAT_2, NOTE_2, SUB_2_0, "sv_cheats 0", "Desativa os cheats no servidor local (0): bloqueia comandos de treinamento e restaura o modo padrão da partida."),
    featured_cmd!(CAT_2, NOTE_2, SUB_2_0, "sv_cheats 1", "Ativa os cheats no servidor local (1): libera o uso de todos os comandos de treinamento abaixo (noclip, munição infinita, granadas e bots)."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "v_regeneration_force_on 1", "ativa a regeneração automática de vida."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_roundtime 60", "define a duração máxima de cada rodada para 60 minutos em servidores locais ou partidas offline."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_roundtime_hostage 60", "Define a duração de cada rodada para 60 minutos."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_roundtime_defuse 60", "Define o tempo do round para 60 minutos."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "noclip", "Liga/desliga o voo livre atravessando paredes e objetos."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_maxmoney 65535; mp_startmoney 65535; impulse 101", "Define o limite máximo de dinheiro de jogador na partida para $65.535."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_buy_anywhere 0", "Desativa a permissão de abrir o menu e comprar armas, coletes ou granadas de qualquer lugar do mapa."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_buy_anywhere 1", "Permite abrir o menu e comprar armas, coletes ou granadas de qualquer lugar do mapa."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_buy_anywhere 2", "Permite abrir o menu e comprar armas, coletes ou granadas de qualquer lugar do mapa apenas no time Terroristas (TR)."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_buy_anywhere 3", "Permite abrir o menu e comprar armas, coletes ou granadas de qualquer lugar do mapa apenas no time Contraterroristas (CT)."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_buytime 60000", "define o tempo máximo em segundos que os jogadores têm para comprar armas e equipamentos após o início de cada round."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "getpos", "Imprime no console as coordenadas exatas (setpos e setang) da posição e mira atuais."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "mp_freezetime 0", "zera o tempo de congelamento no início de cada rodada."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "host_timescale 0.5", "Velocidade do servidor 0.5x (câmera lenta): reduz o tempo pela metade para analisar movimentos e trajetórias."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "host_timescale 1", "Velocidade do servidor 1x (normal): restaura a velocidade padrão do tempo da partida."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "host_timescale 2", "Velocidade do servidor 2x (acelerada): dobra a velocidade do tempo para avançar rapidamente no treino."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "v_grenade_trajectory_prac_trailtime 8", "ativa e mantém visível por 8 segundos a linha que mostra a trajetória das granadas arremessadas no modo de treino."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "sv_rethrow_last_grenade", "Arremessa novamente a última granada jogada no servidor."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "sv_grenade_trajectory_prac_pipreview 0", "Desativa a prévia Picture-in-Picture (0): fecha a janelinha de visualização do ponto de queda da granada."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "sv_grenade_trajectory_prac_pipreview 1", "Ativa a prévia Picture-in-Picture (1): abre a janelinha mostrando em tempo real onde sua granada vai cair."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "cl_sim_grenade_trajectory", "Congela a linha de trajetória atual da granada na sua mão por alguns segundos."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "ent_fire smokegrenade_projectile kill; ent_fire molotov_projectile kill; ent_fire inferno kill; ent_fire flashbang_projectile kill; ent_fire hegrenade_projectile kill; ent_fire decoy_projectile kill; stopsound", "Apaga instantaneamente todas as smokes, Molotovs, granadas no ar e corta efeitos sonoros ativos."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_place", "Posiciona um bot imediatamente no local onde sua mira aponta."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_add_ct", "Adiciona um bot ao lado Contraterrorista (CT)."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_add_t", "Adiciona um bot ao lado Terrorista (TR)."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_kick", "Remove todos os bots da partida."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_crouch 0", "Bots em pé (0): desativa o agachamento forçado e faz todos os bots ficarem de pé."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_crouch 1", "Bots agachados (1): força todos os bots a permanecerem agachados."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_stop 0", "Liberar movimento dos bots (0): permite que os bots voltem a andar normalmente."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_stop 1", "Congelar bots (1): trava todos os bots parados no lugar para treino de mira."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_mimic 0", "Desativar imitação dos bots (0): bots param de copiar seus movimentos e disparos."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_mimic 1", "Ativar imitação dos bots (1): faz os bots copiarem todos os seus movimentos e disparos em tempo real."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_dont_shoot 0", "Permitir disparos dos bots (0): bots voltam a atirar normalmente."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_dont_shoot 1", "Impedir disparos dos bots (1): bots ficam pacíficos e não atiram em você."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "sv_showimpacts 0", "Ocultar impactos de tiro (0): desativa os quadrados coloridos de registro de balas."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "sv_showimpacts 1", "Impactos cliente + servidor (1): exibe quadrados vermelhos (cliente) e azuis (servidor) onde os tiros atingem."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "sv_showimpacts 2", "Impactos apenas cliente (2): exibe somente os quadrados vermelhos registrados localmente pelo cliente."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "sv_infinite_ammo 0", "Munição padrão (0): desativa munição infinita e restaura o consumo normal de balas e granadas."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "sv_infinite_ammo 1", "Munição infinita sem recarregar (1): balas no carregador e granadas nunca acabam."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "sv_infinite_ammo 2", "Munição reserva infinita (2): exige recarregar a arma, mas a munição reserva e granadas nunca acabam."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "god", "Ativa invencibilidade total."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "buddha", "Permite sofrer dano até ficar com 1 de HP sem nunca morrer."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "cl_ent_hitbox", "Exibe as hitboxes do modelo para o qual você está apontando."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "r_drawothermodels 1", "Modelos de jogadores normais (1): restaura a renderização sólida padrão dos personagens."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "r_drawothermodels 2", "Modelos em modo wireframe (2): exibe o esqueleto dos jogadores visível através das paredes."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "impulse 101", "Concede dinheiro máximo instantaneamente."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "mp_restartgame 1", "Reinicia o mapa/partida em 1 segundo."),

    // 3. Jogabilidade (29)
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+attack", "Disparo primário (MOUSE1)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+attack2", "Ação secundária como zoom, silenciador ou arremesso curto (MOUSE2)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+reload", "Recarregar a arma (R)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+use", "Interagir, abrir portas, pegar armas no chão e desarmar a C4 (E)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "switchhands", "Alterna instantaneamente a arma entre a mão direita e a esquerda."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+lookatweapon", "Inspeciona a arma/faca equipada (F)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+spray_menu", "Abre o menu para aplicar grafite nas superfícies (T)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "slot5; drop", "Equipa e larga a C4 rapidamente."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "incrementvar cl_radar_scale 0.25 1.0 0.15", "Aumenta o zoom do minimapa em etapas e volta ao mínimo ao atingir o limite."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "cl_radar_scale 0.35", "Radar panorâmico (0.35): afasta o zoom do minimapa para visualizar praticamente toda a área do mapa."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "cl_radar_scale 0.75", "Radar aproximado (0.75): aproxima o zoom do minimapa com foco detalhado no bombsite e arredores."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "cl_radar_always_centered 0", "Radar sem centralização fixa (0): libera a câmera do minimapa para aproveitar melhor toda a área do mapa."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "cl_radar_always_centered 1", "Radar centralizado no jogador (1): mantém seu personagem sempre fixo no centro do minimapa."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "cl_radar_rotate 0", "Radar fixo (0): trava a orientação do minimapa sem girar com a sua visão."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "cl_radar_rotate 1", "Radar giratório (1): gira o minimapa acompanhando a direção para onde você aponta."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshair_recoil 0", "Desativar Follow Recoil (0): mantém a mira fixa no centro da tela sem acompanhar o recuo."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshair_recoil 1", "Ativar Follow Recoil (1): faz a mira acompanhar dinamicamente o padrão de recuo (spray) da arma."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshairsize 2", "Mira compacta padrão (tamanho 2): define o tamanho normal da mira para combate."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshairsize 1000", "Mira de alinhamento de smokes (tamanho 1000): estende as linhas até as bordas do monitor para alinhar utilitários."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshaircolor 1", "Cor da mira Verde (1): aplica a cor verde padrão ao crosshair."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshaircolor 2", "Cor da mira Amarela (2): aplica a cor amarela ao crosshair."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshaircolor 4", "Cor da mira Ciano (4): aplica a cor ciano ao crosshair."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshaircolor 5", "Cor da mira Personalizada (5): aplica a cor RGB personalizada configurada pelo jogador."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshairdot 0", "Desativar ponto central da mira (0): remove o ponto no meio do crosshair."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_crosshairdot 1", "Ativar ponto central da mira (1): exibe um ponto fixo no centro do crosshair."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "crosshair 0", "Ocultar mira (0): esconde completamente a mira da tela."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "crosshair 1", "Exibir mira (1): torna a mira visível novamente na tela."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_grenadecrosshair_keepusercrosshair 0", "Ocultar mira padrão nas granadas (0): exibe apenas a retícula de alinhamento de granada."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "cl_grenadecrosshair_keepusercrosshair 1", "Manter mira padrão nas granadas (1): mantém sua mira normal visível junto à retícula de granada."),

    // 4. Comunicação (34)
    cmd!(CAT_4, NOTE_4, SUB_4_1, "+voicerecord", "Aciona o microfone (Push-to-Talk)."),
    cmd!(CAT_4, NOTE_4, SUB_4_1, "voice_modenable_toggle", "Muta ou desmuta completamente a comunicação de voz da equipe com um clique."),
    cmd!(CAT_4, NOTE_4, SUB_4_1, "snd_voipvolume 0.08", "Volume da equipe reduzido / clutch (0.08): abaixa as vozes dos aliados para 8% durante situações de clutch."),
    cmd!(CAT_4, NOTE_4, SUB_4_1, "snd_voipvolume 1.0", "Volume da equipe normal (1.0): restaura o volume das vozes dos aliados para 100%."),
    cmd!(CAT_4, NOTE_4, SUB_4_1, "voice_toggle_open_mic", "Alterna entre microfone aberto e Push-to-Talk."),
    cmd!(CAT_4, NOTE_4, SUB_4_2, "player_ping", "Cria um marcador visual 3D (ping) exatamente onde sua mira aponta."),
    cmd!(CAT_4, NOTE_4, SUB_4_2, "+radialradio", "Mantém aberta a Roda de Comunicação 1."),
    cmd!(CAT_4, NOTE_4, SUB_4_2, "+radialradio2", "Mantém aberta a Roda de Comunicação 2."),
    cmd!(CAT_4, NOTE_4, SUB_4_2, "+radialradio3", "Mantém aberta a Roda de Comunicação 3."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "radio", "Abre o menu clássico principal de comandos de rádio na tela."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "radio1", "Abre o menu clássico de comandos de rádio 1."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "radio2", "Abre o menu clássico de comandos de rádio 2."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "radio3", "Abre o menu clássico de comandos de rádio 3."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "coverme", "Fala direta tática: \"Me dê cobertura!\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "takepoint", "Fala direta tática: \"Vá na frente\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "holdpos", "Fala direta tática: \"Mantenha a posição\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "followme", "Fala direta tática: \"Siga-me\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "regroup", "Fala direta tática: \"Reagrupar\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "fallback", "Fala direta tática: \"Recuar!\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "sticktog", "Fala direta tática: \"Fiquem juntos\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "go", "Fala direta tática: \"Vai, vai, vai!\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "enemyspot", "Fala direta de informação: \"Inimigo avistado\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "needbackup", "Fala direta de informação: \"Preciso de reforço\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "sectorclear", "Fala direta de informação: \"Setor limpo\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "inposition", "Fala direta de informação: \"Em posição\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "enemydown", "Fala direta de informação: \"Inimigo abatido\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "getout", "Fala direta de informação: \"Saiam daí, vai explodir!\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "thanks", "Fala direta social: \"Obrigado\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "compliment", "Fala direta social: \"Boa!\"."),
    cmd!(CAT_4, NOTE_4, SUB_4_3, "cheer", "Fala direta social: Comemorar."),
    cmd!(CAT_4, NOTE_4, SUB_4_4, "messagemode", "Abre a caixa de digitação para Todos (Y padrão)."),
    cmd!(CAT_4, NOTE_4, SUB_4_4, "messagemode2", "Abre a caixa de digitação apenas para a Equipe (U padrão)."),
    cmd!(CAT_4, NOTE_4, SUB_4_4, "say \"texto\"" => "say GGWP!", "Envia instantaneamente uma frase no chat geral (gravado sem aspas internas)."),
    cmd!(CAT_4, NOTE_4, SUB_4_4, "say_team \"texto\"" => "say_team Preciso de drop, por favor!", "Envia instantaneamente uma frase apenas para o seu time (gravado sem aspas internas)."),

    // 5. Armas e Granadas (64)
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot1", "Arma primária (Rifles, SMGs, Escopetas, Metralhadoras)."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot2", "Arma secundária (Pistolas)."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot3", "Faca (ou Zeus x27, se já estiver com a faca na mão)."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot4", "Percorre sequencialmente todas as granadas no inventário."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot5", "Equipa diretamente o explosivo C4."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot11", "Equipa diretamente a Zeus x27 sem precisar apertar a faca duas vezes."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "slot12", "Equipa o Healthshot (seringa médica em modos Mata-Mata / Treino)."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "lastinv", "Alterna para o último item equipado (Q padrão)."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "invnext", "Seleciona a próxima arma do inventário."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "invprev", "Seleciona a arma anterior do inventário."),
    cmd!(CAT_5, NOTE_5, SUB_5_1, "drop", "Joga no chão a arma ou utilitário atual (G padrão)."),
    cmd!(CAT_5, NOTE_5, SUB_5_2, "slot6", "Granada HE (Explosiva)."),
    cmd!(CAT_5, NOTE_5, SUB_5_2, "slot7", "Flashbang (Luz)."),
    cmd!(CAT_5, NOTE_5, SUB_5_2, "slot8", "Smoke (Fumaça)."),
    cmd!(CAT_5, NOTE_5, SUB_5_2, "slot9", "Decoy (Distração)."),
    cmd!(CAT_5, NOTE_5, SUB_5_2, "slot10", "Molotov / Granada Incendiária."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "autobuy", "Compra automática (F3 padrão)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "rebuy", "Recompra do round anterior (F4 padrão)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "sellbackall", "Reembolsa instantaneamente todas as compras feitas no freezetime atual."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy vest", "Compra Colete."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy vesthelm", "Compra Colete + Capacete."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy defuser", "Compra Kit Defuse (CT)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy taser", "Compra Zeus x27."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy smokegrenade", "Compra Smoke."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy flashbang", "Compra Flashbang."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy hegrenade", "Compra Granada HE."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy molotov; buy incgrenade", "Compra Molotov (TR) ou Granada Incendiária (CT)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy decoy", "Compra Decoy."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy glock; buy hkp2000; buy usp_silencer", "Compra pistola inicial (Glock / P2000 / USP-S)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy p250", "Compra P250."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy tec9; buy fiveseven; buy cz75a", "Compra Tec-9 / Five-SeveN / CZ75-Auto."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy deagle; buy revolver", "Compra Desert Eagle / R8 Revolver."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy elite", "Compra Dual Berettas."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy galilar; buy famas", "Compra Galil AR (TR) ou FAMAS (CT)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy ak47; buy m4a1; buy m4a1_silencer", "Compra AK-47 (TR), M4A4 (m4a1) ou M4A1-S (m4a1_silencer)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy ssg08", "Compra SSG 08 (Scout)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy sg556; buy aug", "Compra SG 553 (TR) ou AUG (CT)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy awp", "Compra AWP."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy g3sg1; buy scar20", "Compra G3SG1 (TR) ou SCAR-20 (CT)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy mac10; buy mp9", "Compra MAC-10 (TR) ou MP9 (CT)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy mp7; buy mp5sd", "Compra MP7 / MP5-SD."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy ump45", "Compra UMP-45."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy p90", "Compra P90."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy bizon", "Compra PP-Bizon."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy nova", "Compra Nova."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy xm1014", "Compra XM1014."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy mag7; buy sawedoff", "Compra MAG-7 (CT) ou Sawed-Off (TR)."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy m249", "Compra M249."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy negev", "Compra Negev."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy secondary0", "Compra pistola do espaço 0 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy secondary1", "Compra pistola do espaço 1 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy secondary2", "Compra pistola do espaço 2 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy secondary3", "Compra pistola do espaço 3 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy secondary4", "Compra pistola do espaço 4 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy midtier0", "Compra arma intermediária (Mid-Tier) do espaço 0 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy midtier1", "Compra arma intermediária (Mid-Tier) do espaço 1 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy midtier2", "Compra arma intermediária (Mid-Tier) do espaço 2 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy midtier3", "Compra arma intermediária (Mid-Tier) do espaço 3 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy midtier4", "Compra arma intermediária (Mid-Tier) do espaço 4 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy rifle0", "Compra rifle do espaço 0 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy rifle1", "Compra rifle do espaço 1 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy rifle2", "Compra rifle do espaço 2 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy rifle3", "Compra rifle do espaço 3 do Loadout CS2."),
    cmd!(CAT_5, NOTE_5, SUB_5_3, "buy rifle4", "Compra rifle do espaço 4 do Loadout CS2."),

    // 6. Monitoramento (24)
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_frametime_show 0", "Ocultar FPS/Frametime (0 = Nunca): desativa o indicador no canto superior direito."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_frametime_show 1", "FPS/Frametime em problemas (1): exibe automaticamente apenas durante quedas de FPS ou picos de frametime."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_frametime_show 2", "FPS/Frametime sempre visível (2): exibe continuamente FPS e tempo de quadro (ms) no canto superior direito."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_ping_show 0", "Ocultar Ping (0 = Nunca): desativa o indicador de latência no HUD."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_ping_show 1", "Ping em problemas (1): exibe o ping apenas quando houver latência alta ou instável."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_ping_show 2", "Ping sempre visível (2): exibe continuamente o seu Ping (ms) em tempo real."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_net_misdelivery_show 0", "Ocultar perda de pacotes/jitter (0 = Nunca): desativa alertas de rede no HUD."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_net_misdelivery_show 1", "Perda de pacotes/jitter em problemas (1): exibe alerta apenas quando houver perda ou atraso de pacotes."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_net_misdelivery_show 2", "Perda de pacotes/jitter sempre visível (2): exibe continuamente a porcentagem de packet loss e jitter."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_serverrecvmargin_graph_show 0", "Ocultar gráfico de estabilidade do servidor (0 = Nunca): desativa o micrográfico de envio."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_serverrecvmargin_graph_show 1", "Gráfico do servidor em problemas (1): exibe o micrográfico apenas durante instabilidades de envio."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "cl_hud_telemetry_serverrecvmargin_graph_show 2", "Gráfico do servidor sempre visível (2): exibe continuamente o micrográfico de estabilidade de envio ao servidor."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cq_netgraph 0", "Desativar gráfico Command Queue (0): oculta a barra gráfica de sub-tick no canto superior direito."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cq_netgraph 1", "Ativar gráfico Command Queue (1): exibe a barra gráfica de fila de comandos (sub-tick) no canto superior direito."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showfps 0", "Ocultar FPS da engine (0): desativa o contador direto de FPS."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showfps 1", "FPS da engine simples (1): exibe o contador básico de FPS em tempo real."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showfps 2", "FPS da engine com frametime (2): exibe FPS com métricas de tempo mínimo, médio e máximo de quadro."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showfps 3", "FPS da engine com ticks do servidor (3): exibe FPS junto com latência e ticks do servidor."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showfps 4", "FPS da engine completo (4): exibe diagnóstico detalhado de frame e latência da engine."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showpos 0", "Ocultar posição e velocidade (0): remove coordenadas, ângulo e velocidade da tela."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "cl_showpos 1", "Exibir posição e velocidade (1): mostra coordenadas (pos), ângulo (ang) e velocidade horizontal (vel) na tela."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "+showscores", "Abre a tabela de pontuação / Placar (TAB)."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "status", "Imprime no console dados do servidor atual, mapa, ping e jogadores conectados."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "sys_info", "Gera no console um relatório completo de hardware, drivers e configurações ativas."),

    // 7. Outros (34)
    cmd!(CAT_7, NOTE_7, SUB_7_1, "incrementvar volume 0 1 0.05", "Aumenta o volume geral do jogo em passos de 5%."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "incrementvar volume 0 1 -0.05", "Diminui o volume geral do jogo em passos de 5%."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "volume 0.08", "Volume Modo Aquecimento/DM (0.08): reduz o volume geral do jogo para 8%."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "volume 0.55", "Volume Modo Competitivo (0.55): define o volume geral do jogo para 55%."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "snd_mute_losefocus 0", "Manter som no Alt+Tab (0): o áudio do jogo continua tocando em segundo plano ao minimizar."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "snd_mute_losefocus 1", "Mutar som no Alt+Tab (1): silencia automaticamente o jogo quando a janela perde o foco."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demoui", "Abre ou fecha a barra de controle de reprodução de Demo (Shift+F2)."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_togglepause", "Pausa ou retoma a reprodução da Demo instantaneamente."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_timescale 0.25", "Velocidade da Demo 0.25x: reprodução super lenta para análise minuciosa de jogadas."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_timescale 0.5", "Velocidade da Demo 0.5x: câmera lenta pela metade para revisar mira e movimentação."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_timescale 1", "Velocidade da Demo 1x (normal): restaura a velocidade original do replay."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_timescale 2", "Velocidade da Demo 2x: avança o replay no dobro da velocidade."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_timescale 4", "Velocidade da Demo 4x: avança rapidamente o replay em 4 vezes."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_timescale 8", "Velocidade da Demo 8x: avanço ultrarrápido do replay para pular rounds."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_goto -960 1 0", "Volta aproximadamente 15 segundos na linha do tempo da Demo."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_goto 960 1 0", "Avança aproximadamente 15 segundos na linha do tempo da Demo."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "spec_show_xray 0", "Desativar Raio-X (0): oculta a silhueta dos jogadores através das paredes em Demos/espectador."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "spec_show_xray 1", "Ativar Raio-X (1): destaca a silhueta dos jogadores através das paredes em Demos/espectador."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "incrementvar cl_hud_color 0 11 1", "Troca ciclicamente a cor de toda a interface (HUD) entre as 12 cores disponíveis."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "cl_draw_only_deathnotices 0", "Restaurar HUD completo (0): volta a exibir radar, vida, munição e elementos normais da interface."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "cl_draw_only_deathnotices 1", "Modo apenas killfeed (1): oculta HUD e radar deixando somente a mira e o registro de abates."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "cl_drawhud 0", "Ocultar toda a interface (0, exige sv_cheats 1): remove absolutamente todos os elementos do HUD da tela."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "cl_drawhud 1", "Exibir toda a interface (1): restaura a exibição completa dos elementos do HUD."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "screenshot", "Captura uma foto imediata da tela."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "jpeg", "Captura uma foto imediata da tela em formato JPEG."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "toggleconsole", "Abre ou fecha o Console de Desenvolvedor (' ou ~)."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "exec autoexec", "Carrega imediatamente todas as configurações salvas no arquivo autoexec.cfg."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "callvote", "Abre o menu de votações (pedir pausa, expulsar jogador, render-se)."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "disconnect", "Desconecta imediatamente do servidor/partida atual e volta ao menu inicial."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "quit", "Fecha o Counter-Strike 2 instantaneamente para a Área de Trabalho."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "key_listboundkeys", "Lista no console todas as teclas que possuem binds ativas no momento."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "key_findbinding \"comando\"" => "key_findbinding +voicerecord", "Pesquisa qual tecla está associada a um comando específico (gravado sem aspas internas)."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "find \"palavra\"" => "find radar", "Busca qualquer comando ou variável existente no CS2 com o termo pesquisado (gravado sem aspas internas)."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "binddefaults", "Restaura todas as teclas para a configuração original de fábrica da Valve."),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindCategoryInfo {
    pub index: usize,
    pub title: &'static str,
    pub tab_label: &'static str,
    pub subtitle: &'static str,
    pub command_count: usize,
}

fn category_tab_label(title: &'static str) -> &'static str {
    match title {
        CAT_1 => "Movimentação",
        CAT_2 => "Treinamento",
        CAT_3 => "Jogabilidade",
        CAT_4 => "Comunicação",
        CAT_5 => "Armas e Granadas",
        CAT_6 => "Monitoramento",
        CAT_7 => "Outros",
        _ => title,
    }
}

/// Retorna o catálogo completo de binds do CS2.
pub fn cs2_bind_catalog() -> &'static [BindCommandEntry] {
    CS2_BIND_CATALOG
}

/// Retorna os metadados das categorias do catálogo na ordem oficial, derivados de `CS2_BIND_CATALOG`.
pub fn cs2_bind_categories() -> Vec<BindCategoryInfo> {
    let mut categories: Vec<BindCategoryInfo> = Vec::new();
    for entry in CS2_BIND_CATALOG {
        if let Some(last) = categories.last_mut() {
            if last.title == entry.category {
                last.command_count += 1;
                continue;
            }
        }
        let index = categories.len();
        categories.push(BindCategoryInfo {
            index,
            title: entry.category,
            tab_label: category_tab_label(entry.category),
            subtitle: entry.category_note,
            command_count: 1,
        });
    }
    categories
}

/// Normaliza texto para busca insensível a maiúsculas/minúsculas e diacríticos comuns em PT-BR.
pub fn normalize_search_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    for ch in input.chars().flat_map(char::to_lowercase) {
        let folded = match ch {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            other => other,
        };
        normalized.push(folded);
    }
    normalized
}

/// Verifica se um comando do catálogo corresponde a todos os termos da pesquisa no comando ou descrição.
pub fn matches_bind_search(entry: &BindCommandEntry, query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }

    let normalized_query = normalize_search_text(trimmed);
    let terms: Vec<&str> = normalized_query.split_whitespace().collect();
    if terms.is_empty() {
        return true;
    }

    let norm_display = normalize_search_text(entry.display_command);
    let norm_cfg = normalize_search_text(entry.cfg_command);
    let norm_desc = normalize_search_text(entry.description);

    terms.iter().all(|term| {
        norm_display.contains(term) || norm_cfg.contains(term) || norm_desc.contains(term)
    })
}

/// Converte o texto ou identificador de uma tecla capturada para o código oficial do CS2 (Source 2).
pub fn normalize_captured_key_to_cs2(raw_input: &str) -> Option<String> {
    if raw_input.is_empty() {
        return None;
    }

    if raw_input == " " {
        return Some("space".to_string());
    }

    if raw_input.len() == 1 {
        let ch = raw_input.chars().next()?;
        let mapped = match ch {
            '\n' | '\r' => Some("enter".to_string()),
            '\t' => Some("tab".to_string()),
            '\u{001b}' => Some("escape".to_string()),
            '\u{0008}' => Some("backspace".to_string()),
            '\u{007f}' => Some("del".to_string()),
            ';' => Some("semicolon".to_string()),
            '"' | '\0' => None,
            c if c.is_ascii_alphanumeric() => Some(c.to_ascii_lowercase().to_string()),
            c if !c.is_whitespace() && !c.is_control() => Some(c.to_string()),
            _ => None,
        };
        return mapped.and_then(|k| crate::binds::validate_bind_key(&k).ok());
    }

    let trimmed = raw_input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    let candidate = match lower.as_str() {
        "space" | "spacebar" => "space",
        "enter" | "return" => "enter",
        "tab" => "tab",
        "esc" | "escape" => "escape",
        "backspace" | "back" => "backspace",
        "del" | "delete" => "del",
        "ins" | "insert" => "ins",
        "home" => "home",
        "end" => "end",
        "pgup" | "pageup" | "page_up" => "pgup",
        "pgdn" | "pagedown" | "page_down" => "pgdn",
        "uparrow" | "up" | "arrowup" => "uparrow",
        "downarrow" | "down" | "arrowdown" => "downarrow",
        "leftarrow" | "left" | "arrowleft" => "leftarrow",
        "rightarrow" | "right" | "arrowright" => "rightarrow",
        "shift" | "lshift" | "shift_l" => "shift",
        "rshift" | "shiftr" | "shift_r" => "rshift",
        "ctrl" | "control" | "lctrl" | "control_l" => "ctrl",
        "rctrl" | "controlr" | "control_r" => "rctrl",
        "alt" | "lalt" | "alt_l" => "alt",
        "ralt" | "altgr" | "altr" | "alt_r" => "ralt",
        "capslock" | "caps_lock" => "capslock",
        "mouse1" | "mouse2" | "mouse3" | "mouse4" | "mouse5" | "mwheelup" | "mwheeldown" => {
            lower.as_str()
        }
        "kp_0" | "kp_1" | "kp_2" | "kp_3" | "kp_4" | "kp_5" | "kp_6" | "kp_7" | "kp_8"
        | "kp_9" | "kp_del" | "kp_enter" | "kp_plus" | "kp_minus" | "kp_multiply"
        | "kp_divide" => lower.as_str(),
        "f1" | "f2" | "f3" | "f4" | "f5" | "f6" | "f7" | "f8" | "f9" | "f10" | "f11"
        | "f12" | "f13" | "f14" | "f15" | "f16" | "f17" | "f18" | "f19" | "f20" | "f21"
        | "f22" | "f23" | "f24" => lower.as_str(),
        "semicolon" => "semicolon",
        other => other,
    };

    crate::binds::validate_bind_key(candidate).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binds::format_bind_commands;
    use std::collections::HashSet;

    #[test]
    fn test_catalog_count_and_exact_category_order() {
        let catalog = cs2_bind_catalog();
        assert_eq!(catalog.len(), 237);

        let mut categories = Vec::new();
        for entry in catalog {
            if categories.last().copied() != Some(entry.category) {
                categories.push(entry.category);
            }
        }

        assert_eq!(
            categories,
            vec![
                "1. Movimentação",
                "2. Treinamento",
                "3. Jogabilidade",
                "4. Comunicação",
                "5. Armas e Granadas",
                "6. Monitoramento (FPS, Telemetria e Rede)",
                "7. Outros (Áudio, Demos/Replays, HUD e Sistema)",
            ]
        );
    }

    #[test]
    fn test_training_starts_with_featured_sv_cheats_and_no_grouped_toggle_modes() {
        let catalog = cs2_bind_catalog();
        let training: Vec<&BindCommandEntry> = catalog
            .iter()
            .filter(|e| e.category == CAT_2)
            .collect();

        assert_eq!(training.len(), 37);
        assert_eq!(training[0].cfg_command, "sv_cheats 0");
        assert!(training[0].featured);
        assert_eq!(training[0].subcategory, SUB_2_0);
        assert_eq!(training[1].cfg_command, "sv_cheats 1");
        assert!(training[1].featured);
        assert_eq!(training[1].subcategory, SUB_2_0);
        assert!(training[2..].iter().all(|e| !e.featured));
        assert!(catalog.iter().filter(|e| e.featured).count() == 2);

        let mut seen = HashSet::new();
        for entry in catalog {
            assert!(
                !entry.cfg_command.starts_with("toggle "),
                "Comando agrupado com toggle não deveria existir: {}",
                entry.cfg_command
            );
            assert!(
                seen.insert(entry.cfg_command),
                "Comando duplicado no catálogo: {}",
                entry.cfg_command
            );
        }
    }

    #[test]
    fn test_all_catalog_cfg_commands_are_valid_for_cfg_writer() {
        for entry in cs2_bind_catalog() {
            let formatted = format_bind_commands(&[entry.cfg_command]);
            assert!(
                formatted.is_ok(),
                "Comando '{}' inválido para gravação .cfg: {:?}",
                entry.cfg_command,
                formatted.err()
            );
        }
    }

    #[test]
    fn test_normalize_captured_key_to_cs2_mappings() {
        assert_eq!(normalize_captured_key_to_cs2(" "), Some("space".to_string()));
        assert_eq!(normalize_captured_key_to_cs2("K"), Some("k".to_string()));
        assert_eq!(normalize_captured_key_to_cs2("F4"), Some("f4".to_string()));
        assert_eq!(
            normalize_captured_key_to_cs2("UpArrow"),
            Some("uparrow".to_string())
        );
        assert_eq!(
            normalize_captured_key_to_cs2("ShiftR"),
            Some("rshift".to_string())
        );
        assert_eq!(
            normalize_captured_key_to_cs2("kp_1"),
            Some("kp_1".to_string())
        );
        assert_eq!(
            normalize_captured_key_to_cs2(";"),
            Some("semicolon".to_string())
        );
        assert_eq!(normalize_captured_key_to_cs2("\""), None);
        assert_eq!(normalize_captured_key_to_cs2(""), None);
    }

    #[test]
    fn test_cs2_bind_categories_integrity() {
        let categories = cs2_bind_categories();
        assert_eq!(categories.len(), 7);
        assert_eq!(
            categories
                .iter()
                .map(|c| c.tab_label)
                .collect::<Vec<_>>(),
            vec![
                "Movimentação",
                "Treinamento",
                "Jogabilidade",
                "Comunicação",
                "Armas e Granadas",
                "Monitoramento",
                "Outros",
            ]
        );
        assert!(categories.iter().all(|c| !c.subtitle.is_empty()));
        let total: usize = categories.iter().map(|c| c.command_count).sum();
        assert_eq!(total, 237);
    }

    #[test]
    fn test_unbind_is_not_in_regular_bind_catalog() {
        for entry in cs2_bind_catalog() {
            assert!(
                !entry.cfg_command.to_ascii_lowercase().starts_with("unbind"),
                "Comando unbind deve ficar separado do catálogo comum: {}",
                entry.cfg_command
            );
            assert!(
                !entry.display_command.to_ascii_lowercase().starts_with("unbind"),
                "Comando unbind não deve aparecer na listagem comum: {}",
                entry.display_command
            );
        }
    }

    #[test]
    fn test_matches_bind_search_command_and_description() {
        let catalog = cs2_bind_catalog();

        let cheats_on = catalog
            .iter()
            .find(|e| e.cfg_command == "sv_cheats 1")
            .unwrap();
        assert!(matches_bind_search(cheats_on, "sv_cheats"));
        assert!(matches_bind_search(cheats_on, "cheats 1"));

        let noclip = catalog.iter().find(|e| e.cfg_command == "noclip").unwrap();
        assert!(matches_bind_search(noclip, ""));
        assert!(matches_bind_search(noclip, "noclip"));
        assert!(matches_bind_search(noclip, "NOCLIP"));
        assert!(matches_bind_search(noclip, "voo livre"));
        assert!(!matches_bind_search(noclip, "granada"));

        let smoke_slot = catalog.iter().find(|e| e.cfg_command == "slot8").unwrap();
        assert!(matches_bind_search(smoke_slot, "slot8"));
        assert!(matches_bind_search(smoke_slot, "fumaça"));
        assert!(matches_bind_search(smoke_slot, "fumaca"));

        let infinite_ammo = catalog
            .iter()
            .find(|e| e.cfg_command == "sv_infinite_ammo 1")
            .unwrap();
        assert!(matches_bind_search(infinite_ammo, "munição"));
        assert!(matches_bind_search(infinite_ammo, "municao granadas"));

        let say_entry = catalog
            .iter()
            .find(|e| e.cfg_command == "say GGWP!")
            .unwrap();
        assert!(matches_bind_search(say_entry, "ggwp"));
        assert!(matches_bind_search(say_entry, "say texto"));

        assert!(!catalog
            .iter()
            .any(|e| matches_bind_search(e, "comando_inexistente_xyz")));
    }
}
