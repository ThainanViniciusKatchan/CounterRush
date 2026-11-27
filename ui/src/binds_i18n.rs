use counterrush_core::bind_catalog::normalize_search_text;
use counterrush_core::{BindCategoryInfo, BindCommandEntry};

use crate::i18n::UiLanguage;

pub struct LocalizedCategoryStrings {
    pub tab_label: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
}

pub fn localize_category(cat: &BindCategoryInfo, lang: UiLanguage) -> LocalizedCategoryStrings {
    match (lang, cat.index) {
        (UiLanguage::EnUs, 0) => LocalizedCategoryStrings {
            tab_label: "Movement",
            title: "1. Movement",
            subtitle: "Strafing, jumping, stance, and camera control (Source 2)",
        },
        (UiLanguage::EnUs, 1) => LocalizedCategoryStrings {
            tab_label: "Training",
            title: "2. Training",
            subtitle: "Requires sv_cheats 1",
        },
        (UiLanguage::EnUs, 2) => LocalizedCategoryStrings {
            tab_label: "Gameplay",
            title: "3. Gameplay",
            subtitle: "Allowed in competitive matches",
        },
        (UiLanguage::EnUs, 3) => LocalizedCategoryStrings {
            tab_label: "Comms",
            title: "4. Communication",
            subtitle: "Voice, clutch binds, 3D pings, wheels, and radio",
        },
        (UiLanguage::EnUs, 4) => LocalizedCategoryStrings {
            tab_label: "Weapons & Nades",
            title: "5. Weapons and Grenades",
            subtitle: "Slot selection and Source 2 quick buy",
        },
        (UiLanguage::EnUs, 5) => LocalizedCategoryStrings {
            tab_label: "Monitoring",
            title: "6. Monitoring (FPS, Telemetry & Network)",
            subtitle: "Native CS2 HUD telemetry and diagnostics",
        },
        (UiLanguage::EnUs, 6) => LocalizedCategoryStrings {
            tab_label: "Other",
            title: "7. Other (Audio, Demos/Replays, HUD & System)",
            subtitle: "Sound, GOTV demo analysis, HUD, and maintenance",
        },

        (UiLanguage::RuRu, 0) => LocalizedCategoryStrings {
            tab_label: "Движение",
            title: "1. Передвижение",
            subtitle: "Движение, прыжки, приседание и управление камерой (Source 2)",
        },
        (UiLanguage::RuRu, 1) => LocalizedCategoryStrings {
            tab_label: "Тренировка",
            title: "2. Тренировка",
            subtitle: "Требуется sv_cheats 1",
        },
        (UiLanguage::RuRu, 2) => LocalizedCategoryStrings {
            tab_label: "Геймплей",
            title: "3. Геймплей",
            subtitle: "Разрешены в соревновательных матчах",
        },
        (UiLanguage::RuRu, 3) => LocalizedCategoryStrings {
            tab_label: "Связь",
            title: "4. Общение",
            subtitle: "Голос, клатч-бинды, 3D-пинг, колёса и радио",
        },
        (UiLanguage::RuRu, 4) => LocalizedCategoryStrings {
            tab_label: "Оружие",
            title: "5. Оружие и гранаты",
            subtitle: "Выбор по слотам и быстрая закупка Source 2",
        },
        (UiLanguage::RuRu, 5) => LocalizedCategoryStrings {
            tab_label: "Мониторинг",
            title: "6. Мониторинг (FPS, телеметрия и сеть)",
            subtitle: "Встроенная телеметрия HUD CS2 и диагностика",
        },
        (UiLanguage::RuRu, 6) => LocalizedCategoryStrings {
            tab_label: "Прочее",
            title: "7. Прочее (Аудио, демки/повторы, HUD и система)",
            subtitle: "Звук, анализ демок GOTV, HUD и обслуживание",
        },

        _ => LocalizedCategoryStrings {
            tab_label: cat.tab_label,
            title: cat.title,
            subtitle: cat.subtitle,
        },
    }
}

pub fn localize_category_header(entry: &BindCommandEntry, lang: UiLanguage) -> (&'static str, &'static str) {
    let idx = match entry.category {
        "1. Movimentação" => 0,
        "2. Treinamento" => 1,
        "3. Jogabilidade" => 2,
        "4. Comunicação" => 3,
        "5. Armas e Granadas" => 4,
        "6. Monitoramento (FPS, Telemetria e Rede)" => 5,
        "7. Outros (Áudio, Demos/Replays, HUD e Sistema)" => 6,
        _ => return (entry.category, entry.category_note),
    };
    let dummy = BindCategoryInfo {
        index: idx,
        title: entry.category,
        tab_label: entry.category,
        subtitle: entry.category_note,
        command_count: 0,
    };
    let loc = localize_category(&dummy, lang);
    (loc.title, loc.subtitle)
}

