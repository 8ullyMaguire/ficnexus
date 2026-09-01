# Contract: Forum Groups & Privileges

**Route Prefix**: `/api/forum/groups`, `/api/forum/privileges`  
**Spec References**: FR-048..058 (groups), FR-031, FR-059..062 (privileges)  
**Auth**: `AuthUser` — `user_id`, `role`, `trust_level`

---

## 1. Groups

### 1.1 List Groups

#### GET `/api/forum/groups`

**Query Params**:
| Param | Type | Default | Description |
|-------|------|---------|-------------|
| `search` | string | - | Filter by name/description |
| `type` | string | - | `public` | `private` | `system` |
| `limit` | int | 25 | Max 100 |
| `cursor` | int64 | - | Pagination cursor (created_at) |

**Response 200**:
```json
{
  "err": 0,
  "groups": [
    {
      "id": 1,
      "name": "Moderators",
      "slug": "moderators",
      "description": "Forum moderation team",
      "cover_url": "/uploads/forum/2026/08/cover_1.webp",
      "is_private": false,
      "is_system": true,
      "owner": {"id": 1, "username": "admin"},
      "member_count": 5,
      "user_role": "member",  // null if not member
      "created_at": "2026-01-15T10:00:00Z"
    }
  ],
  "next_cursor": 1725105600000,
  "has_more": false
}
```

**Auth Matrix**:
| Role / Trust | Anonymous | TL0 | TL1+ | Curator | Admin |
|--------------|-----------|-----|------|---------|-------|
| **List public** | ✓ | ✓ | ✓ | ✓ | ✓ |
| **List private** | ✗ | ✗ | member only | ✓ | ✓ |
| **See system** | ✗ | ✗ | ✗ | ✓ | ✓ |

---

### 1.2 Get Group Detail

#### GET `/api/forum/groups/{groupId}`

**Response 200**: Includes `members` array (paginated), `privileges` summary.

---

### 1.3 Create Group

#### POST `/api/forum/groups`

**Request**:
```json
{
  "name": "Harry Potter Fans",
  "slug": "hp-fans",
  "description": "For HP enthusiasts",
  "is_private": false
}
```

**Validation**:
- `name`: 1-100 chars, unique
- `slug`: lowercase alphanumeric + hyphens, unique, max 60 chars
- `is_private`: if true, join requires invite/approval

**Trust Gate**: **TL2+** (`PUBLISH_MIN_TRUST=2`)

**Response 201**: Group object with `owner_id` = current user.

**Auth Matrix**:
| Role / Trust | Anonymous | TL0-1 | TL2+ | Curator | Admin |
|--------------|-----------|-------|------|---------|-------|
| **Create** | 401 | 403 | ✓ | ✓ | ✓ |

---

### 1.4 Update Group

#### PATCH `/api/forum/groups/{groupId}`

**Request** (all optional):
```json
{
  "name": "New Name",
  "description": "Updated",
  "cover_url": "/uploads/...",
  "is_private": true
}
```

**Auth Rules**:
- Owner: can update
- Manager (role=manager in group_members): can update
- Admin: can update any

---

### 1.5 Delete Group

#### DELETE `/api/forum/groups/{groupId}`

**Auth Rules**: Owner or Admin. System groups (`is_system=true`) cannot be deleted.

---

### 1.6 Membership

#### POST `/api/forum/groups/{groupId}/join` — Join public group / request invite for private
#### POST `/api/forum/groups/{groupId}/leave` — Leave group
#### POST `/api/forum/groups/{groupId}/invite` — Invite user (owner/manager)
```json
{"user_id": 42}
```
#### POST `/api/forum/groups/{groupId}/members/{userId}/role` — Change role
```json
{"role": "manager"}  // owner | manager | member
```
#### DELETE `/api/forum/groups/{groupId}/members/{userId}` — Remove member (owner/manager/admin)

**Response 200**: `{"err": 0, "success": true}`

---

## 2. Privileges (Per-Category Per-Group)

### 2.1 List Privileges for Category

#### GET `/api/forum/categories/{categoryId}/privileges`

**Response 200**:
```json
{
  "err": 0,
  "privileges": [
    {
      "id": 1,
      "category_id": 1,
      "group": {"id": 2, "name": "Moderators", "slug": "moderators"},
      "privilege": "moderate",
      "granted_by": {"id": 1, "username": "admin"},
      "created_at": "2026-08-01T10:00:00Z"
    },
    {
      "id": 2,
      "category_id": 1,
      "group": {"id": 3, "name": "Members", "slug": "members"},
      "privilege": "write",
      "granted_by": {"id": 1, "username": "admin"},
      "created_at": "2026-08-01T10:00:00Z"
    }
  ]
}
```

**Privilege Types**:
| Privilege | Description |
|-----------|-------------|
| `read` | View category + topics |
| `write` | Create topics |
| `reply` | Reply to topics |
| `moderate` | Lock/pin/move/delete topics, hide posts |
| `flag` | Report posts (TL2+ required regardless) |
| `manage_membership` | Invite/remove group members |

---

### 2.2 Grant Privilege

#### POST `/api/forum/categories/{categoryId}/privileges`

**Request**:
```json
{
  "group_id": 2,
  "privilege": "moderate"
}
```

**Auth**: Admin only (role ≥ 10).

**Response 201**: Privilege object.

---

### 2.3 Revoke Privilege

#### DELETE `/api/forum/categories/{categoryId}/privileges/{privilegeId}`

**Auth**: Admin only.

---

### 2.4 Effective Privileges Check (Internal)

**Used by**: All write routes to gate access.

**Logic**:
```rust
async fn user_has_privilege(db, user_id, category_id, privilege) -> bool {
    // 1. Admin (role >= 10) always true
    // 2. Check user's groups -> forum_privileges where privilege matches
    // 3. System groups (e.g. "Moderators") auto-grant based on trust/role
}
```

**Trust Gates Override**:
| Action | Minimum Trust | Override |
|--------|---------------|----------|
| Create topic | TL1 | Privilege `write` + TL1 |
| Reply | TL1 | Privilege `reply` + TL1 |
| Flag | TL2 | Privilege `flag` + TL2 |
| Moderate | TL5 | Privilege `moderate` + TL5 |

---

## 3. Default Groups (System)

Created at migration time (`is_system=true`):
| Group | Slug | Purpose |
|-------|------|---------|
| Administrators | `administrators` | Full access (role ≥ 10) |
| Moderators | `moderators` | Moderate privilege (role ≥ 5 or TL5+) |
| Members | `members` | Base write/reply (TL1+) |
| Trust Level 3+ | `tl3-plus` | Tag creation, etc. |

**Auto-membership**: Users added to `tl3-plus` when `trust_level >= 3` (via trust promotion cron).