use macroquad::prelude::*;

pub struct Animation {
    spritesheet: Option<Texture2D>,
    color: Color,
    cols: usize,
    rows: usize,
    frame_counts: Vec<usize>,
    pub current_row: usize,
    pub current_frame: usize,
    timer: f32,
    looping: bool,
    durations_per_frame: Vec<f32>,
}

impl Animation {
    pub fn new(
        spritesheet: Option<Texture2D>, rows: usize, cols: usize, frame_counts: Vec<usize>,
    ) -> Self {
        Self {
            spritesheet, color: WHITE, rows, cols, frame_counts,
            current_row: 0, current_frame: 0, timer: 0.0, looping: true, durations_per_frame: Vec::new(),
        }
    }

    pub fn play_animation(&mut self, row: usize, looping: bool, durations: &[f32]) -> bool {

        // Intitialisation si on appelle pour jouer une nouvelle animation
        if self.current_row != row || self.durations_per_frame.is_empty() || self.durations_per_frame != durations.to_vec() {
            self.looping = looping;
            self.current_row = row;
            self.current_frame = 0;
            self.durations_per_frame = durations.to_vec();
            self.timer = self.durations_per_frame[self.current_frame];
        }

        // Vérification que duration est complet.
        if durations.len() != self.frame_counts[self.current_row] {
            panic!("Error: Duration table should be of same lenght than the frame_count of the current_row:\ndurations table lenght: {}, frame_count of current row: {}", durations.len(), self.frame_counts[row]);
        }

        // Vérfication de fin d'animation qui ne boucle pas.
        if !self.looping && self.current_frame == self.frame_counts[self.current_row] - 1 && self.timer <= 0.0 {
            return true;
        }

        // Gestion du timer
        let dt = get_frame_time().clamp(0.001, 0.05);
        self.timer -= dt;

        // Changement de frame
        let mut finished = false;
        if self.timer <= 0.0 {
            finished = self.skip_current_frame();
        }

        finished
    }

    pub fn skip_current_frame(&mut self) -> bool {
        self.current_frame +=1;

        if self.current_frame >= self.frame_counts[self.current_row] {
            if !self.looping {
                self.current_frame = self.frame_counts[self.current_row] - 1;
                return true;
            } 
            self.current_frame = 0;
        }

        self.timer = self.durations_per_frame[self.current_frame];

        return false;
    }

    pub fn change_color(&mut self, color: Color) {
        self.color =  color;
    }

    pub fn draw_current_frame(&self, x: f32, y: f32, w: f32, h: f32, look_right: bool) {
        if let Some(tex) = &self.spritesheet {
            let fw = tex.width() / self.cols as f32;
            let fh = tex.height() / self.rows as f32;
            let source_rect = Rect::new(self.current_frame as f32 * fw, self.current_row as f32 * fh, fw, fh);

            draw_texture_ex(
                tex, x, y, self.color,
                DrawTextureParams { 
                    source: Some(source_rect),
                    dest_size: Some(vec2(w, h)),
                    flip_x: !look_right,
                    ..Default::default() 
                },
            );
        } else {
            draw_rectangle(x, y, w, h, GREEN);
        }
    }
}
