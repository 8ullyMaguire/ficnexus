# Contract: Forum Uploads

**Route Prefix**: `/api/forum/uploads`  
**Spec References**: FR-159..170, FR-047 (image resize)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level`

---

## 1. Upload File (Multipart)

### POST `/api/forum/uploads`

**Content-Type**: `multipart/form-data`

**Form Fields**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | file | **yes** | Image file (png, jpg, webp, gif) |
| `topic_id` | int64 | no | Associate with topic |
| `post_id` | int64 | no | Associate with post |
| `message_id` | int64 | no | Associate with message |
| `draft` | bool | no (default false) | Temporary upload for draft composer |

**Validation**:
- MIME: `image/png`, `image/jpeg`, `image/webp`, `image/gif`
- Max size: `FORUM_UPLOAD_MAX_MB` (default 10MB) → `max_upload_bytes` in config
- Max dimension: `FORUM_UPLOAD_MAX_DIMENSION` (default 1920px) — resized down
- Trust gate: **TL1+** (TL0 upload rejected 403)

**Processing** (server-side, async after upload):
1. Compute SHA256 → check `forum_uploads.sha256` for dedupe
2. Resize using `image` crate: max 1920px on longest side, preserve aspect
3. Convert to WebP (quality 85) → smaller, modern
4. Store at `/public/uploads/forum/{yyyy}/{mm}/{uuid}.webp`
5. Record in `forum_uploads` table

**Response 201**:
```json
{
  "err": 0,
  "upload": {
    "id": 1,
    "url": "/uploads/forum/2026/08/a1b2c3d4.webp",
    "original_name": "photo.png",
    "mime_type": "image/webp",
    "size_bytes": 245760,
    "width": 1920,
    "height": 1080,
    "sha256": "a1b2c3d4...",
    "created_at": "2026-08-31T12:00:00Z"
  }
}
```

**Errors**:
- `400`: Invalid file type / missing file
- `403`: Trust level insufficient (TL0)
- `413`: File too large (> max_upload_bytes)
- `429`: Rate limited (10 uploads/min per user)

---

## 2. Get Upload Info

### GET `/api/forum/uploads/{uploadId}`

**Response 200**: Same as upload response (no file stream).

---

## 3. Delete Upload

### DELETE `/api/forum/uploads/{uploadId}`

**Auth Rules**:
- Uploader: can delete own uploads
- Curator+: can delete any
- Only if not referenced by non-deleted post/topic/message

**Response 200**: `{"err": 0, "deleted": true}`

**Note**: Soft delete — file remains on disk (garbage collected weekly).

---

## 4. List My Uploads

### GET `/api/forum/uploads`

**Query Params**:
| Param | Type | Default |
|-------|------|---------|
| `limit` | int | 25 |
| `cursor` | int64 | - |

**Response 200**:
```json
{
  "err": 0,
  "uploads": [...],
  "next_cursor": ...,
  "has_more": false
}
```

---

## 5. Composer Integration (Client-Side)

**Flow**:
1. User drags/drops or pastes image in composer
2. Client uploads via `POST /api/forum/uploads` (with `draft=true` if no topic/post yet)
3. Server returns `{id, url, width, height}`
4. Client inserts Markdown: `![alt]({url})` or `<img src="{url}" width="{w}" height="{h}">`
5. On topic/post submit, `topic_id`/`post_id` associated via `forum_uploads` update

**Draft Uploads**: `draft=true` uploads cleaned up by cron (older than 24h, no topic/post/message ref).

---

## 6. Image Proxy / Serving

**Static Route**: `/uploads/forum/*` → served by Axum `ServeDir` (or nginx in prod).

**Headers**:
```
Cache-Control: public, max-age=31536000, immutable
Content-Type: image/webp
```

**No auth** — uploads are public once attached to public content. Private content (DMs) uses signed URLs (future).

---

## 7. Avatar Uploads (Separate)

**Endpoint**: `POST /api/user/avatar` (existing ficnexus route) — not part of forum uploads.

---

## 8. Realtime

**No WS events** for uploads — synchronous HTTP response sufficient. Composer shows progress via XHR upload events.