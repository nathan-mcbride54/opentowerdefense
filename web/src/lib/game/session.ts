import init, { WasmGame } from '$lib/wasm/otd';
import { createAudio } from './audio';
import { actionForKey, normalizeKey, P2_KEYS, type ActionId } from './keys';
import { BattlefieldRenderer } from './renderer';
import { applyUiScale, loadSettings, patchSettings, subscribeSettings } from './settings';
import {
	SPEED_STEPS,
	writeBestWave,
	type CatalogItem,
	type MapStatic,
	type Snapshot,
	type Speed,
	type StrikeItem
} from './types';

const DT = 1 / 60;
const MAX_STEPS = 10;
const TAP_PX = 10;

export const WORKSHOP_STORAGE = 'otd-workshop-map';
export const PACK_STORAGE = 'otd-workshop-pack';
export const REPLAY_STORAGE = 'otd-replay-watch';

export interface SessionOpts {
	mapId?: number;
	modifierId?: number;
	utcDay?: number;
	mapJson?: string;
	missionId?: number;
	challengeId?: number;
	seedHex?: string;
	packJson?: string;
	coop?: boolean;
	replayJson?: string;
}

export interface SessionExtras {
	paused: boolean;
	speed: Speed;
	catalog: CatalogItem[];
	strikes: StrikeItem[];
	coop: boolean;
	watch: boolean;
}

export interface Session {
	destroy: () => void;
	setBuild: (id: number, player?: number) => void;
	setStrike: (id: number, player?: number) => void;
	upgrade: (player?: number) => void;
	sell: (player?: number) => void;
	callWave: () => void;
	cycleTargeting: (player?: number) => void;
	convert: (player?: number) => void;
	repair: (player?: number) => void;
	lift: (player?: number) => void;
	overcharge: (player?: number) => void;
	togglePause: () => void;
	cycleSpeed: () => void;
	resetView: () => void;
	replayJson: () => string;
	get paused(): boolean;
	get speed(): Speed;
}

