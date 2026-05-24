use macroquad::prelude::*;
use std::net::{TcpStream, SocketAddr, UdpSocket};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::Assets;
use crate::keybindings::Layout;

// --- DÉTECTION IP LOCALE ROBUSTE (FONCTIONNE HORSLIGNE SOUS LINUX) ---
pub fn get_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    
    // Sur Linux, hostname -I renvoie toutes les IP locales attribuées et fonctionne hors-ligne
    if let Ok(output) = std::process::Command::new("hostname").arg("-I").output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            for ip in s.split_whitespace() {
                if !ip.starts_with("127.") && !ip.contains(':') {
                    ips.push(ip.to_string());
                }
            }
        }
    }
    
    // Méthode de fallback (UDP vers DNS Google) si hostname échoue ou n'est pas dispo
    if ips.is_empty() {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    ips.push(local_addr.ip().to_string());
                }
            }
        }
    }
    
    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }
    
    ips
}

#[allow(dead_code)]
pub fn get_local_ip() -> Option<String> {
    get_local_ips().first().cloned()
}

// --- SCANNER RÉSEAU RAPIDE CONCURRENT ---
pub struct NetworkScanner {
    found_ips: Arc<Mutex<Vec<String>>>,
    is_scanning: Arc<Mutex<bool>>,
    scan_progress: Arc<Mutex<usize>>,
}

impl NetworkScanner {
    pub fn new() -> Self {
        Self {
            found_ips: Arc::new(Mutex::new(Vec::new())),
            is_scanning: Arc::new(Mutex::new(false)),
            scan_progress: Arc::new(Mutex::new(0)),
        }
    }

    pub fn start_scan(&self) {
        let found_ips = self.found_ips.clone();
        let is_scanning = self.is_scanning.clone();
        let scan_progress = self.scan_progress.clone();

        {
            let mut scanning = is_scanning.lock().expect("Mutex poisoned in menu");
            if *scanning {
                return; // Déjà en cours
            }
            *scanning = true;
            let mut ips = found_ips.lock().expect("Mutex poisoned in menu");
            ips.clear();
            let mut progress = scan_progress.lock().expect("Mutex poisoned in menu");
            *progress = 0;
        }

        thread::spawn(move || {
            let local_ips = get_local_ips();
            let mut subnets = Vec::new();
            for ip in local_ips {
                let parts: Vec<&str> = ip.split('.').collect();
                if parts.len() == 4 {
                    subnets.push(format!("{}.{}.{}.", parts[0], parts[1], parts[2]));
                }
            }
            if subnets.is_empty() {
                subnets.push("192.168.1.".to_string());
            }

            let mut threads = Vec::new();
            // Pour chaque sous-réseau détecté
            for base_ip in subnets {
                for last_part in 1..=254 {
                    let ip = format!("{}{}", base_ip, last_part);
                    let found_ips = found_ips.clone();
                    let scan_progress = scan_progress.clone();

                    let handle = thread::spawn(move || {
                        let addr_str = format!("{}:3536", ip);
                        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                            // Timeout très court pour un réseau local
                            if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok() {
                                let mut list = found_ips.lock().expect("Mutex poisoned in menu");
                                if !list.contains(&ip) {
                                    list.push(ip);
                                }
                            }
                        }
                        let mut progress = scan_progress.lock().expect("Mutex poisoned in menu");
                        *progress += 1;
                    });
                    threads.push(handle);

                    // Traiter par vagues pour éviter la saturation de sockets
                    if threads.len() >= 48 {
                        for t in threads {
                            let _ = t.join();
                        }
                        threads = Vec::new();
                    }
                }
            }

            for t in threads {
                let _ = t.join();
            }

            // Toujours scanner l'adresse locale (127.0.0.1)
            let local_addr_str = "127.0.0.1:3536";
            if let Ok(addr) = local_addr_str.parse::<SocketAddr>() {
                if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok() {
                    let mut list = found_ips.lock().expect("Mutex poisoned in menu");
                    if !list.contains(&"127.0.0.1".to_string()) {
                        list.push("127.0.0.1".to_string());
                    }
                }
            }

            let mut scanning = is_scanning.lock().expect("Mutex poisoned in menu");
            *scanning = false;
        });
    }

    pub fn get_found_ips(&self) -> Vec<String> {
        let list = self.found_ips.lock().expect("Mutex poisoned in menu");
        list.clone()
    }

    pub fn is_scanning(&self) -> bool {
        let scanning = self.is_scanning.lock().expect("Mutex poisoned in menu");
        *scanning
    }

    pub fn get_progress(&self) -> f32 {
        let progress = self.scan_progress.lock().expect("Mutex poisoned in menu");
        (*progress as f32 / 254.0).min(1.0)
    }
}

