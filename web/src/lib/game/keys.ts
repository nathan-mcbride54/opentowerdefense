export const ACTION_IDS = [
	'build1',
	'build2',
	'build3',
	'build4',
	'build5',
	'build6',
	'build7',
	'build8',
	'build9',
	'build10',
	'strike1',
	'strike2',
	'strike3',
	'upgrade',
	'sell',
	'target',
	'convert',
	'repair',
	'move',
	'overcharge',
	'call',
	'pause',
	'speed',
	'mute',
	'cancel',
	'viewReset'
] as const;

export type ActionId = (typeof ACTION_IDS)[number];

export const ACTION_LABELS: Record<ActionId, string> = {
	build1: 'Barricade',
	build2: 'Autocannon',
	build3: 'Howitzer',
	build4: 'Skystinger',
	build5: 'Inferno',
	build6: 'Arc Lance',
	build7: 'Pulse Array',
	build8: 'Helios',
	build9: 'Swarm Rack',
	build10: 'Siege Rail',
	strike1: 'Satchel',
	strike2: 'Overload',
	strike3: 'Orbital',
	upgrade: 'Upgrade',
	sell: 'Sell',
	target: 'Cycle targeting',
	convert: 'Helios air',
	repair: 'Repair relay',
	move: 'Move structure',
	overcharge: 'Overcharge',
	call: 'Call wave',
	pause: 'Pause',
	speed: 'Speed',
	mute: 'Mute',
	cancel: 'Cancel',
	viewReset: 'Reset view'
};

export const DEFAULT_KEYS: Record<ActionId, string> = {
	build1: '1',
	build2: '2',
	build3: '3',
	build4: '4',
	build5: '5',
	build6: '6',
	build7: '7',
	build8: '8',
	build9: '9',
	build10: '0',
	strike1: 'q',
	strike2: 'w',
	strike3: 'e',
	upgrade: 'u',
	sell: 'x',
	target: 't',
	convert: 'c',
	repair: 'v',
	move: 'g',
	overcharge: 'b',
	call: 'n',
	pause: 'space',
	speed: 'f',
	mute: 'm',
	cancel: 'escape',
	viewReset: 'home'
};

export function normalizeKey(ev: KeyboardEvent): string {
	if (ev.key === ' ') return 'space';
	return ev.key.length === 1 ? ev.key.toLowerCase() : ev.key.toLowerCase();
}

export function formatKey(key: string): string {
	if (!key) return '—';
	if (key === 'space') return 'Space';
	if (key === 'escape') return 'Esc';
	if (key === 'home') return 'Home';
	if (key === 'backspace') return 'Bksp';
	if (key === 'enter') return 'Enter';
	if (key === 'arrowup') return '↑';
	if (key === 'arrowdown') return '↓';
	if (key === 'arrowleft') return '←';
	if (key === 'arrowright') return '→';
	if (key.length === 1) return key.toUpperCase();
	return key;
}

export function mergeKeys(raw?: Partial<Record<ActionId, string>>): Record<ActionId, string> {
	const next = { ...DEFAULT_KEYS };
	if (!raw) return next;
	for (const id of ACTION_IDS) {
		const v = raw[id];
		if (typeof v === 'string' && v.length > 0) next[id] = v;
	}
	return next;
}

export function actionForKey(
	keys: Record<ActionId, string>,
	key: string
): ActionId | null {
	for (const id of ACTION_IDS) {
		if (keys[id] === key) return id;
	}
	return null;
}
