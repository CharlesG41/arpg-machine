//! Sound effects, synthesized at startup (no audio files): short blips,
//! pops, booms and chimes built from sines, noise and envelopes. Events from
//! the simulation pick the sound; each sound has a minimum gap so a hundred
//! kills in one frame stay one pop.
use arpg_machine::world::{EnemyKind, Event, Phase, World};
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

const RATE: u32 = 44_100;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Cast,
    Kill,
    BigKill,
    Hurt,
    Dash,
    Loot,
    Pickup,
    Shrine,
    BossSpawn,
    BossAlert,
    BossDeath,
    Boom,
    Zap,
    LevelUp,
    Defeat,
}

const ALL: [Sfx; 15] = [
    Sfx::Cast,
    Sfx::Kill,
    Sfx::BigKill,
    Sfx::Hurt,
    Sfx::Dash,
    Sfx::Loot,
    Sfx::Pickup,
    Sfx::Shrine,
    Sfx::BossSpawn,
    Sfx::BossAlert,
    Sfx::BossDeath,
    Sfx::Boom,
    Sfx::Zap,
    Sfx::LevelUp,
    Sfx::Defeat,
];

impl Sfx {
    /// Seconds before the same sound may play again, and its volume.
    fn gap_and_volume(self) -> (f64, f32) {
        match self {
            Sfx::Cast => (0.07, 0.10),
            Sfx::Kill => (0.035, 0.22),
            Sfx::BigKill => (0.08, 0.4),
            Sfx::Hurt => (0.14, 0.5),
            Sfx::Dash => (0.1, 0.35),
            Sfx::Loot => (0.2, 0.45),
            Sfx::Pickup => (0.1, 0.4),
            Sfx::Shrine => (0.2, 0.4),
            Sfx::BossSpawn => (0.5, 0.7),
            Sfx::BossAlert => (0.5, 0.55),
            Sfx::BossDeath => (0.5, 0.8),
            Sfx::Boom => (0.09, 0.35),
            Sfx::Zap => (0.06, 0.25),
            Sfx::LevelUp => (0.5, 0.5),
            Sfx::Defeat => (0.5, 0.5),
        }
    }

    /// Variants to rotate through, so repeats do not sound mechanical.
    fn variants(self) -> usize {
        match self {
            Sfx::Cast | Sfx::Kill => 4,
            _ => 1,
        }
    }
}