// --- CONFIGURATION DU MENU ---
#[derive(PartialEq, Clone, Copy)]
pub enum MenuRole {
    Host,
    Client,
}

pub struct MenuState {
    pub pseudo: String,
    pub character_id: u8,
    pub role: MenuRole,
    pub room_name: String,
    pub server_ip: String,
    pub finished: bool,
    pub scanner: NetworkScanner,
    pub active_input: u8, // 0: Pseudo, 1: Room, 2: Server IP
    cursor_timer: f32,
    pub layout: Layout,
}

impl MenuState {
    pub fn new() -> Self {
        let local_ips = get_local_ips();
        let primary_ip = local_ips.first().cloned().unwrap_or_else(|| "127.0.0.1".to_string());
        
        let scanner = NetworkScanner::new();
        // Lancer un premier scan réseau en arrière-plan d'emblée
        scanner.start_scan();

        let clean_room = primary_ip.replace('.', "_");

        Self {
            pseudo: format!("Hero_{}", macroquad::rand::gen_range(100, 999)),
            character_id: 0,
            role: MenuRole::Host,
            room_name: format!("room_{}", clean_room),
            server_ip: "127.0.0.1".to_string(), // Par défaut localhost pour le jeu local immédiat
            finished: false,
            scanner,
            active_input: 0,
            cursor_timer: 0.0,
            layout: Layout::Azerty,
        }
    }

    pub fn update(&mut self) {
        self.cursor_timer += get_frame_time();
        
        // --- CHANGER DE RÔLE ---
        if is_key_pressed(KeyCode::Tab) {
            self.active_input = (self.active_input + 1) % if self.role == MenuRole::Host { 2 } else { 3 };
        }

        // --- ENTRÉE DU TEXTE ---
        // Vérifier si le nom de la salle est le nom par défaut de l'IP du serveur OU de l'IP locale
        let clean_local_ip_room = format!("room_{}", get_local_ips().first().cloned().unwrap_or_else(|| "127.0.0.1".to_string()).replace('.', "_"));
        let clean_current_server_ip_room = format!("room_{}", self.server_ip.replace('.', "_"));
        let sync_room = self.role == MenuRole::Client && 
            (self.room_name == clean_current_server_ip_room || self.room_name == clean_local_ip_room);

        while let Some(c) = get_char_pressed() {
            if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == ' ' {
                let limit = match self.active_input {
                    0 => 12,
                    1 => 15,
                    2 => 15,
                    _ => 15,
                };
                let active_str = match self.active_input {
                    0 => &mut self.pseudo,
                    1 => &mut self.room_name,
                    2 => &mut self.server_ip,
                    _ => continue,
                };
                if active_str.len() < limit {
                    active_str.push(c);
                }
            }
        }

        // Effacer
        if is_key_pressed(KeyCode::Backspace) {
            let active_str = match self.active_input {
                0 => &mut self.pseudo,
                1 => &mut self.room_name,
                2 => &mut self.server_ip,
                _ => return,
            };
            active_str.pop();
        }

        // Si le nom de la room était synchronisé, on propage la modification de l'IP du serveur
        if sync_room {
            self.room_name = format!("room_{}", self.server_ip.replace('.', "_"));
        }

        // Entrée lance l'arène
        if is_key_pressed(KeyCode::Enter) {
            self.finished = true;
        }
    }

