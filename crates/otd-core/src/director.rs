use crate::defs::CreepKind;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WaveScript {
    Mixed,
    Swarm,
    Air,
    Armor,
    Split,
    Colossus,
}

impl WaveScript {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mixed => "Mixed",
            Self::Swarm => "Swarm",
            Self::Air => "Air corridor",
            Self::Armor => "Armor column",
            Self::Split => "Split",
            Self::Colossus => "Colossus",
        }
    }

    pub fn for_wave(wave: u32, ground_only: bool) -> Self {
        if wave > 0 && wave.is_multiple_of(10) {
            return Self::Colossus;
        }
        if ground_only {
            return match wave % 5 {
                2 => Self::Swarm,
                3 => Self::Armor,
                4 => Self::Split,
                0 => Self::Swarm,
                _ => Self::Mixed,
            };
        }
        match wave % 6 {
            2 => Self::Swarm,
            3 => Self::Air,
            4 => Self::Armor,
            5 => Self::Split,
            _ => Self::Mixed,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WavePlan {
    pub script: WaveScript,
    pub runners: u32,
    pub lorries: u32,
    pub bulwarks: u32,
    pub wasps: u32,
    pub mites: u32,
    pub colossus: u32,
    pub medics: u32,
    pub shades: u32,
    pub flickers: u32,
}

impl WavePlan {
    pub fn for_wave(wave: u32, ground_only: bool) -> Self {
        let script = WaveScript::for_wave(wave, ground_only);
        let mut plan = match script {
            WaveScript::Mixed => Self {
                script,
                runners: 6 + wave * 2,
                lorries: if wave >= 2 { wave } else { 0 },
                bulwarks: if wave >= 5 { (wave - 3) / 2 } else { 0 },
                wasps: if wave >= 4 { 2 + wave / 2 } else { 0 },
                mites: if wave >= 3 { wave / 2 } else { 0 },
                colossus: 0,
                medics: if wave >= 6 { 1 + (wave - 6) / 4 } else { 0 },
                shades: if wave >= 6 { 1 + wave / 4 } else { 0 },
                flickers: if wave >= 4 { 1 + wave / 5 } else { 0 },
            },
            WaveScript::Swarm => Self {
                script,
                runners: 10 + wave * 3,
                lorries: wave / 3,
                bulwarks: 0,
                wasps: 0,
                mites: 6 + wave * 2,
                colossus: 0,
                medics: 0,
                shades: 0,
                flickers: 0,
            },
            WaveScript::Air => Self {
                script,
                runners: 4 + wave,
                lorries: 0,
                bulwarks: 0,
                wasps: 8 + wave,
                mites: wave,
                colossus: 0,
                medics: 0,
                shades: 0,
                flickers: 0,
            },
            WaveScript::Armor => Self {
                script,
                runners: 4 + wave,
                lorries: 4 + wave,
                bulwarks: 1 + wave / 2,
                wasps: 0,
                mites: 0,
                colossus: 0,
                medics: 1 + wave / 5,
                shades: if wave >= 8 { 1 + wave / 8 } else { 0 },
                flickers: 0,
            },
            WaveScript::Split => Self {
                script,
                runners: 4 + wave,
                lorries: wave / 2,
                bulwarks: 0,
                wasps: 0,
                mites: 12 + wave * 3,
                colossus: 0,
                medics: 0,
                shades: 2 + wave / 3,
                flickers: 2 + wave / 4,
            },
            WaveScript::Colossus => Self {
                script,
                runners: 6 + wave,
                lorries: wave,
                bulwarks: wave / 4,
                wasps: 2 + wave / 3,
                mites: wave,
                colossus: 1 + wave / 30,
                medics: 2 + wave / 20,
                shades: 1 + wave / 15,
                flickers: 1 + wave / 20,
            },
        };
        if ground_only {
            plan.lorries += plan.wasps;
            plan.wasps = 0;
        }
        plan
    }

    pub fn total(self) -> u32 {
        self.runners
            + self.lorries
            + self.bulwarks
            + self.wasps
            + self.mites
            + self.colossus
            + self.medics
            + self.shades
            + self.flickers
    }

    pub fn kinds(self) -> Vec<CreepKind> {
        let mut kinds = Vec::with_capacity(self.total() as usize);
        push_n(&mut kinds, CreepKind::Runner, self.runners);
        push_n(&mut kinds, CreepKind::Lorry, self.lorries);
        push_n(&mut kinds, CreepKind::Bulwark, self.bulwarks);
        push_n(&mut kinds, CreepKind::Wasp, self.wasps);
        push_n(&mut kinds, CreepKind::Mite, self.mites);
        push_n(&mut kinds, CreepKind::Medic, self.medics);
        push_n(&mut kinds, CreepKind::Shade, self.shades);
        push_n(&mut kinds, CreepKind::Flicker, self.flickers);
        push_n(&mut kinds, CreepKind::Colossus, self.colossus);
        kinds
    }
}

fn push_n(out: &mut Vec<CreepKind>, kind: CreepKind, n: u32) {
    for _ in 0..n {
        out.push(kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_one_is_mixed_runners() {
        let p = WavePlan::for_wave(1, false);
        assert_eq!(p.script, WaveScript::Mixed);
        assert_eq!(p.runners, 8);
        assert_eq!(p.mites, 0);
        assert_eq!(p.wasps, 0);
        assert_eq!(p.medics, 0);
        assert_eq!(p.shades, 0);
        assert_eq!(p.flickers, 0);
    }

    #[test]
    fn wave_two_is_swarm() {
        let p = WavePlan::for_wave(2, false);
        assert_eq!(p.script, WaveScript::Swarm);
        assert!(p.mites > 0);
    }

    #[test]
    fn wave_ten_is_colossus() {
        let p = WavePlan::for_wave(10, false);
        assert_eq!(p.script, WaveScript::Colossus);
        assert!(p.colossus >= 1);
    }

    #[test]
    fn ground_only_never_schedules_wasps() {
        for w in 1..40 {
            let p = WavePlan::for_wave(w, true);
            assert_eq!(p.wasps, 0, "wave {w}");
        }
    }

    #[test]
    fn mixed_late_brings_shades() {
        let p = WavePlan::for_wave(7, false);
        assert_eq!(p.script, WaveScript::Mixed);
        assert!(p.shades > 0);
    }

    #[test]
    fn split_always_has_shades() {
        let p = WavePlan::for_wave(5, false);
        assert_eq!(p.script, WaveScript::Split);
        assert!(p.shades > 0);
        assert!(p.flickers > 0);
    }

    #[test]
    fn mixed_late_brings_flickers() {
        let p = WavePlan::for_wave(7, false);
        assert_eq!(p.script, WaveScript::Mixed);
        assert!(p.flickers > 0);
    }
}
