/**
 * Anonymous library store (Tier-2).
 *
 * Bookmarks and follows for visitors not logged in. The server stores the
 * library in a signed `fh_dev` cookie via `Set-Cookie` on every mutating
 * endpoint; this store only mirrors the response shape for reactive UI
 * (e.g. "you've bookmarked this" indicators, library count badges).
 *
 * Consent gate: nothing is written until the cookie-consent toast is
 * accepted — see `consent.svelte.ts`.
 *
 * On login/register, the cookie set by these endpoints travels with the
 * browser request to `/api/v1/device/merge`, which folds it into the user
 * account.
 */

import { consentGranted } from './consent.svelte';

const COOKIE_NAME = 'fh_dev';

interface DeviceFollow {
    target_type: string;
    target_id?: number;
    author_name?: string;
}

interface DeviceLibrary {
    device_id: string;
    bookmarks: number[];
    follows: DeviceFollow[];
    updated_at?: string | null;
}

function readDeviceCookieValue(): string {
    if (typeof document === 'undefined') return '';
    const m = document.cookie.split('; ').find((c) => c.startsWith(`${COOKIE_NAME}=`));
    return m?.slice(`${COOKIE_NAME}=`.length) ?? '';
}

// ── Reactive state mirror ───────────────────────────────────────────────────
const _state = $state({
    bookmarks: [] as number[],
    follows: [] as DeviceFollow[],
});

function applyLibrary(lib: DeviceLibrary): void {
    _state.bookmarks = [...lib.bookmarks];
    _state.follows = [...lib.follows];
}

async function refresh(): Promise<void> {
    if (typeof window === 'undefined') return;
    try {
        const res = await fetch('/api/v1/device/library', {
            credentials: 'include',
        });
        if (!res.ok) return;
        const data = (await res.json()) as { err: number; library: DeviceLibrary };
        if (data.err === 0 && data.library) applyLibrary(data.library);
    } catch {
        /* offline — keep whatever's in memory */
    }
}

// Eagerly hydrate from the server on first load so reactive UI sees the
// existing bookmarks/follows even after a hard refresh.
if (typeof window !== 'undefined') {
    void refresh();
}

// ── Public API ──────────────────────────────────────────────────────────────

export function isDeviceBookmarked(workId: number): boolean {
    return _state.bookmarks.includes(workId);
}

export function isDeviceFollowing(
    targetType: string,
    targetId?: number,
    authorName?: string,
): boolean {
    return _state.follows.some(
        (f) =>
            f.target_type === targetType &&
            f.target_id === targetId &&
            f.author_name === authorName,
    );
}

export async function toggleDeviceBookmark(workId: number): Promise<boolean> {
    if (!consentGranted()) return false;
    const currentlyBookmarked = isDeviceBookmarked(workId);
    try {
        const res = await fetch(
            currentlyBookmarked
                ? `/api/v1/device/bookmark/${workId}`
                : '/api/v1/device/bookmark',
            {
                method: currentlyBookmarked ? 'DELETE' : 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: currentlyBookmarked ? undefined : JSON.stringify({ work_id: workId }),
            },
        );
        if (!res.ok) return false;
        const data = (await res.json()) as { err: number; library: DeviceLibrary };
        if (data.err === 0 && data.library) applyLibrary(data.library);
        return true;
    } catch {
        return false;
    }
}

export async function toggleDeviceFollow(
    targetType: string,
    targetId?: number,
    authorName?: string,
): Promise<boolean> {
    if (!consentGranted()) return false;
    const currentlyFollowing = isDeviceFollowing(targetType, targetId, authorName);
    try {
        const res = await fetch('/api/v1/device/follow', {
            method: currentlyFollowing ? 'DELETE' : 'POST',
            credentials: 'include',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                target_type: targetType,
                target_id: targetId,
                author_name: authorName,
            }),
        });
        if (!res.ok) return false;
        const data = (await res.json()) as { err: number; library: DeviceLibrary };
        if (data.err === 0 && data.library) applyLibrary(data.library);
        return true;
    } catch {
        return false;
    }
}

// ── Merge on login ──────────────────────────────────────────────────────────

/**
 * Called after login/register. The signed `fh_dev` cookie travels with the
 * request to `/api/v1/device/merge` (cookies are auto-included because we
 * used `credentials: 'include'` for every mutating call). Backend reads it
 * from the `Cookie` header and folds it into the user account.
 *
 * After successful merge we drop the local mirror — server-side state is
 * the truth going forward.
 */
export async function mergeOnLogin(): Promise<{ merged_bookmarks: number; merged_follows: number }> {
    if (typeof window === 'undefined') return { merged_bookmarks: 0, merged_follows: 0 };
    const cookieValue = readDeviceCookieValue();
    if (!cookieValue) {
        _state.bookmarks = [];
        _state.follows = [];
        return { merged_bookmarks: 0, merged_follows: 0 };
    }
    try {
        const res = await fetch('/api/v1/device/merge', {
            method: 'POST',
            credentials: 'include',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({}), // backend reads from Cookie header
        });
        if (res.ok) {
            const data = (await res.json()) as {
                merged_bookmarks: number;
                merged_follows: number;
            };
            // Clear the cookie so subsequent logins don't re-merge.
            document.cookie = `${COOKIE_NAME}=; Max-Age=0; Path=/; SameSite=Lax`;
            _state.bookmarks = [];
            _state.follows = [];
            return data;
        }
    } catch {
        /* offline — keep the cookie; next merge attempt will pick it up */
    }
    return { merged_bookmarks: 0, merged_follows: 0 };
}
