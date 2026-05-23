# Arche API Specification

## Base URL

All endpoints are served under a single base. Authentication is via `X-API-Key` header.

```
POST /api/generate
GET  /api/blueprints
POST /api/blueprints
GET  /api/blueprints/:id
PUT  /api/blueprints/:id
DELETE /api/blueprints/:id
GET  /api/affixes
POST /api/affixes
GET  /api/affixes/:id
PUT  /api/affixes/:id
DELETE /api/affixes/:id
GET  /api/global-meta-attributes
POST /api/global-meta-attributes
GET  /api/global-meta-attributes/:id
PUT  /api/global-meta-attributes/:id
DELETE /api/global-meta-attributes/:id
GET  /api/schema/blueprints
GET  /api/schema/affixes
GET  /api/schema/generate
POST /api/clients             (super admin only)
GET  /api/clients             (super admin only)
GET  /api/clients/:id
DELETE /api/clients/:id       (super admin only)
POST /api/clients/:id/keys
GET  /api/clients/:id/keys
DELETE /api/clients/:id/keys/:key_id
GET  /api/audit-log           (super admin only, paginated)
POST /api/export              (super admin only)
POST /api/import              (super admin only)
POST /api/blueprints/batch/delete
POST /api/blueprints/batch/assign
POST /api/blueprints/batch/edit
POST /api/affixes/batch/delete
POST /api/affixes/batch/assign
```

---

## Generate

### `POST /api/generate`

**Request (minimal — totally random):**
```json
{}
```

**Request (with constraints):**
```json
{
  "archetype": "sword",
  "seed": 172839465,
  "constraints": {
    "damage": { "gte": 15, "lte": 40 },
    "rarity": { "in": ["rare", "legendary"] },
    "durability": { "gte": 100 },
    "magical_defense": true
  },
  "affixes": {
    "min_prefixes": 1,
    "max_prefixes": 2,
    "min_suffixes": 0,
    "max_suffixes": 1,
    "require": ["uuid-of-fire", "uuid-of-bear"],
    "block": ["uuid-of-ice"]
  }
}
```

**Response:**
```json
{
  "seed": 172839465,
  "name": "Fire Longsword of the Bear",
  "name_parts": {
    "base": "Longsword",
    "prefixes": ["Fire"],
    "suffixes": ["of the Bear"]
  },
  "blueprint_id": "uuid-of-blueprint",
  "blueprint_attributes": {
    "damage": 27.3,
    "durability": 150.0,
    "rarity": "legendary",
    "magical_defense": false
  },
  "affix_attributes": [
    {
      "affix_id": "uuid-of-fire",
      "affix_name": "Fire",
      "fire_damage": 12.7
    },
    {
      "affix_id": "uuid-of-the-bear",
      "affix_name": "of the Bear",
      "vitality": 8.0
    }
  ]
}
```

**Cache control:** Pass `X-Cache-Refresh: true` header to force a fresh DB load for this request (bypassing the cached generation data).

**Response notes:** `blueprint_attributes` are ordered by `attribute_order` on the blueprint. `affix_attributes` are ordered by `sort_order` on the blueprint_affixes join table.

**Error — no matching blueprints (RFC 9457):**
```json
{
  "type": "/errors/no-matching-blueprints",
  "title": "No blueprints match the given constraints",
  "status": 404,
  "detail": "No blueprint satisfies archetype 'sword' with damage >= 15 and rarity in [rare, legendary]."
}
```

---

## Blueprints

### `GET /api/blueprints`

Query params: `cursor`, `limit`, `page`, `per_page`, `archetype`, `search`

Response:
```json
{
  "data": [ /* blueprint objects */ ],
  "next_cursor": "uuid",
  "total": 42
}
```

### `POST /api/blueprints`

```json
{
  "name": "Longsword",
  "archetype": "sword",
  "weight": 1.0,
  "description": "A sturdy standard longsword",
  "attributes": {
    "damage": {
      "description": "Physical damage per hit",
      "value_type": "range",
      "min": 10.0,
      "max": 23.0,
      "distribution": { "type": "uniform" }
    },
    "rarity": {
      "$ref_id": "uuid-of-rarity"
    },
    "durability": {
      "value_type": "single",
      "value": 150.0
    },
    "magical_defense": {
      "value_type": "boolean",
      "value": false
    }
  },
  "attribute_order": ["damage", "rarity", "durability", "magical_defense"],
  "affixes": {
    "min_prefixes": 0,
    "max_prefixes": 1,
    "min_suffixes": 0,
    "max_suffixes": 2,
    "prefixes": [
      { "affix_id": "uuid-of-fire", "weight": 1.0 },
      { "affix_id": "uuid-of-frost", "weight": 0.5 }
    ],
    "suffixes": [
      { "affix_id": "uuid-of-the-bear", "weight": 1.0 }
    ]
  }
}
```

---

## Affixes

### `POST /api/affixes`

```json
{
  "name": "Fire",
  "type": "prefix",
  "description": "Enchants with fire damage",
  "attribute": {
    "name": "fire_damage",
    "description": "Additional fire damage per hit",
    "value_type": "range",
    "min": 5.0,
    "max": 15.0
  }
}
```

Or using a global meta attribute reference:
```json
{
  "name": "Fire",
  "type": "prefix",
  "attribute": {
    "$ref_id": "uuid-of-fire-damage"
  }
}
```

