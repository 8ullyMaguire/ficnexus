// API client for the Send-to-Kindle feature.
// POST /api/send-to-kindle — generates an EPUB for a fic and emails it to
// the logged-in user's Kindle address (requires an auth token).
//
// Auth convention (see social.ts): the JWT lives in localStorage under
// 'fichub_token' and is sent as `Authorization: Bearer <token>`.

const BASE = '/api';
const TOKEN_KEY = 'fichub_token';

interface SendToKindleResponse {
  err: number;
  url_id?: string;
  to?: string;
  msg?: string;
}

function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}

/**
 * Send a fic to the user's Kindle address.
 * Accepts either a fic `url` (scrapes fresh) or a known `url_id`.
 * Requires the user to be logged in (JWT in localStorage).
 */
export async function sendToKindle(body: {
  url?: string;
  url_id?: string;
}): Promise<SendToKindleResponse> {
  const token = getToken();
  if (!token) {
    throw new Error('You must be logged in to send to Kindle.');
  }
  const res = await fetch(`${BASE}/send-to-kindle`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return (await res.json()) as SendToKindleResponse;
}