pub fn localize_subcategory(sub: &'static str, lang: UiLanguage) -> &'static str {
    match (lang, sub) {
        (UiLanguage::EnUs, "Pré-requisito de Treino (sv_cheats)") => {
            "Training Prerequisite (sv_cheats)"
        }
        (UiLanguage::EnUs, "Comandos Principais") => "Core Commands",
        (UiLanguage::EnUs, "Movimentação Livre e Teleporte de Precisão") => {
            "Free Movement & Precision Teleport"
        }
        (UiLanguage::EnUs, "Treino Avançado de Granadas") => "Advanced Grenade Practice",
        (UiLanguage::EnUs, "Controle de Bots") => "Bot Control",
        (UiLanguage::EnUs, "Combate, Hitboxes e Partida") => "Combat, Hitboxes & Match",
        (UiLanguage::EnUs, "Interação, Visão e Bomba (C4)") => "Interaction, View & Bomb (C4)",
        (UiLanguage::EnUs, "Radar Inteligente") => "Smart Radar",
        (UiLanguage::EnUs, "Mira Dinâmica e Alinhamento (Crosshair)") => {
            "Dynamic Crosshair & Alignment"
        }
        (UiLanguage::EnUs, "Microfone e \"Clutch Binds\" (Voz)") => "Microphone & Clutch Binds (Voice)",
        (UiLanguage::EnUs, "Pings e Rodas de Chat (Chat Wheels)") => "Pings & Chat Wheels",
        (UiLanguage::EnUs, "Menus de Rádio e Falas Diretas de Personagem") => {
            "Radio Menus & Direct Character Voice Lines"
        }
        (UiLanguage::EnUs, "Chat de Texto") => "Text Chat",
        (UiLanguage::EnUs, "Slots de Armas e Equipamentos") => "Weapon & Equipment Slots",
        (UiLanguage::EnUs, "Seleção Direta de Granadas (Altamente Recomendado)") => {
            "Direct Grenade Selection (Highly Recommended)"
        }
        (UiLanguage::EnUs, "Comandos de Compra Rápida (buy e Reembolso)") => {
            "Quick Buy Commands (buy & Refund)"
        }
        (UiLanguage::EnUs, "Telemetria Nativa do CS2 (0 = Nunca, 1 = Problema, 2 = Sempre)") => {
            "Native CS2 Telemetry (0 = Never, 1 = On Issue, 2 = Always)"
        }
        (UiLanguage::EnUs, "Diagnósticos Clássicos e Placar") => {
            "Classic Diagnostics & Scoreboard"
        }
        (UiLanguage::EnUs, "Controle de Volume e Áudio") => "Volume & Audio Control",
        (UiLanguage::EnUs, "Replays e Análise de Demos (GOTV / Suas Partidas)") => {
            "Replays & Demo Analysis (GOTV / Your Matches)"
        }
        (UiLanguage::EnUs, "Personalização de HUD e Capturas") => {
            "HUD Customization & Screenshots"
        }
        (UiLanguage::EnUs, "Console, Sistema e Gerenciamento de Binds") => {
            "Console, System & Bind Management"
        }

        (UiLanguage::RuRu, "Pré-requisito de Treino (sv_cheats)") => {
            "Условие тренировки (sv_cheats)"
        }
        (UiLanguage::RuRu, "Comandos Principais") => "Основные команды",
        (UiLanguage::RuRu, "Movimentação Livre e Teleporte de Precisão") => {
            "Свободный полёт и точный телепорт"
        }
        (UiLanguage::RuRu, "Treino Avançado de Granadas") => "Продвинутая тренировка гранат",
        (UiLanguage::RuRu, "Controle de Bots") => "Управление ботами",
        (UiLanguage::RuRu, "Combate, Hitboxes e Partida") => "Бой, хитбоксы и матч",
        (UiLanguage::RuRu, "Interação, Visão e Bomba (C4)") => "Взаимодействие, обзор и бомба (C4)",
        (UiLanguage::RuRu, "Radar Inteligente") => "Умный радар",
        (UiLanguage::RuRu, "Mira Dinâmica e Alinhamento (Crosshair)") => {
            "Динамический прицел и выравнивание (Crosshair)"
        }
        (UiLanguage::RuRu, "Microfone e \"Clutch Binds\" (Voz)") => {
            "Микрофон и клатч-бинды (Голос)"
        }
        (UiLanguage::RuRu, "Pings e Rodas de Chat (Chat Wheels)") => "Пинги и колёса чата (Chat Wheels)",
        (UiLanguage::RuRu, "Menus de Rádio e Falas Diretas de Personagem") => {
            "Радиоменю и прямые реплики персонажа"
        }
        (UiLanguage::RuRu, "Chat de Texto") => "Текстовый чат",
        (UiLanguage::RuRu, "Slots de Armas e Equipamentos") => "Слоты оружия и снаряжения",
        (UiLanguage::RuRu, "Seleção Direta de Granadas (Altamente Recomendado)") => {
            "Прямой выбор гранат (настоятельно рекомендуется)"
        }
        (UiLanguage::RuRu, "Comandos de Compra Rápida (buy e Reembolso)") => {
            "Команды быстрой закупки (buy и возврат)"
        }
        (UiLanguage::RuRu, "Telemetria Nativa do CS2 (0 = Nunca, 1 = Problema, 2 = Sempre)") => {
            "Встроенная телеметрия CS2 (0 = Никогда, 1 = При проблемах, 2 = Всегда)"
        }
        (UiLanguage::RuRu, "Diagnósticos Clássicos e Placar") => {
            "Классическая диагностика и таблица счёта"
        }
        (UiLanguage::RuRu, "Controle de Volume e Áudio") => "Управление громкостью и звуком",
        (UiLanguage::RuRu, "Replays e Análise de Demos (GOTV / Suas Partidas)") => {
            "Повторы и анализ демок (GOTV / Ваши матчи)"
        }
        (UiLanguage::RuRu, "Personalização de HUD e Capturas") => "Настройка HUD и скриншоты",
        (UiLanguage::RuRu, "Console, Sistema e Gerenciamento de Binds") => {
            "Консоль, система и управление биндами"
        }

        _ => sub,
    }
}

pub fn localize_featured_subcategory_note(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::PtBr => "Obrigatório ativar (1) antes de usar os comandos de treino abaixo",
        UiLanguage::EnUs => "Must be enabled (1) before using the training commands below",
        UiLanguage::RuRu => "Необходимо включить (1) перед использованием команд тренировки ниже",
    }
}

pub fn localize_featured_badge(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::PtBr => "ESSENCIAL",
        UiLanguage::EnUs => "ESSENTIAL",
        UiLanguage::RuRu => "ВАЖНО",
    }
}

