/**
 * Drag a panel around its offset parent.
 *
 * The selected-unit panel sits over the battlefield, so wherever it parks it hides the
 * cells beneath it — a unit in that corner becomes unselectable. Letting the player
 * push it aside is cheaper than trying to guess a corner that is always free.
 *
 * Applies a translation rather than writing left/top so the element's own CSS anchoring
 * still decides where it starts, and clamps to the parent so it can never be dragged
 * somewhere it cannot be dragged back from.
 */
export type DragOpts = {
	/** Selector for the grab area. Anything outside it (buttons) still works normally. */
	handle?: string;
	/** Reset the offset when this value changes — e.g. a new unit is selected. */
	resetKey?: unknown;
};

export function draggable(node: HTMLElement, opts: DragOpts = {}) {
	let dx = 0;
	let dy = 0;
	let startX = 0;
	let startY = 0;
	let originX = 0;
	let originY = 0;
	let dragging = false;
	let lastReset = opts.resetKey;

	const apply = () => {
		node.style.transform = dx || dy ? `translate(${dx}px, ${dy}px)` : '';
	};

	/** Untranslated geometry, measured with the transform off so clamping has a fixed
	 *  reference. Deriving it from the live rect instead is wrong: that rect already
	 *  includes the previous offset. */
	let base = { left: 0, top: 0, width: 0, height: 0 };

	const measureBase = () => {
		const prev = node.style.transform;
		node.style.transform = '';
		const r = node.getBoundingClientRect();
		base = { left: r.left, top: r.top, width: r.width, height: r.height };
		node.style.transform = prev;
	};

	/** Keep the panel inside its offset parent, leaving a grabbable strip visible. */
	const clamp = () => {
		const parent = node.offsetParent as HTMLElement | null;
		if (!parent) return;
		const p = parent.getBoundingClientRect();
		const margin = 24;
		// At least `margin` of the panel stays over the parent on every side, and its
		// header can never go above the top edge where the grip would be unreachable.
		dx = Math.min(p.right - margin - base.left, Math.max(p.left + margin - base.left - base.width, dx));
		dy = Math.min(p.bottom - margin - base.top, Math.max(p.top - base.top, dy));
	};

	const onDown = (ev: PointerEvent) => {
		if (ev.button !== 0) return;
		const target = ev.target as HTMLElement | null;
		if (!(target instanceof Element)) return;
		// Never start a drag from a control.
		if (target.closest('button, a, input, select, textarea')) return;
		if (opts.handle && !target.closest(opts.handle)) return;
		dragging = true;
		measureBase();
		startX = ev.clientX;
		startY = ev.clientY;
		originX = dx;
		originY = dy;
		node.setPointerCapture?.(ev.pointerId);
		node.classList.add('dragging');
		ev.preventDefault();
		ev.stopPropagation();
	};

	const onMove = (ev: PointerEvent) => {
		if (!dragging) return;
		dx = originX + (ev.clientX - startX);
		dy = originY + (ev.clientY - startY);
		clamp();
		apply();
		ev.preventDefault();
		ev.stopPropagation();
	};

	const onUp = (ev: PointerEvent) => {
		if (!dragging) return;
		dragging = false;
		node.classList.remove('dragging');
		try {
			node.releasePointerCapture?.(ev.pointerId);
		} catch {
			/* already released */
		}
	};

	// The canvas underneath listens on window for pointer events; stop ours reaching it
	// so dragging the panel never paints a structure or pans the field.
	node.addEventListener('pointerdown', onDown);
	node.addEventListener('pointermove', onMove);
	node.addEventListener('pointerup', onUp);
	node.addEventListener('pointercancel', onUp);

	const onResize = () => {
		measureBase();
		clamp();
		apply();
	};
	window.addEventListener('resize', onResize);

	return {
		update(next: DragOpts) {
			opts = next;
			if (next.resetKey !== lastReset) {
				lastReset = next.resetKey;
				// Deliberately keep the offset: a player who moved the panel out of the way
				// wants it to stay there as they select other units.
				measureBase();
				clamp();
				apply();
			}
		},
		destroy() {
			node.removeEventListener('pointerdown', onDown);
			node.removeEventListener('pointermove', onMove);
			node.removeEventListener('pointerup', onUp);
			node.removeEventListener('pointercancel', onUp);
			window.removeEventListener('resize', onResize);
		}
	};
}
