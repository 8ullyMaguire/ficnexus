// qa/fingerprint.js — deterministic, normalized fingerprint helpers.
// Normalize volatile values (IDs, timestamps, ports, PIDs, SQL literals)
// BEFORE hashing so convergence is stable across runs.
export function normalizeRoute(r) {
  return r
    .replace(/\/\d+(?=\/|$)/g, '/:id')          // /books/123 -> /books/:id
    .replace(/[0-9a-f]{8}-[0-9a-f-]{27,}/gi, '/:uuid')
    .replace(/\/[0-9a-f]{6,}(?=\/|$)/gi, '/:hash')
    .replace(/page=\d+/g, 'page=N')
    .replace(/[?&]q=[^&]*/g, '?q=Q')
    .replace(/\/\d{4}-\d{2}-\d{2}/g, '/:date');
}

export function normalizeMessage(m) {
  return (m || '')
    .replace(/0x[0-9a-fA-F]+/g, ':addr')        // stack addresses
    .replace(/\/home\/[^/\s]+/g, '$HOME')        // home paths
    .replace(/\/personal\/[^/\s]+/g, '$REPO')    // repo root
    .replace(/pid[= ]\d+/gi, 'pid :pid')
    .replace(/\b\d{4}-\d{2}-\d{2}T[\d:.Z-]+\b/g, ':ts')
    .replace(/\b\d{13,}\b/g, ':ts')
    .replace(/'[^']*'/g, "'?'")                  // SQL literals
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 160);
}

// layer + method/route-template + error class + normalized message + source location
export function makeFingerprint({ layer, route, method, errorClass, message, loc }) {
  return [
    layer || 'app',
    method || 'GET',
    normalizeRoute(route || ''),
    errorClass || '?',
    normalizeMessage(message || ''),
    loc || '',
  ].join('|');
}