/// Tabela 1:1 com `CS2_BIND_CATALOG` (237 itens): (cfg_command, descrição EN, descrição RU).
pub const CATALOG_TRANSLATIONS: [(&str, &str, &str); 237] = [
    // 1. Movimentação (15)
    ("+forward", "Move forward (W).", "Движение вперёд (W)."),
    ("+back", "Move backward (S).", "Движение назад (S)."),
    ("+left", "Strafe left (A).", "Движение боком / стрейф влево (A)."),
    ("+right", "Strafe right (D).", "Движение боком / стрейф вправо (D)."),
    ("+jump", "Perform a jump (commonly bound to mouse wheel for easier Bunnyhopping).", "Прыжок (часто ставится на колесо мыши для удобного распрыга / Bunnyhop)."),
    ("+duck", "Crouch character (CTRL).", "Приседание (CTRL)."),
    ("+sprint", "Walk slowly without emitting footstep sounds (SHIFT).", "Медленный шаг без звука шагов (SHIFT)."),
    ("option_duck_method 0", "Hold-to-crouch mode (0): keeps the character crouched only while the key is held.", "Приседание по удержанию (0): персонаж остаётся в приседе только пока клавиша нажата."),
    ("option_duck_method 1", "Toggle-to-crouch mode (1): press once to crouch and press again to stand up.", "Приседание по переключению (1): одно нажатие — присесть, повторное — встать."),
    ("option_speed_method 0", "Hold-to-walk mode (0): walks silently only while the key is held.", "Тихий шаг по удержанию (0): медленная ходьба только пока клавиша нажата."),
    ("option_speed_method 1", "Toggle-to-walk mode (1): press once to enable silent walk and again to disable.", "Тихий шаг по переключению (1): одно нажатие включает тихий шаг, повторное отключает."),
    ("+turnleft", "Turn camera horizontally to the left (Surf, KZ, and Bhop).", "Плавный поворот камеры влево (Surf, KZ и Bhop)."),
    ("+turnright", "Turn camera horizontally to the right (Surf, KZ, and Bhop).", "Плавный поворот камеры вправо (Surf, KZ и Bhop)."),
    ("+lookup", "Pitch view angle vertically upward via keyboard.", "Поднимает угол обзора вверх с клавиатуры."),
    ("+lookdown", "Pitch view angle vertically downward via keyboard.", "Опускает угол обзора вниз с клавиатуры."),

    // 2. Treinamento (37)
    ("sv_cheats 0", "Disable cheats on local server (0): locks training commands and restores standard match rules.", "Отключает читы на локальном сервере (0): блокирует команды тренировки и возвращает обычные правила матча."),
    ("sv_cheats 1", "Enable cheats on local server (1): unlocks all training commands below (noclip, infinite ammo, grenades, and bots).", "Включает читы на локальном сервере (1): открывает доступ ко всем командам тренировки ниже (noclip, патроны, гранаты и боты)."),
    ("noclip", "Toggle free flight passing through walls and objects.", "Включает/отключает свободный полёт сквозь стены и объекты."),
    ("getpos", "Print exact coordinates (setpos and setang) of current position and aim in console.", "Выводит в консоль точные координаты (setpos и setang) текущей позиции и прицела."),
    ("host_timescale 0.5", "Server speed 0.5x (slow motion): slows time by half to inspect movement and trajectories.", "Скорость сервера 0.5x (замедление): замедляет время вдвое для анализа движений и траекторий."),
    ("host_timescale 1", "Server speed 1x (normal): restores standard match time speed.", "Скорость сервера 1x (обычная): восстанавливает стандартную скорость времени матча."),
    ("host_timescale 2", "Server speed 2x (fast-forward): doubles time speed to advance quickly during practice.", "Скорость сервера 2x (ускорение): удваивает скорость времени для быстрой перемотки тренировки."),
    ("sv_rethrow_last_grenade", "Rethrow the last grenade thrown on the server.", "Повторно бросает последнюю брошенную на сервере гранату."),
    ("sv_grenade_trajectory_prac_pipreview 0", "Disable Picture-in-Picture preview (0): closes the grenade landing preview window.", "Отключает окно предпросмотра (0): закрывает мини-окно с точкой падения гранаты."),
    ("sv_grenade_trajectory_prac_pipreview 1", "Enable Picture-in-Picture preview (1): opens a live window showing where your grenade will land.", "Включает окно предпросмотра (1): открывает мини-окно, показывающее место падения гранаты в реальном времени."),
    ("cl_sim_grenade_trajectory", "Freeze the current trajectory line of the grenade in your hand for a few seconds.", "Фиксирует линию траектории текущей гранаты в руке на несколько секунд."),
    ("ent_fire smokegrenade_projectile kill; ent_fire molotov_projectile kill; ent_fire inferno kill; ent_fire flashbang_projectile kill; ent_fire hegrenade_projectile kill; ent_fire decoy_projectile kill; stopsound", "Instantly clear all active smokes, Molotovs, mid-air grenades, and stop active sound effects.", "Мгновенно удаляет все дымы, Молотовы, летящие гранаты и останавливает активные звуки."),
    ("bot_place", "Place a bot immediately at the spot you are aiming at.", "Мгновенно размещает бота в точке, куда направлен прицел."),
    ("bot_add_ct", "Add a bot to the Counter-Terrorist (CT) side.", "Добавляет бота за сторону спецназа (CT)."),
    ("bot_add_t", "Add a bot to the Terrorist (T) side.", "Добавляет бота за сторону террористов (T)."),
    ("bot_kick", "Remove all bots from the match.", "Удаляет всех ботов из матча."),
    ("bot_crouch 0", "Bots standing (0): disables forced crouching and makes all bots stand up.", "Боты стоят (0): отключает принудительный присед и поднимает всех ботов."),
    ("bot_crouch 1", "Bots crouching (1): forces all bots to stay crouched.", "Боты в приседе (1): заставляет всех ботов постоянно сидеть на корточках."),
    ("bot_stop 0", "Unfreeze bots (0): allows bots to move around normally again.", "Разморозить ботов (0): разрешает ботам снова свободно передвигаться."),
    ("bot_stop 1", "Freeze bots (1): locks all bots in place for aim practice.", "Заморозить ботов (1): фиксирует всех ботов на месте для тренировки стрельбы."),
    ("bot_mimic 0", "Disable bot mimic (0): bots stop mirroring your movements and shots.", "Отключить повторение ботов (0): боты перестают копировать ваши движения и выстрелы."),
    ("bot_mimic 1", "Enable bot mimic (1): makes bots mirror all your movements and shots in real time.", "Включить повторение ботов (1): заставляет ботов повторять все ваши движения и выстрелы в реальном времени."),
    ("bot_dont_shoot 0", "Allow bot shooting (0): bots resume firing normally.", "Разрешить стрельбу ботам (0): боты снова начинают стрелять как обычно."),
    ("bot_dont_shoot 1", "Prevent bot shooting (1): makes bots peaceful so they will not shoot you.", "Запретить стрельбу ботам (1): боты становятся мирными и не стреляют по вам."),
    ("sv_showimpacts 0", "Hide bullet impacts (0): disables colored bullet impact boxes.", "Скрыть попадания пуль (0): отключает цветные квадраты регистрации выстрелов."),
    ("sv_showimpacts 1", "Client + server impacts (1): shows red (client) and blue (server) boxes where bullets hit.", "Попадания клиент + сервер (1): показывает красные (клиент) и синие (сервер) квадраты попаданий."),
    ("sv_showimpacts 2", "Client-only impacts (2): shows only the red boxes registered locally by the client.", "Попадания только клиент (2): показывает только красные квадраты попаданий со стороны клиента."),
    ("sv_infinite_ammo 0", "Standard ammo (0): disables infinite ammo and restores normal bullet and grenade consumption.", "Обычные патроны (0): отключает бесконечный боезапас и возвращает стандартный расход патронов и гранат."),
    ("sv_infinite_ammo 1", "Infinite ammo without reload (1): magazine bullets and grenades never run out.", "Бесконечные патроны без перезарядки (1): патроны в магазине и гранаты никогда не заканчиваются."),
    ("sv_infinite_ammo 2", "Infinite reserve ammo (2): requires reloading your weapon, but reserve ammo and grenades never run out.", "Бесконечный запас патронов (2): требует перезарядки оружия, но запас патронов и гранаты бесконечны."),
    ("god", "Enable full invincibility.", "Включает полную неуязвимость."),
    ("buddha", "Take damage down to 1 HP without ever dying.", "Позволяет получать урон до 1 HP без смерти."),
    ("cl_ent_hitbox", "Display the hitboxes of the model you are aiming at.", "Отображает хитбоксы модели, на которую вы смотрите."),
    ("r_drawothermodels 1", "Normal player models (1): restores standard solid character rendering.", "Обычные модели игроков (1): возвращает стандартную сплошную отрисовку персонажей."),
    ("r_drawothermodels 2", "Wireframe player models (2): displays player skeletons visible through walls.", "Каркасные модели игроков (2): отображает каркас персонажей сквозь стены."),
    ("impulse 101", "Instantly grant maximum money.", "Мгновенно выдаёт максимальное количество денег."),
    ("mp_restartgame 1", "Restart the map/match in 1 second.", "Перезапускает карту/матч через 1 секунду."),

    // 3. Jogabilidade (29)
    ("+attack", "Primary fire (MOUSE1).", "Основная стрельба (MOUSE1)."),
    ("+attack2", "Secondary action such as scope zoom, silencer, or underhand throw (MOUSE2).", "Вторичное действие: зум, глушитель или ближний бросок гранаты (MOUSE2)."),
    ("+reload", "Reload weapon (R).", "Перезарядка оружия (R)."),
    ("+use", "Interact, open doors, pick up weapons, and defuse C4 (E).", "Взаимодействие, открытие дверей, подбор оружия и обезвреживание C4 (E)."),
    ("switchhands", "Instantly switch weapon between right and left hand.", "Мгновенно переключает оружие между правой и левой рукой."),
    ("+lookatweapon", "Inspect the equipped weapon/knife (F).", "Осмотр текущего оружия/ножа (F)."),
    ("+spray_menu", "Open the graffiti spray menu (T).", "Открывает меню нанесения граффити (T)."),
    ("slot5; drop", "Quickly equip and drop the C4.", "Быстро выбирает и выбрасывает бомбу C4."),
    ("incrementvar cl_radar_scale 0.25 1.0 0.15", "Step up minimap zoom and wrap back to minimum after reaching the limit.", "Пошагово увеличивает масштаб радара и сбрасывает к минимуму после предела."),
    ("cl_radar_scale 0.35", "Panoramic radar (0.35): zooms out the minimap to see almost the entire map.", "Обзорный радар (0.35): отдаляет масштаб мини-карты, показывая почти всю карту."),
    ("cl_radar_scale 0.75", "Zoomed-in radar (0.75): zooms in the minimap for detailed focus on the bombsite.", "Приближённый радар (0.75): приближает мини-карту с фокусом на пленте и ближайших зонах."),
    ("cl_radar_always_centered 0", "Uncentered radar (0): unlocks minimap camera to use the full radar area.", "Радар без центрирования (0): освобождает камеру мини-карты для обзора всей площади карты."),
    ("cl_radar_always_centered 1", "Player-centered radar (1): keeps your character locked at the center of the minimap.", "Радар по центру игрока (1): всегда удерживает вашего персонажа в центре мини-карты."),
    ("cl_radar_rotate 0", "Fixed radar (0): locks minimap orientation instead of rotating with your view.", "Фиксированный радар (0): закрепляет ориентацию мини-карты без поворота за обзором."),
    ("cl_radar_rotate 1", "Rotating radar (1): rotates the minimap to match the direction you are aiming.", "Вращающийся радар (1): поворачивает мини-карту вслед за направлением вашего взгляда."),
    ("cl_crosshair_recoil 0", "Disable Follow Recoil (0): keeps the crosshair fixed at screen center without tracking recoil.", "Отключить Follow Recoil (0): оставляет прицел неподвижным в центре экрана без слежения за отдачей."),
    ("cl_crosshair_recoil 1", "Enable Follow Recoil (1): makes the crosshair dynamically follow your weapon's recoil pattern.", "Включить Follow Recoil (1): прицел динамически следует за отдачей (спреем) оружия."),
    ("cl_crosshairsize 2", "Standard compact crosshair (size 2): sets normal crosshair size for combat.", "Стандартный компактный прицел (размер 2): устанавливает обычный размер прицела для боя."),
    ("cl_crosshairsize 1000", "Smoke lineup crosshair (size 1000): extends lines to screen edges for utility lineups.", "Прицел для раскидок (размер 1000): растягивает линии до краёв экрана для точных гранат."),
    ("cl_crosshaircolor 1", "Green crosshair color (1): applies standard green color to your crosshair.", "Зелёный цвет прицела (1): устанавливает стандартный зелёный цвет перекрестия."),
    ("cl_crosshaircolor 2", "Yellow crosshair color (2): applies yellow color to your crosshair.", "Жёлтый цвет прицела (2): устанавливает жёлтый цвет перекрестия."),
    ("cl_crosshaircolor 4", "Cyan crosshair color (4): applies cyan color to your crosshair.", "Голубой цвет прицела (4): устанавливает голубой цвет перекрестия."),
    ("cl_crosshaircolor 5", "Custom crosshair color (5): applies your custom RGB color values.", "Пользовательский цвет прицела (5): применяет ваш собственный RGB-цвет перекрестия."),
    ("cl_crosshairdot 0", "Disable center crosshair dot (0): removes the dot in the middle of the crosshair.", "Отключить центральную точку прицела (0): убирает точку в середине перекрестия."),
    ("cl_crosshairdot 1", "Enable center crosshair dot (1): displays a fixed dot in the center of the crosshair.", "Включить центральную точку прицела (1): отображает фиксированную точку в центре перекрестия."),
    ("crosshair 0", "Hide crosshair (0): completely hides the crosshair from the screen.", "Скрыть прицел (0): полностью убирает прицел с экрана."),
    ("crosshair 1", "Show crosshair (1): makes the crosshair visible on screen again.", "Показать прицел (1): снова делает прицел видимым на экране."),
    ("cl_grenadecrosshair_keepusercrosshair 0", "Hide standard crosshair on grenades (0): shows only the grenade lineup reticle.", "Скрыть обычный прицел при гранатах (0): показывает только сетку наведения гранаты."),
    ("cl_grenadecrosshair_keepusercrosshair 1", "Keep standard crosshair on grenades (1): keeps your normal crosshair visible with the grenade reticle.", "Оставить обычный прицел при гранатах (1): сохраняет видимость основного прицела вместе с сеткой гранаты."),

    // 4. Comunicação (34)
    ("+voicerecord", "Activate microphone (Push-to-Talk).", "Активирует микрофон (Push-to-Talk)."),
    ("voice_modenable_toggle", "Mute or unmute all team voice communication in one click.", "Одним нажатием отключает или включает голосовой чат команды."),
    ("snd_voipvolume 0.08", "Lower team voice / clutch volume (0.08): reduces teammate voices to 8% during clutch situations.", "Тихий голос команды / клатч (0.08): снижает громкость союзников до 8% во время клатчей."),
    ("snd_voipvolume 1.0", "Normal team voice volume (1.0): restores teammate voice volume to 100%.", "Обычный голос команды (1.0): восстанавливает громкость голосов союзников на 100%."),
    ("voice_toggle_open_mic", "Toggle between open mic and Push-to-Talk.", "Переключает между открытым микрофоном и Push-to-Talk."),
    ("player_ping", "Place a 3D visual marker (ping) exactly where you are aiming.", "Ставит 3D-метку (пинг) точно в точку, куда направлен прицел."),
    ("+radialradio", "Hold open Communication Wheel 1.", "Удерживает открытым колесо общения 1."),
    ("+radialradio2", "Hold open Communication Wheel 2.", "Удерживает открытым колесо общения 2."),
    ("+radialradio3", "Hold open Communication Wheel 3.", "Удерживает открытым колесо общения 3."),
    ("radio", "Open the classic main radio command menu on screen.", "Открывает классическое главное меню радиокоманд на экране."),
    ("radio1", "Open classic radio command menu 1.", "Открывает классическое меню радиокоманд 1."),
    ("radio2", "Open classic radio command menu 2.", "Открывает классическое меню радиокоманд 2."),
    ("radio3", "Open classic radio command menu 3.", "Открывает классическое меню радиокоманд 3."),
    ("coverme", "Direct tactical line: \"Cover me!\".", "Тактическая реплика: «Прикрой меня!»."),
    ("takepoint", "Direct tactical line: \"You take the point\".", "Тактическая реплика: «Иди первым»."),
    ("holdpos", "Direct tactical line: \"Hold this position\".", "Тактическая реплика: «Держите позицию»."),
    ("followme", "Direct tactical line: \"Follow me\".", "Тактическая реплика: «За мной»."),
    ("regroup", "Direct tactical line: \"Regroup\".", "Тактическая реплика: «Перегруппироваться»."),
    ("fallback", "Direct tactical line: \"Fall back!\".", "Тактическая реплика: «Отступаем!»."),
    ("sticktog", "Direct tactical line: \"Stick together\".", "Тактическая реплика: «Держитесь вместе»."),
    ("go", "Direct tactical line: \"Go, go, go!\".", "Тактическая реплика: «Вперёд, вперёд!»."),
    ("enemyspot", "Direct info line: \"Enemy spotted\".", "Инфо-реплика: «Вижу врага»."),
    ("needbackup", "Direct info line: \"Need backup\".", "Инфо-реплика: «Нужна поддержка»."),
    ("sectorclear", "Direct info line: \"Sector clear\".", "Инфо-реплика: «Сектор чист»."),
    ("inposition", "Direct info line: \"In position\".", "Инфо-реплика: «На позиции»."),
    ("enemydown", "Direct info line: \"Enemy down\".", "Инфо-реплика: «Враг уничтожен»."),
    ("getout", "Direct info line: \"Get out of there, it's gonna blow!\".", "Инфо-реплика: «Уходите, сейчас рванёт!»."),
    ("thanks", "Direct social line: \"Thanks\".", "Социальная реплика: «Спасибо»."),
    ("compliment", "Direct social line: \"Nice!\".", "Социальная реплика: «Отлично!»."),
    ("cheer", "Direct social line: Cheer.", "Социальная реплика: Подбодрить."),
    ("messagemode", "Open text chat input for Everyone (default Y).", "Открывает строку текстового чата для всех (по умолчанию Y)."),
    ("messagemode2", "Open text chat input for Team only (default U).", "Открывает строку текстового чата только для команды (по умолчанию U)."),
    ("say GGWP!", "Instantly send a phrase in global chat (saved without inner quotes).", "Мгновенно отправляет фразу в общий чат (записывается без внутренних кавычек)."),
    ("say_team Preciso de drop, por favor!", "Instantly send a phrase to your team only (saved without inner quotes).", "Мгновенно отправляет фразу только команде (записывается без внутренних кавычек)."),

    // 5. Armas e Granadas (64)
    ("slot1", "Primary weapon (Rifles, SMGs, Shotguns, Machine guns).", "Основное оружие (винтовки, ПП, дробовики, пулемёты)."),
    ("slot2", "Secondary weapon (Pistols).", "Вторичное оружие (пистолеты)."),
    ("slot3", "Knife (or Zeus x27 if knife is already equipped).", "Нож (или Zeus x27, если нож уже в руках)."),
    ("slot4", "Cycle sequentially through all grenades in inventory.", "Последовательно перебирает все гранаты в инвентаре."),
    ("slot5", "Directly equip the C4 explosive.", "Напрямую выбирает взрывчатку C4."),
    ("slot11", "Directly equip the Zeus x27 without pressing knife twice.", "Напрямую выбирает Zeus x27 без двойного нажатия клавиши ножа."),
    ("slot12", "Equip the Medi-Shot / Healthshot (Deathmatch / Practice modes).", "Выбирает мед-шприц Healthshot (в режимах Бой насмерть / Тренировка)."),
    ("lastinv", "Switch to the last equipped item (default Q).", "Переключает на предыдущий выбранный предмет (по умолчанию Q)."),
    ("invnext", "Select the next weapon in inventory.", "Выбирает следующее оружие в инвентаре."),
    ("invprev", "Select the previous weapon in inventory.", "Выбирает предыдущее оружие в инвентаре."),
    ("drop", "Drop the currently held weapon or utility (default G).", "Выбрасывает текущее оружие или гранату на землю (по умолчанию G)."),
    ("slot6", "HE Grenade (Explosive).", "Осколочная граната HE."),
    ("slot7", "Flashbang.", "Светошумовая граната (Flashbang)."),
    ("slot8", "Smoke Grenade.", "Дымовая граната (Smoke)."),
    ("slot9", "Decoy Grenade.", "Ложная граната (Decoy)."),
    ("slot10", "Molotov / Incendiary Grenade.", "Коктейль Молотова / Зажигательная граната."),
    ("autobuy", "Auto-buy loadout (default F3).", "Автоматическая закупка (по умолчанию F3)."),
    ("rebuy", "Rebuy previous round loadout (default F4).", "Повторная закупка прошлого раунда (по умолчанию F4)."),
    ("sellbackall", "Instantly refund all purchases made during the current freeze time.", "Мгновенно возвращает деньги за все покупки текущего фризтайма."),
    ("buy vest", "Buy Kevlar Vest.", "Купить бронежилет (Kevlar)."),
    ("buy vesthelm", "Buy Kevlar + Helmet.", "Купить бронежилет + шлем."),
    ("buy defuser", "Buy Defuse Kit (CT).", "Купить набор сапёра (CT)."),
    ("buy taser", "Buy Zeus x27.", "Купить Zeus x27."),
    ("buy smokegrenade", "Buy Smoke Grenade.", "Купить дымовую гранату (Smoke)."),
    ("buy flashbang", "Buy Flashbang.", "Купить светошумовую гранату (Flashbang)."),
    ("buy hegrenade", "Buy HE Grenade.", "Купить осколочную гранату (HE)."),
    ("buy molotov; buy incgrenade", "Buy Molotov (T) or Incendiary Grenade (CT).", "Купить Молотов (T) или зажигательную гранату (CT)."),
    ("buy decoy", "Buy Decoy Grenade.", "Купить ложную гранату (Decoy)."),
    ("buy glock; buy hkp2000; buy usp_silencer", "Buy starter pistol (Glock / P2000 / USP-S).", "Купить стартовый пистолет (Glock / P2000 / USP-S)."),
    ("buy p250", "Buy P250.", "Купить P250."),
    ("buy tec9; buy fiveseven; buy cz75a", "Buy Tec-9 / Five-SeveN / CZ75-Auto.", "Купить Tec-9 / Five-SeveN / CZ75-Auto."),
    ("buy deagle; buy revolver", "Buy Desert Eagle / R8 Revolver.", "Купить Desert Eagle / R8 Revolver."),
    ("buy elite", "Buy Dual Berettas.", "Купить Dual Berettas."),
    ("buy galilar; buy famas", "Buy Galil AR (T) or FAMAS (CT).", "Купить Galil AR (T) или FAMAS (CT)."),
    ("buy ak47; buy m4a1; buy m4a1_silencer", "Buy AK-47 (T), M4A4 (m4a1), or M4A1-S (m4a1_silencer).", "Купить AK-47 (T), M4A4 (m4a1) или M4A1-S (m4a1_silencer)."),
    ("buy ssg08", "Buy SSG 08 (Scout).", "Купить SSG 08 (Scout)."),
    ("buy sg556; buy aug", "Buy SG 553 (T) or AUG (CT).", "Купить SG 553 (T) или AUG (CT)."),
    ("buy awp", "Buy AWP.", "Купить AWP."),
    ("buy g3sg1; buy scar20", "Buy G3SG1 (T) or SCAR-20 (CT).", "Купить G3SG1 (T) или SCAR-20 (CT)."),
    ("buy mac10; buy mp9", "Buy MAC-10 (T) or MP9 (CT).", "Купить MAC-10 (T) или MP9 (CT)."),
    ("buy mp7; buy mp5sd", "Buy MP7 / MP5-SD.", "Купить MP7 / MP5-SD."),
    ("buy ump45", "Buy UMP-45.", "Купить UMP-45."),
    ("buy p90", "Buy P90.", "Купить P90."),
    ("buy bizon", "Buy PP-Bizon.", "Купить PP-Bizon."),
    ("buy nova", "Buy Nova.", "Купить Nova."),
    ("buy xm1014", "Buy XM1014.", "Купить XM1014."),
    ("buy mag7; buy sawedoff", "Buy MAG-7 (CT) or Sawed-Off (T).", "Купить MAG-7 (CT) или Sawed-Off (T)."),
    ("buy m249", "Buy M249.", "Купить M249."),
    ("buy negev", "Buy Negev.", "Купить Negev."),
    ("buy secondary0", "Buy pistol from CS2 Loadout slot 0.", "Купить пистолет из слота 0 снаряжения CS2."),
    ("buy secondary1", "Buy pistol from CS2 Loadout slot 1.", "Купить пистолет из слота 1 снаряжения CS2."),
    ("buy secondary2", "Buy pistol from CS2 Loadout slot 2.", "Купить пистолет из слота 2 снаряжения CS2."),
    ("buy secondary3", "Buy pistol from CS2 Loadout slot 3.", "Купить пистолет из слота 3 снаряжения CS2."),
    ("buy secondary4", "Buy pistol from CS2 Loadout slot 4.", "Купить пистолет из слота 4 снаряжения CS2."),
    ("buy midtier0", "Buy mid-tier weapon from CS2 Loadout slot 0.", "Купить среднее оружие (Mid-Tier) из слота 0 снаряжения CS2."),
    ("buy midtier1", "Buy mid-tier weapon from CS2 Loadout slot 1.", "Купить среднее оружие (Mid-Tier) из слота 1 снаряжения CS2."),
    ("buy midtier2", "Buy mid-tier weapon from CS2 Loadout slot 2.", "Купить среднее оружие (Mid-Tier) из слота 2 снаряжения CS2."),
    ("buy midtier3", "Buy mid-tier weapon from CS2 Loadout slot 3.", "Купить среднее оружие (Mid-Tier) из слота 3 снаряжения CS2."),
    ("buy midtier4", "Buy mid-tier weapon from CS2 Loadout slot 4.", "Купить среднее оружие (Mid-Tier) из слота 4 снаряжения CS2."),
    ("buy rifle0", "Buy rifle from CS2 Loadout slot 0.", "Купить винтовку из слота 0 снаряжения CS2."),
    ("buy rifle1", "Buy rifle from CS2 Loadout slot 1.", "Купить винтовку из слота 1 снаряжения CS2."),
    ("buy rifle2", "Buy rifle from CS2 Loadout slot 2.", "Купить винтовку из слота 2 снаряжения CS2."),
    ("buy rifle3", "Buy rifle from CS2 Loadout slot 3.", "Купить винтовку из слота 3 снаряжения CS2."),
    ("buy rifle4", "Buy rifle from CS2 Loadout slot 4.", "Купить винтовку из слота 4 снаряжения CS2."),

    // 6. Monitoramento (24)
    ("cl_hud_telemetry_frametime_show 0", "Hide FPS/Frametime (0 = Never): disables the top-right telemetry indicator.", "Скрыть FPS/Frametime (0 = Никогда): отключает индикатор в правом верхнем углу."),
    ("cl_hud_telemetry_frametime_show 1", "FPS/Frametime on issues (1): shows automatically only during FPS drops or frametime spikes.", "FPS/Frametime при проблемах (1): появляется только при просадках FPS или скачках времени кадра."),
    ("cl_hud_telemetry_frametime_show 2", "Always show FPS/Frametime (2): continuously displays FPS and frame time (ms) in the top-right corner.", "Всегда показывать FPS/Frametime (2): постоянно отображает FPS и время кадра (мс) в правом верхнем углу."),
    ("cl_hud_telemetry_ping_show 0", "Hide Ping (0 = Never): disables the HUD ping indicator.", "Скрыть пинг (0 = Никогда): отключает индикатор задержки в HUD."),
    ("cl_hud_telemetry_ping_show 1", "Ping on issues (1): shows ping only when latency is high or unstable.", "Пинг при проблемах (1): показывает пинг только при высокой или нестабильной задержке."),
    ("cl_hud_telemetry_ping_show 2", "Always show Ping (2): continuously displays your real-time Ping (ms).", "Всегда показывать пинг (2): постоянно отображает ваш пинг (мс) в реальном времени."),
    ("cl_hud_telemetry_net_misdelivery_show 0", "Hide packet loss/jitter (0 = Never): disables network misdelivery alerts in the HUD.", "Скрыть потери/джиттер пакетов (0 = Никогда): отключает сетевые предупреждения в HUD."),
    ("cl_hud_telemetry_net_misdelivery_show 1", "Packet loss/jitter on issues (1): shows alerts only when packet loss or delay occurs.", "Потери/джиттер при проблемах (1): показывает предупреждение только при потере или задержке пакетов."),
    ("cl_hud_telemetry_net_misdelivery_show 2", "Always show packet loss/jitter (2): continuously displays packet loss and jitter percentage.", "Всегда показывать потери/джиттер (2): постоянно отображает процент потерь пакетов и джиттера."),
    ("cl_hud_telemetry_serverrecvmargin_graph_show 0", "Hide server stability graph (0 = Never): disables the server send micro-graph.", "Скрыть график стабильности сервера (0 = Никогда): отключает микрографик отправки данных."),
    ("cl_hud_telemetry_serverrecvmargin_graph_show 1", "Server graph on issues (1): shows the micro-graph only during server send instability.", "График сервера при проблемах (1): показывает микрографик только при нестабильной отправке данных."),
    ("cl_hud_telemetry_serverrecvmargin_graph_show 2", "Always show server stability graph (2): continuously displays the real-time server send micro-graph.", "Всегда показывать график сервера (2): постоянно отображает микрографик стабильности отправки данных на сервер."),
    ("cq_netgraph 0", "Disable Command Queue graph (0): hides the sub-tick graph bar in the top-right corner.", "Отключить график Command Queue (0): скрывает полосу очереди команд (sub-tick) в правом верхнем углу."),
    ("cq_netgraph 1", "Enable Command Queue graph (1): displays the sub-tick Command Queue graph bar in the top-right corner.", "Включить график Command Queue (1): показывает полосу очереди команд (sub-tick) в правом верхнем углу."),
    ("cl_showfps 0", "Hide engine FPS (0): disables the direct engine FPS counter.", "Скрыть FPS движка (0): отключает прямой счётчик FPS."),
    ("cl_showfps 1", "Simple engine FPS (1): displays the basic real-time FPS counter.", "Простой FPS движка (1): отображает базовый счётчик FPS в реальном времени."),
    ("cl_showfps 2", "Engine FPS with frametime (2): displays FPS along with min, avg, and max frame times.", "FPS движка со временем кадра (2): отображает FPS и мин./сред./макс. время кадра."),
    ("cl_showfps 3", "Engine FPS with server ticks (3): displays FPS along with server tick and latency metrics.", "FPS движка с тиками сервера (3): отображает FPS, задержку и данные тиков сервера."),
    ("cl_showfps 4", "Full engine FPS diagnostics (4): displays detailed frame and engine latency diagnostics.", "Полная диагностика FPS движка (4): отображает подробную статистику кадров и задержек движка."),
    ("cl_showpos 0", "Hide position and velocity (0): removes coordinates, angle, and velocity from screen.", "Скрыть позицию и скорость (0): убирает координаты, угол и скорость с экрана."),
    ("cl_showpos 1", "Show position and velocity (1): displays coordinates (pos), angle (ang), and horizontal velocity (vel).", "Показать позицию и скорость (1): отображает координаты (pos), угол (ang) и горизонтальную скорость (vel)."),
    ("+showscores", "Open the scoreboard (TAB).", "Открывает таблицу счёта (TAB)."),
    ("status", "Print current server info, map, ping, and connected players to console.", "Выводит в консоль данные текущего сервера, карту, пинг и список игроков."),
    ("sys_info", "Print a full hardware, driver, and active configuration report to console.", "Выводит в консоль полный отчёт об оборудовании, драйверах и активных настройках."),

    // 7. Outros (34)
    ("incrementvar volume 0 1 0.05", "Increase master game volume in 5% steps.", "Увеличивает общую громкость игры с шагом 5%."),
    ("incrementvar volume 0 1 -0.05", "Decrease master game volume in 5% steps.", "Уменьшает общую громкость игры с шагом 5%."),
    ("volume 0.08", "Warmup/DM volume mode (0.08): lowers master game volume to 8%.", "Громкость разминки/DM (0.08): снижает общую громкость игры до 8%."),
    ("volume 0.55", "Competitive volume mode (0.55): sets master game volume to 55%.", "Соревновательная громкость (0.55): устанавливает общую громкость игры на 55%."),
    ("snd_mute_losefocus 0", "Keep audio on Alt+Tab (0): game sound continues playing in the background when minimized.", "Не отключать звук при Alt+Tab (0): звук игры продолжает работать в фоне при сворачивании."),
    ("snd_mute_losefocus 1", "Mute audio on Alt+Tab (1): automatically mutes the game when the window loses focus.", "Отключать звук при Alt+Tab (1): автоматически заглушает игру при потере фокуса окна."),
    ("demoui", "Open or close the Demo playback control bar (Shift+F2).", "Открывает или закрывает панель управления воспроизведением демки (Shift+F2)."),
    ("demo_togglepause", "Instantly pause or resume Demo playback.", "Мгновенно ставит демку на паузу или возобновляет воспроизведение."),
    ("demo_timescale 0.25", "Demo speed 0.25x: ultra-slow playback for detailed play inspection.", "Скорость демки 0.25x: сверхмедленный просмотр для детального разбора моментов."),
    ("demo_timescale 0.5", "Demo speed 0.5x: half-speed slow motion to review aim and movement.", "Скорость демки 0.5x: замедление вдвое для анализа аима и передвижения."),
    ("demo_timescale 1", "Demo speed 1x (normal): restores original replay playback speed.", "Скорость демки 1x (обычная): восстанавливает стандартную скорость воспроизведения повтора."),
    ("demo_timescale 2", "Demo speed 2x: plays back the replay at double speed.", "Скорость демки 2x: воспроизведение повтора на удвоенной скорости."),
    ("demo_timescale 4", "Demo speed 4x: fast-forwards the replay at 4x speed.", "Скорость демки 4x: быстрая перемотка повтора в 4 раза."),
    ("demo_timescale 8", "Demo speed 8x: ultra-fast replay playback to skip rounds quickly.", "Скорость демки 8x: сверхбыстрое воспроизведение для быстрого пропуска раундов."),
    ("demo_goto -960 1 0", "Rewind approximately 15 seconds on the Demo timeline.", "Перематывает демку примерно на 15 секунд назад."),
    ("demo_goto 960 1 0", "Fast-forward approximately 15 seconds on the Demo timeline.", "Перематывает демку примерно на 15 секунд вперёд."),
    ("spec_show_xray 0", "Disable X-Ray (0): hides player outlines through walls in Demos/spectator mode.", "Отключить рентген X-Ray (0): скрывает силуэты игроков через стены в демках и режиме наблюдателя."),
    ("spec_show_xray 1", "Enable X-Ray (1): highlights player outlines through walls in Demos/spectator mode.", "Включить рентген X-Ray (1): подсвечивает силуэты игроков через стены в демках и режиме наблюдателя."),
    ("incrementvar cl_hud_color 0 11 1", "Cycle the entire HUD color across all 12 available colors.", "Циклически меняет цвет всего интерфейса (HUD) среди 12 доступных цветов."),
    ("cl_draw_only_deathnotices 0", "Restore full HUD (0): shows radar, health, ammo, and standard UI elements again.", "Вернуть полный HUD (0): снова отображает радар, здоровье, патроны и обычный интерфейс."),
    ("cl_draw_only_deathnotices 1", "Killfeed-only mode (1): hides HUD and radar leaving only crosshair and deathnotices.", "Режим только killfeed (1): скрывает HUD и радар, оставляя только прицел и ленту убийств."),
    ("cl_drawhud 0", "Hide entire HUD (0, requires sv_cheats 1): removes absolutely all HUD elements from screen.", "Скрыть весь HUD (0, требуется sv_cheats 1): полностью убирает все элементы интерфейса с экрана."),
    ("cl_drawhud 1", "Show entire HUD (1): restores full display of all HUD elements.", "Показать весь HUD (1): восстанавливает полное отображение всех элементов интерфейса."),
    ("screenshot", "Take an immediate screenshot.", "Делает мгновенный снимок экрана (скриншот)."),
    ("jpeg", "Take an immediate screenshot in JPEG format.", "Делает мгновенный снимок экрана в формате JPEG."),
    ("toggleconsole", "Open or close the Developer Console (' or ~).", "Открывает или закрывает консоль разработчика (' или ~)."),
    ("exec autoexec", "Immediately load all settings saved in autoexec.cfg.", "Мгновенно загружает все настройки из файла autoexec.cfg."),
    ("callvote", "Open the vote menu (timeout, kick player, surrender).", "Открывает меню голосования (тайм-аут, кик игрока, сдача)."),
    ("disconnect", "Immediately disconnect from current server/match and return to main menu.", "Мгновенно отключается от текущего сервера/матча и возвращает в главное меню."),
    ("quit", "Immediately exit Counter-Strike 2 to the Desktop.", "Мгновенно закрывает Counter-Strike 2 на рабочий стол."),
    ("key_listboundkeys", "List all keys with active binds in the console.", "Выводит в консоль список всех клавиш с активными биндами."),
    ("key_findbinding +voicerecord", "Find which key is bound to a specific command (saved without inner quotes).", "Находит клавишу, назначенную на указанную команду (записывается без внутренних кавычек)."),
    ("find radar", "Search any CS2 command or variable containing the query term (saved without inner quotes).", "Ищет любую команду или переменную CS2 по заданному слову (записывается без внутренних кавычек)."),
    ("binddefaults", "Restore all keys to Valve's original factory defaults.", "Сбрасывает все клавиши к заводским настройкам Valve по умолчанию."),
];

