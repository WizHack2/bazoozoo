use matchbox_socket::WebRtcSocket;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: i32,
    pub x: f32,
    pub y: f32,
}

pub struct NetworkManager {
    socket: WebRtcSocket,
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
}
