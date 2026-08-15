import type { Snapshot } from './types';

type Tone = OscillatorType;

export interface AudioBus {
	unlock: () => void;
	onSnap: (prev: Snapshot | null, next: Snapshot) => void;
	destroy: () => void;
}

export function createAudio(
	opts: () => { mute: boolean; volume: number; reducedFx: boolean }
): AudioBus {
	let ctx: AudioContext | null = null;
	let master: GainNode | null = null;
	let lastGun = 0;
	let lastKill = 0;

	const ensure = () => {
		if (ctx) return ctx;
		const AC =
			window.AudioContext ||
			(window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
		if (!AC) return null;
		ctx = new AC();
		master = ctx.createGain();
		master.connect(ctx.destination);
		return ctx;
	};

	const beep = (
		freq: number,
		dur: number,
		type: Tone,
		gain: number,
		slide = 0,
		delay = 0
	) => {
		const c = ensure();
		if (!c || !master) return;
		const { mute, volume } = opts();
		if (mute || volume <= 0.01) return;
		const t0 = c.currentTime + delay;
		const osc = c.createOscillator();
		const g = c.createGain();
		osc.type = type;
		osc.frequency.setValueAtTime(freq, t0);
		if (slide) osc.frequency.exponentialRampToValueAtTime(Math.max(40, freq + slide), t0 + dur);
		g.gain.setValueAtTime(0.0001, t0);
		g.gain.exponentialRampToValueAtTime(gain * volume, t0 + 0.012);
		g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
		osc.connect(g);
		g.connect(master);
		osc.start(t0);
		osc.stop(t0 + dur + 0.02);
	};

	return {
		unlock() {
			const c = ensure();
			void c?.resume();
		},
		onSnap(prev, next) {
			if (!prev) {
				beep(196, 0.12, 'triangle', 0.08);
				beep(294, 0.16, 'sine', 0.05, 0, 0.08);
				return;
			}
			const { reducedFx } = opts();
			if (next.kills > prev.kills) {
				const now = performance.now();
				if (now - lastKill > 40) {
					lastKill = now;
					beep(520, 0.04, 'sine', 0.045, 80);
				}
			}
			if (next.leaks > prev.leaks) {
				beep(140, 0.22, 'sawtooth', 0.12, -40);
			}
			if (next.status === 'incoming' && prev.status !== 'incoming') {
				beep(392, 0.1, 'square', 0.07);
				beep(523, 0.14, 'square', 0.05, 0, 0.1);
			}
			if (next.defeated && !prev.defeated) {
				beep(110, 0.45, 'sawtooth', 0.14, -60);
			}
			const placed = next.fx.filter((f) => f.kind === 'place').length;
			const prevPlaced = prev.fx.filter((f) => f.kind === 'place').length;
			if (placed > prevPlaced) beep(240, 0.06, 'square', 0.07, 40);
			const up = next.fx.filter((f) => f.kind === 'upgrade').length;
			const prevUp = prev.fx.filter((f) => f.kind === 'upgrade').length;
			if (up > prevUp) {
				beep(440, 0.05, 'triangle', 0.06);
				beep(660, 0.07, 'triangle', 0.05, 0, 0.05);
			}
			const sold = next.fx.filter((f) => f.kind === 'sell').length;
			const prevSold = prev.fx.filter((f) => f.kind === 'sell').length;
			if (sold > prevSold) beep(180, 0.1, 'triangle', 0.06, -80);
			if (!reducedFx) {
				const guns = next.fx.filter((f) => f.kind === 'muzzle').length;
				const prevGuns = prev.fx.filter((f) => f.kind === 'muzzle').length;
				if (guns > prevGuns) {
					const now = performance.now();
					if (now - lastGun > 70) {
						lastGun = now;
						beep(1680, 0.018, 'sine', 0.025);
					}
				}
				const boom = next.fx.some(
					(f) =>
						(f.kind === 'satchel' || f.kind === 'orbital' || f.kind === 'burst') &&
						!prev.fx.some((p) => p.kind === f.kind && p.x === f.x && p.y === f.y)
				);
				if (boom) beep(90, 0.18, 'sawtooth', 0.1, -30);
				const roar = next.fx.some(
					(f) => f.kind === 'roar' && !prev.fx.some((p) => p.kind === 'roar' && p.x === f.x && p.y === f.y)
				);
				if (roar) beep(70, 0.28, 'sawtooth', 0.12, -20);
			}
			const charged = next.fx.filter((f) => f.kind === 'overcharge').length;
			const prevCharged = prev.fx.filter((f) => f.kind === 'overcharge').length;
			if (charged > prevCharged) {
				beep(520, 0.05, 'square', 0.06);
				beep(780, 0.08, 'square', 0.05, 0, 0.04);
			}
		},
		destroy() {
			void ctx?.close();
			ctx = null;
			master = null;
		}
	};
}