export async function createSession(
	canvas: HTMLCanvasElement,
	onSnap: (snap: Snapshot, extras: SessionExtras) => void,
	opts: SessionOpts | number = 0
): Promise<Session> {
	const resolved: SessionOpts = typeof opts === 'number' ? { mapId: opts } : opts;
	await init();
	const watch = Boolean(resolved.replayJson);
	const sourceReplay = resolved.replayJson ?? null;
	const game = resolved.replayJson
		? WasmGame.fromReplay(resolved.replayJson)
		: resolved.mapJson
			? WasmGame.fromMapJson(resolved.mapJson, resolved.modifierId ?? 0)
			: resolved.missionId != null
				? WasmGame.withMission(resolved.missionId)
				: resolved.challengeId != null
					? WasmGame.withChallenge(resolved.challengeId)
					: resolved.utcDay != null
						? WasmGame.withDaily(resolved.utcDay)
						: resolved.seedHex
							? WasmGame.withSeed(resolved.mapId ?? 0, resolved.modifierId ?? 0, resolved.seedHex)
							: WasmGame.withMatch(resolved.mapId ?? 0, resolved.modifierId ?? 0);
	if (resolved.packJson && !watch) {
		game.applyPack(resolved.packJson);
	}
	const catalog = JSON.parse(game.matchCatalog()) as CatalogItem[];
	const strikes = JSON.parse(game.matchStrikes()) as StrikeItem[];
	const map = JSON.parse(game.mapStatic()) as MapStatic;
	const renderer = new BattlefieldRenderer(canvas);
	renderer.setMap(map);
	const coop = Boolean(resolved.coop) && !watch;
	const core = map.core[0];
	let p2x = core ? Math.floor(core[0]) : Math.floor(map.w / 2);
	let p2y = core ? Math.floor(core[1]) : Math.floor(map.h / 2);

	let settings = loadSettings();
	applyUiScale(settings.uiScale);
	const unsubSettings = subscribeSettings(() => {
		settings = loadSettings();
		applyUiScale(settings.uiScale);
	});
	const audio = createAudio(() => settings);
	const unlock = () => audio.unlock();

	let destroyed = false;
	let paused = false;
	let speed: Speed = 1;
	let acc = 0;
	let last = performance.now();
	let raf = 0;
	let lastSnap: Snapshot | null = null;
	let recordedDefeat = false;

	const pointers = new Map<number, { x: number; y: number; ox: number; oy: number }>();
	let panned = false;
	let pinch0: { dist: number; zoomMidX: number; zoomMidY: number } | null = null;
	let miniNav = false;
	let lastPaint: { x: number; y: number } | null = null;
	let p2Paint = false;
	let handBuild = 0;
	let handStrike = 0;

	const extras = (): SessionExtras => ({ paused, speed, catalog, strikes, coop, watch });

	const emit = (snap: Snapshot) => {
		audio.onSnap(lastSnap, snap);
		lastSnap = snap;
		handBuild = snap.build;
		handStrike = snap.strike;
		if (snap.defeated && !recordedDefeat) {
			recordedDefeat = true;
			writeBestWave(snap.mapId, snap.wave, snap.modifierId ?? 0);
		}
		onSnap(snap, extras());
	};

	const read = () => JSON.parse(game.snapshot()) as Snapshot;

	const loop = (now: number) => {
		if (destroyed) return;
		const rawDt = Math.min(0.05, (now - last) / 1000);
		last = now;
		if (!paused) {
			acc += rawDt * speed;
			let steps = 0;
			while (acc >= DT && steps < MAX_STEPS) {
				if (watch) game.stepRecorded();
				else game.step();
				acc -= DT;
				steps += 1;
			}
			if (steps === MAX_STEPS) acc = 0;
		}
		if (coop) game.setHoverP(1, p2x, p2y);
		const snap = read();
		renderer.setLook(settings.palette, settings.reducedFx);
		renderer.render(snap, paused);
		emit(snap);
		raf = requestAnimationFrame(loop);
	};

	const onResize = () => renderer.resize();
	const ro = new ResizeObserver(onResize);
	ro.observe(canvas);

	const cellFromEvent = (ev: PointerEvent) => renderer.cellAt(ev.clientX, ev.clientY);

	const dist = (
		a: { x: number; y: number },
		b: { x: number; y: number }
	) => Math.hypot(a.x - b.x, a.y - b.y);

	const painting = () => !watch && handBuild > 0 && handStrike === 0;

	const paintCell = (x: number, y: number) => {
		if (lastPaint && lastPaint.x === x && lastPaint.y === y) return;
		game.click(x, y);
		lastPaint = { x, y };
	};

	const onPointerDown = (ev: PointerEvent) => {
		if (ev.button === 2) return;
		canvas.setPointerCapture(ev.pointerId);
		pointers.set(ev.pointerId, { x: ev.clientX, y: ev.clientY, ox: ev.clientX, oy: ev.clientY });
		panned = false;
		miniNav = false;
		lastPaint = null;
		const mini = renderer.minimapCell(ev.clientX, ev.clientY);
		if (mini) {
			renderer.lookAt(mini.x, mini.y);
			miniNav = true;
			panned = true;
			return;
		}
		if (pointers.size === 2) {
			const pts = [...pointers.values()];
			pinch0 = {
				dist: dist(pts[0], pts[1]),
				zoomMidX: (pts[0].x + pts[1].x) / 2,
				zoomMidY: (pts[0].y + pts[1].y) / 2
			};
		}
		const cell = cellFromEvent(ev);
		if (!watch && cell && pointers.size === 1) game.setHover(cell.x, cell.y);
		if (painting() && cell && pointers.size === 1 && ev.button === 0) {
			paintCell(cell.x, cell.y);
		}
	};

	const onPointerMove = (ev: PointerEvent) => {
		if (miniNav) {
			const mini = renderer.minimapCell(ev.clientX, ev.clientY);
			if (mini) renderer.lookAt(mini.x, mini.y);
			return;
		}
		const prev = pointers.get(ev.pointerId);
		if (prev) {
			const now = { x: ev.clientX, y: ev.clientY, ox: prev.ox, oy: prev.oy };
			if (pointers.size === 2 && pinch0) {
				pointers.set(ev.pointerId, now);
				const pts = [...pointers.values()];
				const d = dist(pts[0], pts[1]);
				if (pinch0.dist > 8) {
					renderer.zoomAt(pinch0.zoomMidX, pinch0.zoomMidY, d / pinch0.dist);
					pinch0 = {
						dist: d,
						zoomMidX: (pts[0].x + pts[1].x) / 2,
						zoomMidY: (pts[0].y + pts[1].y) / 2
					};
					panned = true;
				}
				return;
			}
			const dx = now.x - prev.x;
			const dy = now.y - prev.y;
			const dragging = ev.buttons === 1 || ev.buttons === 4 || ev.pointerType === 'touch';
			const total = Math.hypot(now.x - prev.ox, now.y - prev.oy);
			if (painting() && pointers.size === 1 && ev.buttons === 1) {
				pointers.set(ev.pointerId, now);
				const cell = cellFromEvent(ev);
				if (cell) {
					game.setHover(cell.x, cell.y);
					paintCell(cell.x, cell.y);
				}
				return;
			}
			if (dragging && (panned || total > TAP_PX)) {
				renderer.panBy(dx, dy);
				panned = true;
			}
			pointers.set(ev.pointerId, now);
		}
		if (!watch && pointers.size <= 1 && !panned) {
			const cell = cellFromEvent(ev);
			if (cell) game.setHover(cell.x, cell.y);
			else game.clearHover();
		}
	};

	const onPointerUp = (ev: PointerEvent) => {
		const wasPinch = pointers.size >= 2;
		pointers.delete(ev.pointerId);
		pinch0 = null;
		if (ev.button === 1) return;
		if (!watch && !panned && !wasPinch && ev.button === 0 && lastPaint == null) {
			const cell = cellFromEvent(ev);
			if (cell) game.click(cell.x, cell.y);
		}
		if (pointers.size === 0) {
			panned = false;
			miniNav = false;
			lastPaint = null;
		}
		try {
			canvas.releasePointerCapture(ev.pointerId);
		} catch {
			/* already released */
		}
	};

	const onLeave = () => {
		if (!watch && pointers.size === 0) game.clearHover();
	};

	const onContext = (ev: MouseEvent) => {
		ev.preventDefault();
		if (!watch) {
			game.setStrike(0);
			game.setBuild(0);
			handBuild = 0;
			handStrike = 0;
		}
	};

	const onWheel = (ev: WheelEvent) => {
		ev.preventDefault();
		const factor = ev.deltaY < 0 ? 1.12 : 1 / 1.12;
		renderer.zoomAt(ev.clientX, ev.clientY, factor);
	};

	const cycle = () => {
		const i = SPEED_STEPS.indexOf(speed);
		speed = SPEED_STEPS[(i + 1) % SPEED_STEPS.length];
	};

	const runAction = (id: ActionId, player = 0) => {
		if (watch && id !== 'pause' && id !== 'speed' && id !== 'mute' && id !== 'viewReset') {
			if (id === 'cancel') {
				paused = !paused;
				return;
			}
			return;
		}
		const p = player === 1 ? 1 : 0;
		switch (id) {
			case 'build1':
			case 'build2':
			case 'build3':
			case 'build4':
			case 'build5':
			case 'build6':
			case 'build7':
			case 'build8':
			case 'build9': {
				const n = Number(id.slice(5));
				game.setBuildP(p, n);
				if (p === 0) {
					handBuild = n;
					handStrike = 0;
				}
				break;
			}
			case 'build10':
				game.setBuildP(p, 10);
				if (p === 0) {
					handBuild = 10;
					handStrike = 0;
				}
				break;
			case 'strike1':
				game.setStrikeP(p, 1);
				if (p === 0) {
					handStrike = 1;
					handBuild = 0;
				}
				break;
			case 'strike2':
				game.setStrikeP(p, 2);
				if (p === 0) {
					handStrike = 2;
					handBuild = 0;
				}
				break;
			case 'strike3':
				game.setStrikeP(p, 3);
				if (p === 0) {
					handStrike = 3;
					handBuild = 0;
				}
				break;
			case 'upgrade':
				game.upgradeP(p);
				break;
			case 'sell':
				game.sellP(p);
				break;
			case 'target':
				game.cycleTargetingP(p);
				break;
			case 'convert':
				game.convertP(p);
				break;
			case 'repair':
				game.repairP(p);
				break;
			case 'move':
				game.liftP(p);
				if (p === 0) {
					handBuild = 0;
					handStrike = 0;
				}
				break;
			case 'overcharge':
				game.overchargeP(p);
				break;
			case 'call':
				game.callWave();
				break;
			case 'pause':
				paused = !paused;
				break;
			case 'speed':
				cycle();
				break;
			case 'mute':
				patchSettings({ mute: !loadSettings().mute });
				break;
			case 'cancel':
				if (!game.cancelP(p)) paused = !paused;
				if (p === 0) {
					handBuild = 0;
					handStrike = 0;
				}
				break;
			case 'viewReset':
				renderer.resetView();
				break;
		}
	};

	const onKeyDown = (ev: KeyboardEvent) => {
		const el = ev.target as HTMLElement | null;
		if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT')) {
			return;
		}
		const key = normalizeKey(ev);
		if (coop) {
			if (key === 'arrowup') {
				p2y = Math.max(0, p2y - 1);
				if (p2Paint) game.clickP(1, p2x, p2y);
				ev.preventDefault();
				return;
			}
			if (key === 'arrowdown') {
				p2y = Math.min(map.h - 1, p2y + 1);
				if (p2Paint) game.clickP(1, p2x, p2y);
				ev.preventDefault();
				return;
			}
			if (key === 'arrowleft') {
				p2x = Math.max(0, p2x - 1);
				if (p2Paint) game.clickP(1, p2x, p2y);
				ev.preventDefault();
				return;
			}
			if (key === 'arrowright') {
				p2x = Math.min(map.w - 1, p2x + 1);
				if (p2Paint) game.clickP(1, p2x, p2y);
				ev.preventDefault();
				return;
			}
			if (key === 'enter') {
				p2Paint = true;
				game.clickP(1, p2x, p2y);
				ev.preventDefault();
				return;
			}
		}
		if (ev.repeat && ev.key !== ' ') return;
		const p1 = actionForKey(settings.keys, key);
		if (p1) {
			runAction(p1);
			ev.preventDefault();
			return;
		}
		if (coop) {
			const p2 = actionForKey(P2_KEYS, key);
			if (p2 && p2 !== 'pause' && p2 !== 'speed' && p2 !== 'mute' && p2 !== 'viewReset') {
				runAction(p2, 1);
				ev.preventDefault();
			}
		}
	};

	const onKeyUp = (ev: KeyboardEvent) => {
		if (normalizeKey(ev) === 'enter') p2Paint = false;
	};

	canvas.addEventListener('pointermove', onPointerMove);
	canvas.addEventListener('pointerleave', onLeave);
	canvas.addEventListener('pointerdown', onPointerDown);
	canvas.addEventListener('pointerup', onPointerUp);
	canvas.addEventListener('pointercancel', onPointerUp);
	canvas.addEventListener('contextmenu', onContext);
	canvas.addEventListener('wheel', onWheel, { passive: false });
	window.addEventListener('keydown', onKeyDown);
	window.addEventListener('keyup', onKeyUp);
	window.addEventListener('pointerdown', unlock);
	window.addEventListener('keydown', unlock);

	if (watch) game.pumpRecorded();
	emit(read());
	raf = requestAnimationFrame(loop);

	return {
		destroy() {
			destroyed = true;
			cancelAnimationFrame(raf);
			ro.disconnect();
			unsubSettings();
			audio.destroy();
			canvas.removeEventListener('pointermove', onPointerMove);
			canvas.removeEventListener('pointerleave', onLeave);
			canvas.removeEventListener('pointerdown', onPointerDown);
			canvas.removeEventListener('pointerup', onPointerUp);
			canvas.removeEventListener('pointercancel', onPointerUp);
			canvas.removeEventListener('contextmenu', onContext);
			canvas.removeEventListener('wheel', onWheel);
			window.removeEventListener('keydown', onKeyDown);
			window.removeEventListener('keyup', onKeyUp);
			window.removeEventListener('pointerdown', unlock);
			window.removeEventListener('keydown', unlock);
			game.free();
		},
		setBuild(id: number, player = 0) {
			if (watch) return;
			game.setBuildP(player === 1 ? 1 : 0, id);
			if (player !== 1) {
				handBuild = id;
				if (id > 0) handStrike = 0;
			}
		},
		setStrike(id: number, player = 0) {
			if (watch) return;
			game.setStrikeP(player === 1 ? 1 : 0, id);
			if (player !== 1) {
				handStrike = id;
				if (id > 0) handBuild = 0;
			}
		},
		upgrade(player = 0) {
			if (watch) return;
			game.upgradeP(player === 1 ? 1 : 0);
		},
		sell(player = 0) {
			if (watch) return;
			game.sellP(player === 1 ? 1 : 0);
		},
		callWave() {
			if (watch) return;
			game.callWave();
		},
		cycleTargeting(player = 0) {
			if (watch) return;
			game.cycleTargetingP(player === 1 ? 1 : 0);
		},
		convert(player = 0) {
			if (watch) return;
			game.convertP(player === 1 ? 1 : 0);
		},
		repair(player = 0) {
			if (watch) return;
			game.repairP(player === 1 ? 1 : 0);
		},
		lift(player = 0) {
			if (watch) return;
			game.liftP(player === 1 ? 1 : 0);
			if (player !== 1) {
				handBuild = 0;
				handStrike = 0;
			}
		},
		overcharge(player = 0) {
			if (watch) return;
			game.overchargeP(player === 1 ? 1 : 0);
		},
		togglePause() {
			paused = !paused;
			if (lastSnap) emit(lastSnap);
		},
		cycleSpeed() {
			cycle();
			if (lastSnap) emit(lastSnap);
		},
		resetView() {
			renderer.resetView();
		},
		replayJson() {
			if (watch && sourceReplay) return sourceReplay;
			return game.replay();
		},
		get paused() {
			return paused;
		},
		get speed() {
			return speed;
		}
	};
}