/// A tiny deterministic noise source.
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Samples of a sound, from -1 to 1.
fn synth(sfx: Sfx, variant: usize) -> Vec<f32> {
    let v = variant as f32;
    let mut noise = Noise(0x9E37_79B9 ^ (variant as u32 + 1) * 7919);
    let tau = std::f32::consts::TAU;
    let len = |secs: f32| (secs * RATE as f32) as usize;
    // A tone whose frequency slides from f0 to f1, with an exponential decay.
    let sweep = |f0: f32, f1: f32, secs: f32, decay: f32, out: &mut Vec<f32>, gain: f32| {
        let n = len(secs);
        let mut phase = 0.0f32;
        if out.len() < n {
            out.resize(n, 0.0);
        }
        for (i, o) in out.iter_mut().enumerate().take(n) {
            let t = i as f32 / RATE as f32;
            let k = i as f32 / n as f32;
            let f = f0 + (f1 - f0) * k;
            phase += tau * f / RATE as f32;
            let attack = (t / 0.004).min(1.0);
            *o += phase.sin() * (-t * decay).exp() * attack * gain;
        }
    };
    let mut out = Vec::new();
    match sfx {
        Sfx::Cast => sweep(620.0 + 90.0 * v, 900.0 + 120.0 * v, 0.06, 45.0, &mut out, 0.6),
        Sfx::Kill => {
            sweep(420.0 + 40.0 * v, 140.0, 0.08, 40.0, &mut out, 0.7);
            for (i, o) in out.iter_mut().enumerate().take(len(0.03)) {
                *o += noise.next() * 0.35 * (1.0 - i as f32 / len(0.03) as f32);
            }
        }
        Sfx::BigKill => {
            sweep(260.0, 60.0, 0.25, 14.0, &mut out, 0.9);
            for (i, o) in out.iter_mut().enumerate().take(len(0.12)) {
                *o += noise.next() * 0.5 * (1.0 - i as f32 / len(0.12) as f32);
            }
        }
        Sfx::Hurt => {
            sweep(150.0, 55.0, 0.18, 18.0, &mut out, 1.0);
            for (i, o) in out.iter_mut().enumerate().take(len(0.06)) {
                *o += noise.next() * 0.4 * (1.0 - i as f32 / len(0.06) as f32);
            }
        }
        Sfx::Dash => {
            let n = len(0.2);
            let mut low = 0.0f32;
            for i in 0..n {
                let k = i as f32 / n as f32;
                // Filtered noise that swells then fades: a whoosh.
                let cut = 0.05 + 0.25 * k;
                low += (noise.next() - low) * cut;
                out.push(low * (k * std::f32::consts::PI).sin() * 1.6);
            }
        }
        Sfx::Loot | Sfx::Pickup | Sfx::Shrine | Sfx::LevelUp => {
            let notes: &[f32] = match sfx {
                Sfx::Loot => &[880.0, 1318.5],
                Sfx::Pickup => &[1046.5, 1568.0],
                Sfx::Shrine => &[659.3, 830.6, 987.8, 1318.5],
                _ => &[523.3, 659.3, 784.0, 1046.5],
            };
            let step = if sfx == Sfx::LevelUp { 0.09 } else { 0.06 };
            let total = len(step * notes.len() as f32 + 0.5);
            out.resize(total, 0.0);
            for (k, &f) in notes.iter().enumerate() {
                let start = len(step * k as f32);
                for i in 0..total - start {
                    let t = i as f32 / RATE as f32;
                    let s = (tau * f * t).sin() + 0.3 * (tau * f * 2.0 * t).sin();
                    out[start + i] += s * (-t * 7.0).exp() * (t / 0.003).min(1.0) * 0.35;
                }
            }
        }
        Sfx::BossSpawn | Sfx::Boom | Sfx::BossDeath => {
            let (secs, f0, grit) = match sfx {
                Sfx::BossSpawn => (1.1, 90.0, 0.5),
                Sfx::BossDeath => (1.6, 110.0, 0.7),
                _ => (0.35, 140.0, 0.45),
            };
            sweep(f0, 35.0, secs, 4.0 / secs, &mut out, 1.0);
            let n = len(secs);
            let mut low = 0.0f32;
            for (i, o) in out.iter_mut().enumerate().take(n) {
                low += (noise.next() - low) * 0.08;
                let t = i as f32 / RATE as f32;
                *o += low * grit * 2.5 * (-t * 5.0 / secs).exp();
            }
            if sfx == Sfx::BossDeath {
                let chime = synth(Sfx::LevelUp, 0);
                let at = len(0.35);
                out.resize(out.len().max(at + chime.len()), 0.0);
                for (i, s) in chime.into_iter().enumerate() {
                    out[at + i] += s * 0.8;
                }
            }
        }
        Sfx::BossAlert => {
            // A low horn: two detuned saws.
            let n = len(0.7);
            for i in 0..n {
                let t = i as f32 / RATE as f32;
                let saw = |f: f32| 2.0 * ((t * f) % 1.0) - 1.0;
                let env = (t / 0.05).min(1.0) * (1.0 - i as f32 / n as f32);
                out.push((saw(110.0) + saw(111.5) + 0.6 * saw(165.0)) * 0.22 * env);
            }
        }
        Sfx::Zap => sweep(1400.0, 500.0, 0.07, 30.0, &mut out, 0.5),
        Sfx::Defeat => {
            for (k, f) in [392.0f32, 311.1, 261.6].into_iter().enumerate() {
                let start = len(0.22 * k as f32);
                let n = len(0.5);
                out.resize(out.len().max(start + n), 0.0);
                for i in 0..n {
                    let t = i as f32 / RATE as f32;
                    out[start + i] += (tau * f * t).sin() * (-t * 4.0).exp() * 0.4;
                }
            }
        }
    }
    out
}

