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
            // Est-ce que c'est le JSON de l'hôte ?
            if let Ok(json_str) = String::from_utf8(packet.to_vec()) {
                if json_str.starts_with('{') {
                    messages.push(GameMessage::HostSync(json_str));
                    continue;
                }
            }
            // Sinon, c'est le PlayerState d'un client !
            if let Ok(state) = bincode::deserialize::<PlayerState>(&packet) {
                messages.push(GameMessage::ClientUpdate(state));
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
