use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound_from_bytes};

pub struct Assets {
    pub background: Texture2D,
    pub hollow_background: Texture2D,
    pub platform_tile: Texture2D,
    pub player: Texture2D,
    pub fox: Texture2D,
    pub shadow: Texture2D,
    pub sound_shoot: Sound,
    pub sound_explosion: Sound,
    pub sound_jump: Sound,
    pub sound_land: Sound,
    pub sound_reload: Sound,
}

impl Assets {
    pub async fn load() -> Self {
        let background = load_texture("assets/background.png").await
            .expect("Failed to load assets/background.png");
        background.set_filter(FilterMode::Nearest);

        let hollow_background = load_texture("assets/hollow_map.png").await
            .expect("Failed to load assets/hollow_map.png");
        hollow_background.set_filter(FilterMode::Nearest);

        let platform_tile = load_texture("assets/platform_tile.png").await
            .expect("Failed to load assets/platform_tile.png");
        platform_tile.set_filter(FilterMode::Nearest);

        let player = load_texture("assets/Asterion.png").await
            .expect("Failed to load assets/Asterion.png");
        player.set_filter(FilterMode::Nearest);

        let fox = load_texture("assets/fox.png").await
            .expect("Failed to load assets/fox.png");
        fox.set_filter(FilterMode::Nearest);

        let player_img = load_image("assets/Asterion.png").await
            .expect("Failed to load assets/Asterion.png");
        let shadow = create_shadow_texture(&player_img);

        let sound_shoot = load_sound_from_bytes(&make_wav_shoot()).await
            .expect("Failed to generate shoot sound");
        let sound_explosion = load_sound_from_bytes(&make_wav_explosion()).await
            .expect("Failed to generate explosion sound");
        let sound_jump = load_sound_from_bytes(&make_wav_jump()).await
            .expect("Failed to generate jump sound");
        let sound_land = load_sound_from_bytes(&make_wav_land()).await
            .expect("Failed to generate land sound");
        let sound_reload = load_sound_from_bytes(&make_wav_reload()).await
            .expect("Failed to generate reload sound");

        Self {
            background,
            hollow_background,
            platform_tile,
            player,
            fox,
            shadow,
            sound_shoot,
            sound_explosion,
            sound_jump,
            sound_land,
            sound_reload,
        }
    }
}

fn make_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let bytes_per_sample = 2;
    let num_channels: u16 = 1;
    let data_size = samples.len() as u32 * bytes_per_sample;
    let riff_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    wav.extend(b"RIFF");
    wav.extend(&riff_size.to_le_bytes());
    wav.extend(b"WAVE");

    // fmt chunk
    wav.extend(b"fmt ");
    wav.extend(&16u32.to_le_bytes());
    wav.extend(&1u16.to_le_bytes()); // PCM
    wav.extend(&num_channels.to_le_bytes());
    wav.extend(&sample_rate.to_le_bytes());
    wav.extend(&(sample_rate * bytes_per_sample as u32 * num_channels as u32).to_le_bytes());
    wav.extend(&(bytes_per_sample as u16 * num_channels).to_le_bytes());
    wav.extend(&(bytes_per_sample as u16 * 8).to_le_bytes());

    // data chunk
    wav.extend(b"data");
    wav.extend(&data_size.to_le_bytes());

    for s in samples {
        wav.extend(&s.to_le_bytes());
    }

    wav
}

fn make_wav_shoot() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.12;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let freq = 440.0 + t * 600.0;
        let phase = 2.0 * std::f64::consts::PI * freq * t;
        let env = (1.0 - t / duration).max(0.0);
        let sample = (phase.sin() * 0.8 + (2.0 * phase).sin() * 0.4) * env * 0.5;
        samples.push((sample * 32767.0) as i16);
    }
    make_wav(&samples, sample_rate)
}

fn make_wav_explosion() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.35;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let env = (1.0 - t / duration).max(0.0);
        let noise = (i as f64 * 12.7).sin() * (i as f64 * 7.3).cos()
            + (i as f64 * 19.1).sin() * 0.5
            + (i as f64 * 3.1).cos() * 0.3;
        let sample = noise * env * 0.4;
        samples.push((sample * 32767.0) as i16);
    }
    make_wav(&samples, sample_rate)
}

fn make_wav_jump() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.08;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let freq = 200.0 + t * 800.0;
        let phase = 2.0 * std::f64::consts::PI * freq * t;
        let env = (1.0 - t / duration).max(0.0);
        let sample = (phase.sin() * 0.7 + (3.0 * phase).sin() * 0.3) * env * 0.4;
        samples.push((sample * 32767.0) as i16);
    }
    make_wav(&samples, sample_rate)
}

fn make_wav_land() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.06;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let env = (1.0 - t / duration).max(0.0);
        let noise = (i as f64 * 5.0).sin() * env * 0.5;
        samples.push((noise * 32767.0) as i16);
    }
    make_wav(&samples, sample_rate)
}

fn make_wav_reload() -> Vec<u8> {
    let sample_rate = 22050;
    let duration = 0.03;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let env = (1.0 - t / duration).max(0.0);
        let sample = (t * 4000.0).sin() * env * 0.6;
        samples.push((sample * 32767.0) as i16);
    }
    make_wav(&samples, sample_rate)
}

fn create_shadow_texture(img: &Image) -> Texture2D {
    let mut shadow_image = img.clone();
    for y in 0..shadow_image.height() as u32 {
        for x in 0..shadow_image.width() as u32 {
            let color = shadow_image.get_pixel(x, y);
            if color.a > 0.0 {
                if color.r > 0.4 && color.g < 0.8 && color.b < 0.8 {
                    let new_color = Color::new(color.r * 0.5, color.g * 0.25, color.r * 0.9, color.a);
                    shadow_image.set_pixel(x, y, new_color);
                } else if color.r > 0.2 && color.g > 0.2 && color.b > 0.2 && (color.r - color.g).abs() < 0.25 && (color.g - color.b).abs() < 0.25 {
                    let new_color = Color::new(color.r * 0.45, color.g * 0.4, color.b * 0.75, color.a);
                    shadow_image.set_pixel(x, y, new_color);
                }
            }
        }
    }
    let texture = Texture2D::from_image(&shadow_image);
    texture.set_filter(FilterMode::Nearest);
    texture
}