pub fn localize_description(idx: usize, entry: &BindCommandEntry, lang: UiLanguage) -> &'static str {
    if lang == UiLanguage::PtBr {
        return entry.description;
    }
    if let Some(&(cfg, en_desc, ru_desc)) = CATALOG_TRANSLATIONS.get(idx) {
        if cfg == entry.cfg_command {
            return match lang {
                UiLanguage::EnUs => en_desc,
                UiLanguage::RuRu => ru_desc,
                UiLanguage::PtBr => entry.description,
            };
        }
    }
    entry.description
}

fn normalize_multilingual_text(input: &str) -> String {
    let base = normalize_search_text(input);
    if !base.contains('ё') {
        return base;
    }
    base.chars()
        .map(|c| if c == 'ё' { 'е' } else { c })
        .collect()
}

pub fn matches_localized_bind_search(
    idx: usize,
    entry: &BindCommandEntry,
    query: &str,
    _lang: UiLanguage,
) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }

    let normalized_query = normalize_multilingual_text(trimmed);
    let terms: Vec<&str> = normalized_query.split_whitespace().collect();
    if terms.is_empty() {
        return true;
    }

    let norm_display = normalize_multilingual_text(entry.display_command);
    let norm_cfg = normalize_multilingual_text(entry.cfg_command);
    let norm_pt = normalize_multilingual_text(entry.description);
    let (norm_en, norm_ru) = CATALOG_TRANSLATIONS
        .get(idx)
        .filter(|&&(cfg, _, _)| cfg == entry.cfg_command)
        .map(|&(_, en, ru)| (normalize_multilingual_text(en), normalize_multilingual_text(ru)))
        .unwrap_or_default();

    terms.iter().all(|term| {
        norm_display.contains(term)
            || norm_cfg.contains(term)
            || norm_pt.contains(term)
            || norm_en.contains(term)
            || norm_ru.contains(term)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use counterrush_core::{cs2_bind_catalog, cs2_bind_categories};
    use std::collections::BTreeSet;

    #[test]
    fn test_catalog_translations_align_1_to_1_with_core_catalog() {
        let catalog = cs2_bind_catalog();
        assert_eq!(CATALOG_TRANSLATIONS.len(), catalog.len());
        assert_eq!(catalog.len(), 237);

        for (idx, entry) in catalog.iter().enumerate() {
            let (cfg, en_desc, ru_desc) = CATALOG_TRANSLATIONS[idx];
            assert_eq!(cfg, entry.cfg_command, "Desalinhamento no índice {idx}");
            assert!(!en_desc.trim().is_empty(), "EN vazio no índice {idx}");
            assert!(!ru_desc.trim().is_empty(), "RU vazio no índice {idx}");
            assert_ne!(
                en_desc, entry.description,
                "EN igual ao PT no índice {idx} ({cfg})"
            );
            assert_ne!(
                ru_desc, entry.description,
                "RU igual ao PT no índice {idx} ({cfg})"
            );
        }
    }

    #[test]
    fn test_all_categories_and_subcategories_are_translated() {
        for cat in cs2_bind_categories() {
            let en = localize_category(&cat, UiLanguage::EnUs);
            let ru = localize_category(&cat, UiLanguage::RuRu);
            assert_ne!(en.tab_label, cat.tab_label);
            assert_ne!(en.title, cat.title);
            assert_ne!(en.subtitle, cat.subtitle);
            assert_ne!(ru.tab_label, cat.tab_label);
            assert_ne!(ru.title, cat.title);
            assert_ne!(ru.subtitle, cat.subtitle);
        }

        let subs: BTreeSet<&'static str> =
            cs2_bind_catalog().iter().map(|e| e.subcategory).collect();
        assert_eq!(subs.len(), 22);
        for sub in subs {
            let en = localize_subcategory(sub, UiLanguage::EnUs);
            let ru = localize_subcategory(sub, UiLanguage::RuRu);
            assert_ne!(en, sub, "Subcategoria sem tradução EN: {sub}");
            assert_ne!(ru, sub, "Subcategoria sem tradução RU: {sub}");
        }
    }

    #[test]
    fn test_multilingual_search_finds_commands_in_pt_en_and_ru() {
        let catalog = cs2_bind_catalog();
        let noclip_idx = catalog.iter().position(|e| e.cfg_command == "noclip").unwrap();
        let noclip = &catalog[noclip_idx];

        assert!(matches_localized_bind_search(
            noclip_idx,
            noclip,
            "walls",
            UiLanguage::EnUs
        ));
        assert!(matches_localized_bind_search(
            noclip_idx,
            noclip,
            "стены",
            UiLanguage::RuRu
        ));
        assert!(matches_localized_bind_search(
            noclip_idx,
            noclip,
            "paredes",
            UiLanguage::PtBr
        ));
        // Normalização de ё -> е
        assert!(matches_localized_bind_search(
            noclip_idx,
            noclip,
            "полет",
            UiLanguage::RuRu
        ));
    }
}
