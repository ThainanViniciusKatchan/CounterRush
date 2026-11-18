#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindCommandEntry {
    pub category: &'static str,
    pub category_note: &'static str,
    pub subcategory: &'static str,
    pub display_command: &'static str,
    pub cfg_command: &'static str,
    pub description: &'static str,
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
        }
    };
}

const CAT_1: &str = "1. Movimentação";
const NOTE_1: &str = "Deslocamento, saltos, postura e controle de câmera (Source 2)";
const SUB_1_1: &str = "Comandos Principais";

const CAT_2: &str = "2. Treinamento";
const NOTE_2: &str = "Exigem sv_cheats 1";
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
    // 1. Movimentação
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+forward", "Move para frente (W)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+back", "Move para trás (S)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+left", "Move lateralmente / strafe para a esquerda (A)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+right", "Move lateralmente / strafe para a direita (D)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+jump", "Executa o pulo (muito utilizado no scroll do mouse para facilitar o Bunnyhop)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+duck", "Agacha o personagem (CTRL)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+sprint", "Caminha devagar e sem emitir som de passos (SHIFT)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "toggle option_duck_method 0 1", "Alterna o modo de agachar entre segurar a tecla (0) e manter agachado (1)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "toggle option_speed_method 0 1", "Alterna o caminhar silencioso entre segurar (0) e alternar com um toque (1)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+turnleft", "Gira a câmera horizontalmente para a esquerda (Surf, KZ e Bhop)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+turnright", "Gira a câmera horizontalmente para a direita (Surf, KZ e Bhop)."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+lookup", "Move o ângulo de visão verticalmente para cima via teclado."),
    cmd!(CAT_1, NOTE_1, SUB_1_1, "+lookdown", "Move o ângulo de visão verticalmente para baixo via teclado."),

    // 2. Treinamento (Exigem sv_cheats 1)
    cmd!(CAT_2, NOTE_2, SUB_2_1, "noclip", "Liga/desliga o voo livre atravessando paredes e objetos."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "getpos", "Imprime no console as coordenadas exatas (setpos e setang) da posição e mira atuais."),
    cmd!(CAT_2, NOTE_2, SUB_2_1, "toggle host_timescale 1 0.5 2", "Alterna a velocidade do servidor entre normal (1), câmera lenta (0.5) e acelerada (2)."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "sv_rethrow_last_grenade", "Arremessa novamente a última granada jogada no servidor."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "toggle sv_grenade_trajectory_prac_pipreview 0 1", "Abre ou fecha a janelinha Picture-in-Picture mostrando onde sua granada vai cair."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "cl_sim_grenade_trajectory", "Congela a linha de trajetória atual da granada na sua mão por alguns segundos."),
    cmd!(CAT_2, NOTE_2, SUB_2_2, "ent_fire smokegrenade_projectile kill; ent_fire molotov_projectile kill; ent_fire inferno kill; ent_fire flashbang_projectile kill; ent_fire hegrenade_projectile kill; ent_fire decoy_projectile kill; stopsound", "Apaga instantaneamente todas as smokes, Molotovs, granadas no ar e corta efeitos sonoros ativos."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_place", "Posiciona um bot imediatamente no local onde sua mira aponta."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_add_ct", "Adiciona um bot ao lado Contraterrorista (CT)."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_add_t", "Adiciona um bot ao lado Terrorista (TR)."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "bot_kick", "Remove todos os bots da partida."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "toggle bot_crouch 0 1", "Faz todos os bots ficarem agachados ou em pé."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "toggle bot_stop 0 1", "Congela ou libera a movimentação dos bots."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "toggle bot_mimic 0 1", "Faz os bots copiarem todos os seus movimentos e disparos em tempo real."),
    cmd!(CAT_2, NOTE_2, SUB_2_3, "toggle bot_dont_shoot 0 1", "Impede ou permite que os bots atirem."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "toggle sv_showimpacts 0 1 2", "Mostra quadrados coloridos de impacto dos tiros (1 = cliente + servidor, 2 = apenas cliente)."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "toggle sv_infinite_ammo 0 1 2", "Alterna munição e granadas infinitas (1 = sem recarregar, 2 = reserva infinita)."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "god", "Ativa invencibilidade total."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "buddha", "Permite sofrer dano até ficar com 1 de HP sem nunca morrer."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "cl_ent_hitbox", "Exibe as hitboxes do modelo para o qual você está apontando."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "toggle r_drawothermodels 1 2", "Alterna renderização de personagens entre normal (1) e modo wireframe através das paredes (2)."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "impulse 101", "Concede dinheiro máximo instantaneamente."),
    cmd!(CAT_2, NOTE_2, SUB_2_4, "mp_restartgame 1", "Reinicia o mapa/partida em 1 segundo."),

    // 3. Jogabilidade
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+attack", "Disparo primário (MOUSE1)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+attack2", "Ação secundária como zoom, silenciador ou arremesso curto (MOUSE2)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+reload", "Recarregar a arma (R)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+use", "Interagir, abrir portas, pegar armas no chão e desarmar a C4 (E)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "switchhands", "Alterna instantaneamente a arma entre a mão direita e a esquerda."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+lookatweapon", "Inspeciona a arma/faca equipada (F)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "+spray_menu", "Abre o menu para aplicar grafite nas superfícies (T)."),
    cmd!(CAT_3, NOTE_3, SUB_3_1, "slot5; drop", "Equipa e larga a C4 rapidamente."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "incrementvar cl_radar_scale 0.25 1.0 0.15", "Aumenta o zoom do minimapa em etapas e volta ao mínimo ao atingir o limite."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "toggle cl_radar_scale 0.35 0.75", "Alterna entre visão panorâmica do mapa (0.35) e aproximada no bombsite (0.75)."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "toggle cl_radar_always_centered 0 1", "Alterna entre manter o personagem sempre no centro do radar ou usar todo o espaço."),
    cmd!(CAT_3, NOTE_3, SUB_3_2, "toggle cl_radar_rotate 0 1", "Alterna entre radar giratório e radar fixo."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "toggle cl_crosshair_recoil 0 1", "Ativa ou desativa a mira que segue o padrão de recuo (Follow Recoil)."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "toggle cl_crosshairsize 2 1000", "Estende as linhas da mira até as bordas do monitor (1000) para alinhar smokes e volta a 2."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "toggle cl_crosshaircolor 1 2 4 5", "Troca a cor da mira na hora (Verde, Amarelo, Ciano, Personalizada)."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "toggle cl_crosshairdot 0 1", "Liga ou desliga o ponto central da mira."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "toggle crosshair 0 1", "Oculta ou exibe completamente a mira."),
    cmd!(CAT_3, NOTE_3, SUB_3_3, "toggle cl_grenadecrosshair_keepusercrosshair 0 1", "Define se sua mira padrão permanece visível quando a retícula de granada entra na tela."),

    // 4. Comunicação
    cmd!(CAT_4, NOTE_4, SUB_4_1, "+voicerecord", "Aciona o microfone (Push-to-Talk)."),
    cmd!(CAT_4, NOTE_4, SUB_4_1, "voice_modenable_toggle", "Muta ou desmuta completamente a comunicação de voz da equipe com um clique."),
    cmd!(CAT_4, NOTE_4, SUB_4_1, "toggle snd_voipvolume 0.08 1.0", "Reduz o volume das vozes do time para 8% (0.08) em clutch e retorna a 100% (1.0)."),
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

    // 5. Armas e Granadas
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

    // 6. Monitoramento (FPS, Telemetria e Rede)
    cmd!(CAT_6, NOTE_6, SUB_6_1, "toggle cl_hud_telemetry_frametime_show 0 1 2", "Exibe o contador de FPS e o tempo de quadro (Frametime em ms) no canto superior direito."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "toggle cl_hud_telemetry_ping_show 0 1 2", "Exibe o seu Ping (ms) em tempo real."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "toggle cl_hud_telemetry_net_misdelivery_show 0 1 2", "Exibe alertas e porcentagem de perda ou atraso de pacotes (packet loss / jitter)."),
    cmd!(CAT_6, NOTE_6, SUB_6_1, "toggle cl_hud_telemetry_serverrecvmargin_graph_show 0 1 2", "Mostra um micrográfico em tempo real da estabilidade de envio ao servidor."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "toggle cq_netgraph 0 1", "Liga/desliga a barra gráfica de Command Queue (sub-tick) no canto superior direito."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "toggle cl_showfps 0 1 2 3 4", "Contador direto de FPS da engine (1 = simples, 2 a 4 = latência detalhada)."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "toggle cl_showpos 0 1", "Mostra coordenadas (pos), ângulo (ang) e velocidade horizontal (vel)."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "+showscores", "Abre a tabela de pontuação / Placar (TAB)."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "status", "Imprime no console dados do servidor atual, mapa, ping e jogadores conectados."),
    cmd!(CAT_6, NOTE_6, SUB_6_2, "sys_info", "Gera no console um relatório completo de hardware, drivers e configurações ativas."),

    // 7. Outros (Áudio, Demos/Replays, HUD e Sistema)
    cmd!(CAT_7, NOTE_7, SUB_7_1, "incrementvar volume 0 1 0.05", "Aumenta o volume geral do jogo em passos de 5%."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "incrementvar volume 0 1 -0.05", "Diminui o volume geral do jogo em passos de 5%."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "toggle volume 0.08 0.55", "Alterna entre Modo Aquecimento/DM (0.08) e Modo Competitivo (0.55)."),
    cmd!(CAT_7, NOTE_7, SUB_7_1, "toggle snd_mute_losefocus 0 1", "Define se o áudio fica mudo ou continua tocando quando você dá Alt+Tab."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demoui", "Abre ou fecha a barra de controle de reprodução de Demo (Shift+F2)."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_togglepause", "Pausa ou retoma a reprodução da Demo instantaneamente."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "toggle demo_timescale 0.25 0.5 1 2 4 8", "Alterna a velocidade do replay entre 0.25x e 8x."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_goto -960 1 0", "Volta aproximadamente 15 segundos na linha do tempo da Demo."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "demo_goto 960 1 0", "Avança aproximadamente 15 segundos na linha do tempo da Demo."),
    cmd!(CAT_7, NOTE_7, SUB_7_2, "toggle spec_show_xray 0 1", "Liga ou desliga o Raio-X (X-Ray) durante análise do replay ou modo espectador."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "incrementvar cl_hud_color 0 11 1", "Troca ciclicamente a cor de toda a interface (HUD) entre as 12 cores disponíveis."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "toggle cl_draw_only_deathnotices 0 1", "Esconde HUD e radar deixando apenas a mira e o registro de abates (killfeed)."),
    cmd!(CAT_7, NOTE_7, SUB_7_3, "toggle cl_drawhud 0 1", "Oculta absolutamente toda a interface da tela (exige sv_cheats 1)."),
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
    cmd!(CAT_7, NOTE_7, SUB_7_4, "unbind \"TECLA\"" => "unbind v", "Remove o comando atribuído a uma tecla específica (gravado sem aspas internas)."),
    cmd!(CAT_7, NOTE_7, SUB_7_4, "binddefaults", "Restaura todas as teclas para a configuração original de fábrica da Valve."),
];

/// Retorna o catálogo completo de binds do CS2 na ordem exata de `guia_completo_de_binds_no_cs2.md`.
pub fn cs2_bind_catalog() -> &'static [BindCommandEntry] {
    CS2_BIND_CATALOG
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

    #[test]
    fn test_catalog_count_and_exact_category_order() {
        let catalog = cs2_bind_catalog();
        assert_eq!(catalog.len(), 186);

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
}
