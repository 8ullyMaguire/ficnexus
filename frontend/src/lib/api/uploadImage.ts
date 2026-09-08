// Image upload helper for forum composers (Lane 4 — paste/drop wiring).
//
// Handles paste and drop events on <textarea> elements. Uploads the image
// via POST /api/uploads (multipart), then inserts `![](url)` at the cursor.

import { authHeaders } from './social';

export interface UploadResult {
  ok: boolean;
  url?: string;
  error?: string;
}

/** Upload a single image file to /api/uploads. Returns the public URL or an error. */
export async function uploadImage(file: File): Promise<UploadResult> {
  const form = new FormData();
  form.append('file', file);
  try {
    const res = await fetch('/api/uploads', {
      method: 'POST',
      headers: authHeaders(),
      body: form,
    });
    const json = await res.json();
    if (json.err === 0 && json.url) {
      return { ok: true, url: json.url };
    }
    return { ok: false, error: json.msg ?? 'Upload failed' };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

/** Insert text at the cursor position of a textarea (or append at end). */
function insertAtCursor(textarea: HTMLTextAreaElement, text: string) {
  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const value = textarea.value;
  textarea.value = value.slice(0, start) + text + value.slice(end);
  // Move cursor after the inserted text
  textarea.selectionStart = textarea.selectionEnd = start + text.length;
  textarea.dispatchEvent(new Event('input', { bubbles: true }));
}

/** Handle a paste or drop event containing an image. Returns true if handled. */
export async function handleImageEvent(
  e: ClipboardEvent | DragEvent,
  textarea: HTMLTextAreaElement,
  onStatus?: (msg: string) => void,
): Promise<boolean> {
  const items =
    e instanceof ClipboardEvent
      ? e.clipboardData?.items
      : e.dataTransfer?.items;
  if (!items) return false;

  let file: File | null = null;
  for (const item of items) {
    if (item.type.startsWith('image/')) {
      file = item.getAsFile();
      break;
    }
  }
  if (!file) return false;

  // Prevent default paste/drop behavior for images
  e.preventDefault();
  onStatus?.('Uploading image…');

  const result = await uploadImage(file);
  if (result.ok && result.url) {
    const md = `![](${result.url})`;
    insertAtCursor(textarea, md);
    onStatus?.('');
  } else {
    onStatus?.(result.error ?? 'Upload failed');
  }
  return true;
}
