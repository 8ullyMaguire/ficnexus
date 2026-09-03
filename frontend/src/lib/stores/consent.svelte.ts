/**
 * Cookie-consent state (Tier-2 prerequisite).
 *
 * Two-state consent gate that the anonymous-library device cookie (and any
 * future non-essential cookie) must respect:
 *  - `granted` → `fh_consent=granted` cookie, 1 year, SameSite=Lax
 *  - `rejected` → `fh_consent=rejected` sessionStorage flag (per-tab, nothing
 *    persistent is written)
 * `null` → not yet asked, show the toast.
 *
 * Not localStorage: consent is a site-level decision, so a cookie (readable
 * server-side too, which the device-cookie feature needs) is the right store.
 */

const CONSENT_COOKIE = 'fh_consent';
const ONE_YEAR_S = 60 * 60 * 24 * 365;

function readConsentCookie(): 'granted' | 'rejected' | null {
  if (typeof document === 'undefined') return null;
  const m = document.cookie.split('; ').find((c) => c.startsWith(`${CONSENT_COOKIE}=`));
  if (m?.endsWith('=granted')) return 'granted';
  if (m?.endsWith('=rejected')) return 'rejected';
  return null;
}

let consent = $state<'granted' | 'rejected' | null>(
  typeof window === 'undefined' ? null : readConsentCookie(),
);

/** True when a non-essential cookie may be set. */
export function consentGranted(): boolean {
  return consent === 'granted';
}

export function getConsent(): 'granted' | 'rejected' | null {
  return consent;
}

export function grantConsent(): void {
  consent = 'granted';
  document.cookie = `${CONSENT_COOKIE}=granted; max-age=${ONE_YEAR_S}; path=/; SameSite=Lax`;
}

export function rejectConsent(): void {
  consent = 'rejected';
  try {
    sessionStorage.setItem(`${CONSENT_COOKIE}Rejected`, '1');
  } catch {
    /* private mode etc. — consent state just resets next navigation */
  }
}

export function consentToastVisible(): boolean {
  return consent === null;
}