/// 16-bit mono WAV bytes.
fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&RATE.to_le_bytes());
    b.extend_from_slice(&(RATE * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes());
    }
    b
}

pub struct Audio {
    sounds: Vec<(Sfx, Vec<Sound>)>,
    last: Vec<f64>,
    turn: Vec<usize>,
    pub muted: bool,
    /// Phase at the last frame, to catch the map ending.
    phase: Option<Phase>,
}

impl Audio {
    pub async fn load(muted: bool) -> Self {
        let mut sounds = Vec::new();
        for sfx in ALL {
            let mut variants = Vec::new();
            for v in 0..sfx.variants() {
                if let Ok(sound) = load_sound_from_bytes(&wav(&synth(sfx, v))).await {
                    variants.push(sound);
                }
            }
            sounds.push((sfx, variants));
        }
        Audio { last: vec![f64::MIN; ALL.len()], turn: vec![0; ALL.len()], sounds, muted, phase: None }
    }

    pub fn play(&mut self, sfx: Sfx) {
        if self.muted {
            return;
        }
        let i = ALL.iter().position(|&s| s == sfx).unwrap_or(0);
        let (gap, volume) = sfx.gap_and_volume();
        let now = macroquad::time::get_time();
        if now - self.last[i] < gap {
            return;
        }
        let variants = &self.sounds[i].1;
        if variants.is_empty() {
            return;
        }
        self.last[i] = now;
        self.turn[i] = (self.turn[i] + 1) % variants.len();
        play_sound(&variants[self.turn[i]], PlaySoundParams { looped: false, volume });
    }

    /// Sounds for this step's simulation events.
    pub fn consume(&mut self, world: &World) {
        for event in &world.events {
            match event {
                Event::Cast { .. } => self.play(Sfx::Cast),
                Event::Death { kind, .. } => match kind {
                    EnemyKind::Boss => self.play(Sfx::BossDeath),
                    EnemyKind::Brute => self.play(Sfx::BigKill),
                    _ => self.play(Sfx::Kill),
                },
                Event::PlayerHurt { .. } => self.play(Sfx::Hurt),
                Event::Dash { .. } => self.play(Sfx::Dash),
                Event::LootDropped { .. } => self.play(Sfx::Loot),
                Event::LootPicked { .. } => self.play(Sfx::Pickup),
                Event::ShrineClaimed { .. } => self.play(Sfx::Shrine),
                Event::BossSpawned { .. } => self.play(Sfx::BossSpawn),
                Event::BossAlert { .. } => self.play(Sfx::BossAlert),
                Event::Detonate { .. } | Event::Explosion { .. } => self.play(Sfx::Boom),
                Event::ShotHit { .. } => self.play(Sfx::Zap),
                _ => {}
            }
        }
        if self.phase != Some(world.phase) {
            if world.phase == Phase::Defeat {
                self.play(Sfx::Defeat);
            }
            self.phase = Some(world.phase);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_is_short_and_not_clipping_much() {
        for sfx in ALL {
            for v in 0..sfx.variants() {
                let s = synth(sfx, v);
                assert!(!s.is_empty() && s.len() < RATE as usize * 3, "length");
                let peak = s.iter().fold(0.0f32, |m, x| m.max(x.abs()));
                assert!(peak > 0.05 && peak < 1.6, "peak {peak}");
            }
        }
        assert_eq!(&wav(&[0.0; 10])[..4], b"RIFF");
    }
}
