export function trapFocus(node: HTMLElement) {
	const previous = document.activeElement as HTMLElement | null;

	function handleKeydown(event: KeyboardEvent) {
		if (event.key !== 'Tab') return;

		const current = document.activeElement;

		const elements = node.querySelectorAll(
			'a[href], button, input, textarea, select, details, [tabindex]:not([tabindex="-1"])'
		);
		const focusable = Array.from(elements).filter((el) => {
			return !el.hasAttribute('disabled') && el.getAttribute('aria-hidden') !== 'true';
		}) as HTMLElement[];

		if (focusable.length === 0) {
			event.preventDefault();
			return;
		}

		const first = focusable[0];
		const last = focusable[focusable.length - 1];

		if (event.shiftKey && current === first) {
			last.focus();
			event.preventDefault();
		} else if (!event.shiftKey && current === last) {
			first.focus();
			event.preventDefault();
		}
	}

	node.addEventListener('keydown', handleKeydown);

	// Initial focus
	const focusable = node.querySelectorAll(
		'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
	);
	// Try to find the first non-disabled, visible element to focus
	// Simple check for now
	const firstFocusable = Array.from(focusable).find(
		(el) => !el.hasAttribute('disabled') && el.getAttribute('aria-hidden') !== 'true'
	) as HTMLElement | undefined;

	if (firstFocusable) {
		firstFocusable.focus();
	} else {
		// Fallback to the container itself if it has tabindex, or just don't focus anything yet
		node.focus();
	}

	return {
		destroy() {
			node.removeEventListener('keydown', handleKeydown);
			previous?.focus();
		}
	};
}