    pub fn draw(&mut self, assets: &Assets) {
        clear_background(Color::new(0.02, 0.02, 0.04, 1.0));

        let screen_w = screen_width();
        let screen_h = screen_height();

        // 1. DESSINER LE FOND RETRO-FUTURISTE (Grille subtile pulsante)
        let time = macroquad::time::get_time();
        let grid_size = 40.0;
        let offset_y = (time * 15.0) % grid_size as f64;
        for y in (0..(screen_h as i32)).step_by(grid_size as usize) {
            let line_y = y as f32 + offset_y as f32;
            draw_line(0.0, line_y, screen_w, line_y, 1.0, Color::new(0.08, 0.08, 0.15, 0.3));
        }
        for x in (0..(screen_w as i32)).step_by(grid_size as usize) {
            draw_line(x as f32, 0.0, x as f32, screen_h, 1.0, Color::new(0.08, 0.08, 0.15, 0.3));
        }

        // 2. DESSIN DU TITRE AVEC EFFET NEON CYAN
        let title = "B A Z O O Z O O";
        let title_font_size = 64.0;
        let title_w = measure_text(title, None, title_font_size as u16, 1.0).width;
        let title_x = (screen_w - title_w) / 2.0;
        let title_y = 80.0;
        
        let glow_offset = (time * 3.0).sin() as f32 * 1.5;
        // Halo de lueur cyan
        draw_text(title, title_x - 2.0 + glow_offset, title_y + 2.0, title_font_size, Color::new(0.0, 0.9, 1.0, 0.3));
        draw_text(title, title_x + 2.0 - glow_offset, title_y - 2.0, title_font_size, Color::new(1.0, 0.0, 0.9, 0.3));
        draw_text(title, title_x, title_y, title_font_size, Color::new(0.95, 0.95, 1.0, 1.0));

        let sub_title = "ARENA MULTIPLAYER ROCKET JUMPING";
        let sub_font_size = 18.0;
        let sub_w = measure_text(sub_title, None, sub_font_size as u16, 1.0).width;
        draw_text(sub_title, (screen_w - sub_w) / 2.0, title_y + 28.0, sub_font_size, Color::new(0.5, 0.5, 0.6, 1.0));

        // 3. FOND ET LAYOUT DU PANNEAU (Glassmorphism haut de gamme)
        let p_w = 900.0;
        let p_h = 480.0;
        let p_x = (screen_w - p_w) / 2.0;
        let p_y = 150.0;

        // Panneau de fond translucide sombre avec bordures cyan/magenta
        draw_rectangle(p_x, p_y, p_w, p_h, Color::new(0.03, 0.03, 0.06, 0.92));
        draw_rectangle_lines(p_x, p_y, p_w, p_h, 3.0, Color::new(0.2, 0.25, 0.35, 1.0));
        draw_rectangle_lines(p_x - 3.0, p_y - 3.0, p_w + 6.0, p_h + 6.0, 1.0, Color::new(0.0, 0.8, 1.0, 0.25));

        // --- COLONNE GAUCHE : PARAMÈTRES (Largeur 400.0) ---
        let col1_x = p_x + 30.0;
        let start_inputs_y = p_y + 40.0;

        // A. CHAMP PSEUDO
        draw_text("PSEUDO DU COMBATTANT :", col1_x, start_inputs_y, 18.0, Color::new(0.8, 0.8, 0.9, 1.0));
        let pseudo_box_y = start_inputs_y + 8.0;
        let box_w = 340.0;
        let box_h = 36.0;
        let pseudo_active = self.active_input == 0;
        
        draw_rectangle(col1_x, pseudo_box_y, box_w, box_h, Color::new(0.01, 0.01, 0.02, 1.0));
        draw_rectangle_lines(col1_x, pseudo_box_y, box_w, box_h, 2.0, if pseudo_active { SKYBLUE } else { Color::new(0.15, 0.15, 0.2, 1.0) });
        
        draw_text(&self.pseudo, col1_x + 10.0, pseudo_box_y + 24.0, 20.0, WHITE);
        if pseudo_active && (self.cursor_timer % 0.8 < 0.4) {
            let caret_x = col1_x + 12.0 + measure_text(&self.pseudo, None, 20, 1.0).width;
            draw_rectangle(caret_x, pseudo_box_y + 8.0, 2.0, 20.0, SKYBLUE);
        }

        // B. CHOIX RÔLE : HÔTE OU CLIENT
        let role_y = pseudo_box_y + 65.0;
        draw_text("RÔLE RÉSEAU :", col1_x, role_y, 18.0, Color::new(0.8, 0.8, 0.9, 1.0));
        let btn_w = 160.0;
        let btn_h = 34.0;

        // Bouton HÔTE
        let host_btn_x = col1_x;
        let host_btn_y = role_y + 8.0;
        let host_hover = mouse_position().0 >= host_btn_x && mouse_position().0 <= host_btn_x + btn_w && mouse_position().1 >= host_btn_y && mouse_position().1 <= host_btn_y + btn_h;
        if host_hover && is_mouse_button_pressed(MouseButton::Left) {
            self.role = MenuRole::Host;
            self.active_input = 1; // Focus sur le nom de la room
        }
        draw_rectangle(host_btn_x, host_btn_y, btn_w, btn_h, if self.role == MenuRole::Host { Color::new(0.2, 0.5, 0.2, 0.7) } else { Color::new(0.05, 0.05, 0.08, 1.0) });
        draw_rectangle_lines(host_btn_x, host_btn_y, btn_w, btn_h, 2.0, if self.role == MenuRole::Host { GREEN } else { Color::new(0.15, 0.15, 0.2, 1.0) });
        draw_text("CRÉER SALLE (HOST)", host_btn_x + 10.0, host_btn_y + 22.0, 15.0, if self.role == MenuRole::Host { WHITE } else { GRAY });

        // Bouton CLIENT
        let client_btn_x = col1_x + 180.0;
        let client_btn_y = role_y + 8.0;
        let client_hover = mouse_position().0 >= client_btn_x && mouse_position().0 <= client_btn_x + btn_w && mouse_position().1 >= client_btn_y && mouse_position().1 <= client_btn_y + btn_h;
        if (client_hover || (mouse_position().0 >= client_btn_x && mouse_position().0 <= client_btn_x + btn_w && mouse_position().1 >= client_btn_y && mouse_position().1 <= client_btn_y + btn_h)) && is_mouse_button_pressed(MouseButton::Left) {
            self.role = MenuRole::Client;
            self.active_input = 2; // Focus sur l'IP
            self.scanner.start_scan();
        }
        draw_rectangle(client_btn_x, client_btn_y, btn_w, btn_h, if self.role == MenuRole::Client { Color::new(0.2, 0.4, 0.6, 0.7) } else { Color::new(0.05, 0.05, 0.08, 1.0) });
        draw_rectangle_lines(client_btn_x, client_btn_y, btn_w, btn_h, 2.0, if self.role == MenuRole::Client { SKYBLUE } else { Color::new(0.15, 0.15, 0.2, 1.0) });
        draw_text("REJOINDRE (CLIENT)", client_btn_x + 10.0, client_btn_y + 22.0, 15.0, if self.role == MenuRole::Client { WHITE } else { GRAY });

        // C. LAYOUT CLAVIER (AZERTY / QWERTY)
        let clavier_y = host_btn_y + 44.0;
        draw_text("DISPOSITION CLAVIER :", col1_x, clavier_y, 16.0, Color::new(0.8, 0.8, 0.9, 1.0));
        let layout_btn_y = clavier_y + 8.0;
        let layout_btn_w = 90.0;
        let layout_btn_h = 26.0;

        let azerty_x = col1_x;
        let azerty_hover = mouse_position().0 >= azerty_x && mouse_position().0 <= azerty_x + layout_btn_w && mouse_position().1 >= layout_btn_y && mouse_position().1 <= layout_btn_y + layout_btn_h;
        if azerty_hover && is_mouse_button_pressed(MouseButton::Left) {
            self.layout = Layout::Azerty;
        }
        draw_rectangle(azerty_x, layout_btn_y, layout_btn_w, layout_btn_h, if self.layout == Layout::Azerty { Color::new(0.2, 0.5, 0.2, 0.7) } else { Color::new(0.05, 0.05, 0.08, 1.0) });
        draw_rectangle_lines(azerty_x, layout_btn_y, layout_btn_w, layout_btn_h, 2.0, if self.layout == Layout::Azerty { GREEN } else { Color::new(0.15, 0.15, 0.2, 1.0) });
        draw_text("AZERTY", azerty_x + 16.0, layout_btn_y + 17.0, 14.0, if self.layout == Layout::Azerty { WHITE } else { GRAY });

        let qwerty_x = col1_x + 100.0;
        let qwerty_hover = mouse_position().0 >= qwerty_x && mouse_position().0 <= qwerty_x + layout_btn_w && mouse_position().1 >= layout_btn_y && mouse_position().1 <= layout_btn_y + layout_btn_h;
        if qwerty_hover && is_mouse_button_pressed(MouseButton::Left) {
            self.layout = Layout::Qwerty;
        }
        draw_rectangle(qwerty_x, layout_btn_y, layout_btn_w, layout_btn_h, if self.layout == Layout::Qwerty { Color::new(0.2, 0.4, 0.6, 0.7) } else { Color::new(0.05, 0.05, 0.08, 1.0) });
        draw_rectangle_lines(qwerty_x, layout_btn_y, layout_btn_w, layout_btn_h, 2.0, if self.layout == Layout::Qwerty { SKYBLUE } else { Color::new(0.15, 0.15, 0.2, 1.0) });
        draw_text("QWERTY", qwerty_x + 12.0, layout_btn_y + 17.0, 14.0, if self.layout == Layout::Qwerty { WHITE } else { GRAY });

        // D. ZONE DYNAMIQUE RÉSEAU (HÔTE vs CLIENT)
        let net_y = layout_btn_y + 50.0;
        if self.role == MenuRole::Host {
            draw_text("NOM DE LA ROOM :", col1_x, net_y, 18.0, Color::new(0.8, 0.8, 0.9, 1.0));
            let room_box_y = net_y + 8.0;
            let room_active = self.active_input == 1;
            draw_rectangle(col1_x, room_box_y, box_w, box_h, Color::new(0.01, 0.01, 0.02, 1.0));
            draw_rectangle_lines(col1_x, room_box_y, box_w, box_h, 2.0, if room_active { GREEN } else { Color::new(0.15, 0.15, 0.2, 1.0) });
            draw_text(&self.room_name, col1_x + 10.0, room_box_y + 24.0, 20.0, WHITE);
            
            if room_active && (self.cursor_timer % 0.8 < 0.4) {
                let caret_x = col1_x + 12.0 + measure_text(&self.room_name, None, 20, 1.0).width;
                draw_rectangle(caret_x, room_box_y + 8.0, 2.0, 20.0, GREEN);
            }
            
            draw_text("La room sera hébergée sur votre serveur local.", col1_x, room_box_y + 55.0, 14.0, ORANGE);
            draw_text("Partagez votre IP ou le nom de la room avec vos amis.", col1_x, room_box_y + 73.0, 13.0, GRAY);
        } else {
            draw_text("IP DU SERVEUR / MATCHMAKER :", col1_x, net_y, 18.0, Color::new(0.8, 0.8, 0.9, 1.0));
            let ip_box_y = net_y + 8.0;
            let ip_active = self.active_input == 2;
            draw_rectangle(col1_x, ip_box_y, box_w, box_h, Color::new(0.01, 0.01, 0.02, 1.0));
            draw_rectangle_lines(col1_x, ip_box_y, box_w, box_h, 2.0, if ip_active { SKYBLUE } else { Color::new(0.15, 0.15, 0.2, 1.0) });
            draw_text(&self.server_ip, col1_x + 10.0, ip_box_y + 24.0, 20.0, WHITE);
            
            if ip_active && (self.cursor_timer % 0.8 < 0.4) {
                let caret_x = col1_x + 12.0 + measure_text(&self.server_ip, None, 20, 1.0).width;
                draw_rectangle(caret_x, ip_box_y + 8.0, 2.0, 20.0, SKYBLUE);
            }

            // --- SECTION SCAN RÉSEAU ---
            let scan_y = ip_box_y + 50.0;
            if self.scanner.is_scanning() {
                draw_rectangle(col1_x, scan_y, box_w, 20.0, Color::new(0.1, 0.1, 0.15, 1.0));
                draw_rectangle(col1_x, scan_y, box_w * self.scanner.get_progress(), 20.0, Color::new(0.0, 0.6, 0.9, 0.7));
                draw_text("SCAN RÉSEAU LOCAL EN COURS...", col1_x + 10.0, scan_y + 15.0, 13.0, WHITE);
            } else {
                let found = self.scanner.get_found_ips();
                if found.is_empty() {
                    draw_text("Aucune salle détectée sur le réseau local.", col1_x, scan_y + 12.0, 14.0, RED);
                    
                    let rscan_x = col1_x;
                    let rscan_y = scan_y + 22.0;
                    let rscan_w = 180.0;
                    let rscan_h = 24.0;
                    let rscan_hover = mouse_position().0 >= rscan_x && mouse_position().0 <= rscan_x + rscan_w && mouse_position().1 >= rscan_y && mouse_position().1 <= rscan_y + rscan_h;
                    if rscan_hover && is_mouse_button_pressed(MouseButton::Left) {
                        self.scanner.start_scan();
                    }
                    draw_rectangle(rscan_x, rscan_y, rscan_w, rscan_h, if rscan_hover { Color::new(0.15, 0.15, 0.25, 1.0) } else { Color::new(0.08, 0.08, 0.12, 1.0) });
                    draw_rectangle_lines(rscan_x, rscan_y, rscan_w, rscan_h, 1.0, Color::new(0.3, 0.3, 0.4, 1.0));
                    draw_text("LANCER LE SCANNER LAN", rscan_x + 10.0, rscan_y + 16.0, 12.0, SKYBLUE);
                } else {
                    draw_text("Salles détectées sur le réseau (Port 3536) :", col1_x, scan_y + 12.0, 14.0, Color::new(0.4, 0.9, 0.4, 1.0));
                    
                    // On liste les 3 premières IPs trouvées
                    for (i, found_ip) in found.iter().take(3).enumerate() {
                        let item_y = scan_y + 32.0 + (i as f32 * 26.0);
                        let item_hover = mouse_position().0 >= col1_x && mouse_position().0 <= col1_x + box_w && mouse_position().1 >= item_y - 18.0 && mouse_position().1 <= item_y + 8.0;
                        
                        if item_hover && is_mouse_button_pressed(MouseButton::Left) {
                            self.server_ip = found_ip.clone();
                            self.room_name = format!("room_{}", found_ip.replace('.', "_"));
                        }

                        draw_rectangle(col1_x, item_y - 18.0, box_w, 24.0, if item_hover { Color::new(0.0, 0.2, 0.4, 0.4) } else { Color::new(0.02, 0.02, 0.04, 0.6) });
                        draw_rectangle_lines(col1_x, item_y - 18.0, box_w, 24.0, 1.0, if item_hover { SKYBLUE } else { Color::new(0.1, 0.15, 0.2, 0.5) });
                        
                        draw_text(&format!("IP HÔTE : {}  (Cliquez pour choisir)", found_ip), col1_x + 10.0, item_y - 1.0, 13.0, WHITE);
                    }
                }
            }
        }

        // --- COLONNE DROITE : SELECTION DES PERSONNAGES (Largeur 400.0) ---
        let col2_x = p_x + 440.0;
        
        draw_text("CHOIX DE VOTRE PERSONNAGE :", col2_x, start_inputs_y, 18.0, Color::new(0.8, 0.8, 0.9, 1.0));
        
        let card_w = 130.0;
        let card_h = 175.0;
        let cards_y = start_inputs_y + 15.0;
        
        let chars_info = [
            ("ASTERION", assets.player.clone(), "Guerrier", "ÉQUILIBRÉ", "Speed: 40.0\nAmmo: 3 (Rockets)\nDefault balanced mode."),
            ("FOX", assets.fox.clone(), "Agile", "TRÈS RAPIDE", "Speed: 48.0\nAmmo: 2 (Rockets)\nAgile and swift ranger."),
            ("SHADOW", assets.shadow.clone(), "Lourd", "ROBUSTE", "Speed: 34.0\nAmmo: 4 (Rockets)\nHeavy, slow & resilient."),
        ];

        for i in 0..3 {
            let card_x = col2_x + (i as f32 * (card_w + 14.0));
            let is_selected = self.character_id == i as u8;
            
            let card_hover = mouse_position().0 >= card_x && mouse_position().0 <= card_x + card_w && mouse_position().1 >= cards_y && mouse_position().1 <= cards_y + card_h;
            if card_hover && is_mouse_button_pressed(MouseButton::Left) {
                self.character_id = i as u8;
            }

            // Dessiner la boîte de carte
            let border_c = if is_selected {
                Color::new(0.9, 0.1, 0.8, 1.0) // Magenta néon pour le sélectionné
            } else if card_hover {
                SKYBLUE
            } else {
                Color::new(0.15, 0.18, 0.25, 1.0)
            };

            draw_rectangle(card_x, cards_y, card_w, card_h, if is_selected { Color::new(0.1, 0.02, 0.1, 0.8) } else { Color::new(0.01, 0.01, 0.02, 0.85) });
            draw_rectangle_lines(card_x, cards_y, card_w, card_h, if is_selected { 3.0 } else { 1.5 }, border_c);

            // Titre de la carte
            draw_text(chars_info[i].0, card_x + 12.0, cards_y + 24.0, 16.0, border_c);
            draw_text(chars_info[i].2, card_x + 12.0, cards_y + 38.0, 12.0, GRAY);
            
            // Texture du personnage (Preview agrandie rétro)
            let tex = &chars_info[i].1;
            // On extrait la première frame de la spritesheet
            let frame_h = tex.height() / 2.0;
            draw_texture_ex(
                tex,
                card_x + (card_w - 48.0) / 2.0,
                cards_y + 45.0,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, 0.0, tex.width(), frame_h)),
                    dest_size: Some(vec2(48.0, 48.0)),
                    ..Default::default()
                }
            );

            // Trait distinctif
            draw_text(chars_info[i].3, card_x + 12.0, cards_y + 115.0, 12.0, if i == 1 { Color::new(0.9, 0.8, 0.2, 1.0) } else if i == 2 { PURPLE } else { SKYBLUE });
            
            // Stats rapides
            let stats_lines = chars_info[i].4.split('\n');
            for (line_idx, stat_line) in stats_lines.enumerate() {
                draw_text(stat_line, card_x + 12.0, cards_y + 133.0 + (line_idx as f32 * 12.0), 10.0, LIGHTGRAY);
            }
        }

        // --- DESCRIPTION COMPLÈTE DU PERSONNAGE SÉLECTIONNÉ ---
        let desc_y = cards_y + card_h + 20.0;
        let selected_info = &chars_info[self.character_id as usize];
        
        draw_rectangle(col2_x, desc_y, 418.0, 68.0, Color::new(0.01, 0.01, 0.02, 1.0));
        draw_rectangle_lines(col2_x, desc_y, 418.0, 68.0, 1.0, Color::new(0.12, 0.15, 0.25, 0.5));
        
        let desc_title = format!("{} (Classe : {})", selected_info.0, selected_info.2);
        draw_text(&desc_title, col2_x + 12.0, desc_y + 20.0, 16.0, SKYBLUE);
        
        let full_desc = match self.character_id {
            0 => "Équipement militaire standard. Stat de vitesse et de propulsion fiables.",
            1 => "Châssis ultra léger. Se déplace extrêmement vite mais n'embarque que 2 rockets maximum.",
            2 => "Armure lourde expérimentale. Réduit la vitesse globale, mais permet d'enchaîner 4 rockets !",
            _ => "",
        };
        draw_text(full_desc, col2_x + 12.0, desc_y + 40.0, 12.0, GRAY);
        draw_text("Le recul des roquettes permet de vous propulser en l'air !", col2_x + 12.0, desc_y + 55.0, 12.0, ORANGE);

        // 4. BOUTON MAGIQUE DE LANCEMENT "ENTRER DANS L'ARÈNE"
        let arena_w = 400.0;
        let arena_h = 50.0;
        let arena_x = (screen_w - arena_w) / 2.0;
        let arena_y = p_y + p_h - 65.0;

        let arena_hover = mouse_position().0 >= arena_x && mouse_position().0 <= arena_x + arena_w && mouse_position().1 >= arena_y && mouse_position().1 <= arena_y + arena_h;
        if arena_hover && is_mouse_button_pressed(MouseButton::Left) {
            self.finished = true;
        }

        // Effet de pulsation lumineuse sur le bouton d'arène
        let pulse = 0.8 + (time * 6.0).sin() as f32 * 0.15;
        let start_c = if arena_hover { Color::new(0.8 * pulse, 0.0, 0.7 * pulse, 0.9) } else { Color::new(0.1, 0.4 * pulse, 0.6 * pulse, 0.8) };
        let outline_c = if arena_hover { Color::new(1.0, 0.0, 0.8, 1.0) } else { SKYBLUE };

        draw_rectangle(arena_x, arena_y, arena_w, arena_h, start_c);
        draw_rectangle_lines(arena_x, arena_y, arena_w, arena_h, 3.0, outline_c);
        
        let start_text = "ENTRER DANS L'ARENE (ENTRÉE)";
        let start_font_size = 22.0;
        let text_w = measure_text(start_text, None, start_font_size as u16, 1.0).width;
        draw_text(start_text, arena_x + (arena_w - text_w) / 2.0, arena_y + 32.0, start_font_size, WHITE);
    }
}
