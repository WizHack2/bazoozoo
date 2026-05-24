use matchbox_socket::WebRtcSocket;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub a_tire: bool,          // NOUVEAU : Vrai si le joueur a cliqué à cette frame
    pub souris_x: f32,         // Où il visait
    pub souris_y: f32,
}

pub struct NetworkManager {
    socket: WebRtcSocket,
}

pub enum GameMessage {
    ClientUpdate(PlayerState),
    HostSync(String),
}

impl NetworkManager {
    pub async fn new(room_url: &str) -> Self {
        let (socket, loop_fut) = WebRtcSocket::new_reliable(room_url);
        
        // La boucle réseau doit tourner en arrière-plan
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(loop_fut);
        
        #[cfg(not(target_arch = "wasm32"))]
        async_std::task::spawn(loop_fut); 

        Self { socket }
    }

    #[allow(dead_code)]
    pub fn update_and_receive(&mut self) -> Vec<PlayerState> {
        self.socket.update_peers();

        let mut states = Vec::new();
        for (_peer, packet) in self.socket.receive() {
            if let Ok(data) = bincode::deserialize::<PlayerState>(&packet) {
                states.push(data);
            }
        }
        states
    }

    pub fn send_state(&mut self, state: &PlayerState) {
        let bytes = bincode::serialize(state).unwrap().into_boxed_slice();
        let peers: Vec<_> = self.socket.connected_peers().collect();
        
        for peer in peers {
            self.socket.send(bytes.clone(), peer);
        }
    }

    pub fn receive_messages(&mut self) -> Vec<GameMessage> {
        self.socket.update_peers();
        let mut messages = Vec::new();
        for (_peer, packet) in self.socket.receive() {
            let mut is_json = false;
            // Un message JSON commence toujours par le caractère '{' (ASCII 123)
            if packet.first() == Some(&b'{') {
                if let Ok(json_str) = String::from_utf8(packet.to_vec()) {
                    // Pour éviter les faux positifs (comme un ID bincode commençant par 123),
                    // on vérifie si la chaîne est du JSON valide.
                    if serde_json::from_str::<serde_json::Value>(&json_str).is_ok() {
                        messages.push(GameMessage::HostSync(json_str));
                        is_json = true;
                    }
                }
            }
            if !is_json {
                // Sinon, c'est le PlayerState bincode d'un client !
                if let Ok(state) = bincode::deserialize::<PlayerState>(&packet) {
                    messages.push(GameMessage::ClientUpdate(state));
                }
            }
        }
        messages
    }

    pub fn send_json(&mut self, json_str: &str) {
        let bytes = json_str.as_bytes().to_vec().into_boxed_slice();
        let peers: Vec<_> = self.socket.connected_peers().collect();
        for peer in peers {
            self.socket.send(bytes.clone(), peer);
        }
    }
    
}
