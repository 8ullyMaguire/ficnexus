// Recipe Builder API client for /api/recipes
// Backend: src/routes/recipes.rs + src/services/recipes.rs

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...options,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...(options?.headers ?? {}),
    },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as T;
}

// ── Types ────────────────────────────────────────────────────────────────────

export interface Recipe {
  id: number;
  user_id: number;
  name: string;
  description: string | null;
  blend: Record<string, number>;
  filters: Record<string, unknown>;
  boost: Record<string, unknown>;
  curator_prior: number;
  is_active: boolean;
  is_public: boolean;
  installs: number;
  created_at: string;
  updated_at: string;
}

export interface RecipeResponse {
  err: number;
  recipe?: Recipe;
  recipes?: Recipe[];
  message?: string;
}

// ── API functions ────────────────────────────────────────────────────────────

/** GET /api/recipes — list user's recipes */
export async function listRecipes(): Promise<RecipeResponse> {
  return request<RecipeResponse>('/recipes');
}

/** POST /api/recipes — create a new recipe */
export async function createRecipe(payload: {
  name: string;
  description?: string;
  blend?: Record<string, number>;
  filters?: Record<string, unknown>;
  boost?: Record<string, unknown>;
  curator_prior?: number;
}): Promise<RecipeResponse> {
  return request<RecipeResponse>('/recipes', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

/** PUT /api/recipes/:id — update a recipe */
export async function updateRecipe(
  id: number,
  payload: {
    name?: string;
    description?: string | null;
    blend?: Record<string, number>;
    filters?: Record<string, unknown>;
    boost?: Record<string, unknown>;
    curator_prior?: number;
  },
): Promise<RecipeResponse> {
  return request<RecipeResponse>(`/recipes/${id}`, {
    method: 'PUT',
    body: JSON.stringify(payload),
  });
}

/** DELETE /api/recipes/:id — delete a recipe */
export async function deleteRecipe(id: number): Promise<RecipeResponse> {
  return request<RecipeResponse>(`/recipes/${id}`, {
    method: 'DELETE',
  });
}

/** POST /api/recipes/:id/activate — set as active recipe */
export async function activateRecipe(id: number): Promise<RecipeResponse> {
  return request<RecipeResponse>(`/recipes/${id}/activate`, {
    method: 'POST',
  });
}

/** GET /api/recipes/active — get user's active recipe */
export async function getActiveRecipe(): Promise<RecipeResponse> {
  return request<RecipeResponse>('/recipes/active');
}

/** GET /api/recipes/gallery — browse public recipes */
export async function browseGallery(
  limit = 20,
  offset = 0,
): Promise<RecipeResponse> {
  return request<RecipeResponse>(
    `/recipes/gallery?limit=${limit}&offset=${offset}`,
  );
}

/** POST /api/recipes/:id/install — install from gallery */
export async function installRecipe(id: number): Promise<RecipeResponse> {
  return request<RecipeResponse>(`/recipes/${id}/install`, {
    method: 'POST',
  });
}

/** POST /api/recipes/:id/publish — make public / unpublish */
export async function publishRecipe(
  id: number,
  isPublic: boolean,
): Promise<RecipeResponse> {
  return request<RecipeResponse>(`/recipes/${id}/publish`, {
    method: 'POST',
    body: JSON.stringify({ is_public: isPublic }),
  });
}