---

## Global Meta Attributes

### `POST /api/global-meta-attributes`

```json
{
  "name": "rarity",
  "description": "Quality tier of an item",
  "value_type": "enum",
  "values": ["common", "uncommon", "rare", "legendary"]
}
```

---

## Schema (JSON Schema)

### `GET /api/schema/blueprints`

Returns a JSON Schema document describing the blueprint resource structure (required fields, value types, attribute schema, etc.). Static — same for all clients.

### `GET /api/schema/affixes`

Same for affix resource structure.

### `GET /api/schema/generate`

Same for generate request/response structure.

---

## Client Management (super admin only)

### `POST /api/clients`
```json
{ "name": "My RPG Game" }
```
Returns:
```json
{
  "id": "uuid",
  "name": "My RPG Game",
  "created_at": "...",
  "api_keys": []  // initially empty
}
```

### `DELETE /api/clients/:id`
Super admin only. Returns 409 if client still has active keys. Danger zone.

---

## RBAC Keys

### `POST /api/clients/:id/keys`
```json
{
  "name": "game-server-key",
  "permissions": ["read", "generate"]
}
```
Returns the raw API key **once only**:
```json
{
  "id": "uuid",
  "name": "game-server-key",
  "permissions": ["read", "generate"],
  "key": "arche_k_xxxxxxxxxxxx..."  // shown once, not stored again
}
```

### `DELETE /api/clients/:id/keys/:key_id`
Revokes the key immediately.

---

## Audit Log

### `GET /api/audit-log`

Query params: `cursor`, `limit`, `client_id`, `resource_type`, `action`, `actor_key_id`

Response:
```json
{
  "data": [
    {
      "id": "uuid",
      "timestamp": "2026-05-22T10:30:00Z",
      "actor_key_id": "uuid",
      "actor_key_name": "cli-user-key",
      "client_id": "uuid",
      "resource_type": "blueprint",
      "resource_id": "uuid",
      "action": "updated",
      "before": { "weight": 1.0 },
      "after": { "weight": 2.0 }
    }
  ],
  "next_cursor": "uuid"
}
```

---

## Common Errors

| Status | Type | When |
|--------|------|------|
| 400 | `/errors/validation-error` | Malformed payload |
| 401 | `/errors/unauthorized` | Missing or invalid API key |
| 403 | `/errors/forbidden` | Key lacks required permission |
| 404 | `/errors/not-found` | Resource not found |
| 404 | `/errors/no-matching-blueprints` | No blueprints match generate constraints |
| 409 | `/errors/delete-referenced-resource` | DELETE blocked (resource referenced) |
| 422 | `/errors/unprocessable-entity` | Semantic validation failure |
| 409 | `/errors/import-conflict` | Import requires user resolution |

---

## Export / Import

### `POST /api/export`

**Request:**
```json
{
  "client_ids": ["uuid-1", "uuid-2"],
  "include_api_keys": false,
  "include_audit_log": false,
  "inline_global_refs": false
}
```

**Response:** Binary ZIP stream (`application/zip`). Folder-per-client structure.

### `POST /api/import`

**Request:** Binary ZIP upload (`multipart/form-data`).

**Response (no conflicts):**
```json
{
  "status": "imported",
  "clients_created": 1,
  "resources_imported": 42
}
```

**Response (conflicts detected — returns 409):**
```json
{
  "type": "/errors/import-conflict",
  "title": "Import conflicts require resolution",
  "status": 409,
  "conflicts": [
    {
      "resource_type": "blueprint",
      "resource_id": "uuid",
      "resource_name": "Longsword",
      "attributes": [
        {
          "key": "damage",
          "old_value": { "min": 10, "max": 23 },
          "new_value": { "min": 15, "max": 30 },
          "value_type": "range"
        }
      ]
    }
  ]
}
```

### `POST /api/import/resolve`

Resubmit the import with resolution choices.

**Request:**
```json
{
  "import_token": "token-from-409-response",
  "resolutions": {
    "uuid-of-blueprint": {
      "strategy": "per_attribute",
      "attributes": {
        "damage": "keep_new",
        "weight": "keep_old"
      }
    },
    "uuid-of-affix": {
      "strategy": "keep_new"
    }
  }
}
```

---

## Batch Operations

### `POST /api/blueprints/batch/delete`
```json
{ "ids": ["uuid-1", "uuid-2", "uuid-3"] }
```
Returns count of deleted blueprints.

### `POST /api/blueprints/batch/assign`
```json
{
  "blueprint_ids": ["uuid-1", "uuid-2"],
  "affix_ids": ["uuid-3", "uuid-4"],
  "weight": 1.0
}
```
Adds each affix to each blueprint's pool with the given weight. Duplicates silently skipped.

### `POST /api/blueprints/batch/edit`
```json
{
  "blueprint_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "attributes": {
    "damage": { "min": 5, "max": 20 }
  }
}
```
Only blueprints that already have the attribute (by key name) are updated. Blueprints without the attribute are skipped. Type mismatch returns 422.

### `POST /api/affixes/batch/delete`
```json
{ "ids": ["uuid-1", "uuid-2"] }
```

### `POST /api/affixes/batch/assign`
```json
{
  "affix_ids": ["uuid-1", "uuid-2"],
  "blueprint_ids": ["uuid-3", "uuid-4"],
  "weight": 1.0
}
```
