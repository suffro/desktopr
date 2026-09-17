export const wait = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

// Accepts "example.com", "https://example.com/path" or "localhost:3000" and returns an
// absolute URL. Local hosts default to http, everything else to https.
export function normalizeUrl(input: string): string {
	const trimmed = input.trim();
	if (!trimmed) return '';
	const [base, ...query] = trimmed.split('?');
	const host = base.trim().replace(/^https?:\/\//, '');
	const isLocal = host.startsWith('localhost') || host.startsWith('127.0.0.1');
	const search = query.join('?').trim();
	return `${isLocal ? 'http' : 'https'}://${host}${search ? `?${search}` : ''}`;
}

export function isHttpUrl(value: unknown): value is string {
	if (typeof value !== 'string' || !value.trim()) return false;
	try {
		const url = new URL(value);
		return (url.protocol === 'http:' || url.protocol === 'https:') && !!url.hostname;
	} catch {
		return false;
	}
}
