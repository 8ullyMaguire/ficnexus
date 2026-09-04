# Contract: Forum Categories

**Route Prefix**: `/api/forum/categories`  
**Spec References**: FR-001..008, FR-030 (mod-only), FR-031 (privileges)  
**Auth**: `AuthUser` extractor — `user_id: Option<i32>`, `role: i16` (0/5/10), `trust_level: i16` (0-6)

---

## 1. List Categories

### GET `/api/forum/categories`

**Query Params**: None

**Response 200**:
```json
{
  "err": 0,
  "categories": [
    {
      "id": 1,
      "slug": "general",
      "title": "General Discussion",
      "description": "Anything goes",
      "position": 0,
      "is_mod_only": false,
      "topic_count": 42,
      "unread_count": 3,
      "last_topic": {
        "id": 105,
        "title": "Welcome!",
        "slug": "welcome-105",
        "last_post_at": "2026-08-31T12:00:00Z",
        "author": {"id": 7, "username": "alice", "trust_level": 3}
      }
    }
  ]
}
```

**Auth Matrix**:
| Role / Trust | Anonymous | TL0 | TL1+ | Curator (role≥5) | Admin (role≥10) |
|--------------|-----------|-----|------|------------------|-----------------|
| **Read** | ✓ | ✓ | ✓ | ✓ | ✓ |
| **Mod-only visible** | ✗ | ✗ | ✗ | ✓ | ✓ |

**Notes**: `unread_count` and `last_topic` only for authenticated users (joins `forum_read_state`). Mod-only categories (`is_mod_only=true`) hidden unless `role >= 5` or `trust_level >= 5`.

---

## 2. Create Category (Admin)

### POST `/api/forum/categories`

**Request**:
```json
{
  "title": "Announcements",
  "slug": "announcements",
  "description": "Official announcements",
  "position": -1,
  "is_mod_only": true
}
```

**Validation**:
- `slug`: lowercase alphanumeric + hyphens, unique, max 60 chars
- `title`: non-empty, max 120 chars
- `description`: max 1000 chars
- `position`: -1 = append to end

**Response 201**:
```json
{
  "err": 0,
  "category": {
    "id": 5,
    "slug": "announcements",
    "title": "Announcements",
    "description": "Official announcements",
    "position": 4,
    "is_mod_only": true,
    "created_at": "2026-08-31T12:00:00Z"
  }
}
```

**Auth Matrix**:
| Role / Trust | Anonymous | TL0-4 | TL5-6 | Curator | Admin |
|--------------|-----------|-------|-------|---------|-------|
| **Create** | 401 | 403 | 403 | 403 | ✓ |

**Errors**:
- `400` (`err:400`): slug invalid / title empty / description too long
- `401` (`err:401`): not authenticated
- `403` (`err:403`): requires admin (role ≥ 10)
- `409` (`err:409`): slug already exists

---

## 3. Update Category (Admin)

### PATCH `/api/forum/categories/{id}`

**Request** (all optional):
```json
{
  "title": "New Title",
  "description": "Updated description",
  "position": 2,
  "is_mod_only": false
}
```

**Response 200**: Same shape as create response.

**Auth Matrix**: Same as Create (Admin only).

**Errors**: Same as Create + `404` (`err:404`) if category not found.

---

## 4. Delete Category (Admin)

### DELETE `/api/forum/categories/{id}`

**Response 200**:
```json
{"err": 0, "deleted": true}
```

**Auth Matrix**: Admin only.

**Cascade**: Deletes topics → posts → votes → follows → read_state → uploads. Soft-delete (`deleted_at`) preferred; hard delete only for empty categories.

---

## 5. Reorder Categories (Admin)

### POST `/api/forum/categories/reorder`

**Request**:
```json
{
  "order": [3, 1, 4, 2]  // category ids in new position order
}
```

**Response 200**:
```json
{"err": 0, "reordered": true}
```

**Auth Matrix**: Admin only.

---

## 6. Category Privileges (see contracts/forum-groups-privileges.md)

- `GET /api/forum/categories/{id}/privileges` — list privileges
- `POST /api/forum/categories/{id}/privileges` — grant privilege to group
- `DELETE /api/forum/categories/{id}/privileges/{privilege_id}` — revoke

**Auth**: Admin only for write; Curator+ for read.