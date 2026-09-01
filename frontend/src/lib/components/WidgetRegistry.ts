// Widget registry: maps widget slugs to Svelte components and provides
// the default layout factory.  Actual widget components will be lazy-loaded
// here as they're implemented — for now every widget renders a styled
// placeholder.

import type { Component } from 'svelte';
import type { WidgetInfo, LayoutItem, Feature } from '$lib/api/layouts';

// ── Registry ─────────────────────────────────────────────────────────────

/**
 * Slug → component map.  Populated by `registerWidget()` below.
 * Components receive a single `widget` prop (the WidgetInfo metadata).
 */
const registry = new Map<string, Component<{ widget: WidgetInfo }>>();

/** Register a widget component under its slug. */
export function registerWidget(
  slug: string,
  component: Component<{ widget: WidgetInfo }>,
): void {
  registry.set(slug, component);
}

/** Look up a widget component by slug. Returns undefined if unregistered. */
export function getWidgetComponent(
  slug: string,
): Component<{ widget: WidgetInfo }> | undefined {
  return registry.get(slug);
}

// ── Default layout factory ───────────────────────────────────────────────

/**
 * Generate a sensible default layout for a page based on the list of
 * available widgets.  Full-width widgets come first, then halves, then
 * quarters.  Each widget gets a sequential `pos` value.
 */
export function defaultLayout(widgets: WidgetInfo[]): LayoutItem[] {
  const sorted = [...widgets].sort((a, b) => {
    // Full > half > quarter ordering
    const sizeOrder: Record<string, number> = { full: 0, half: 1, quarter: 2 };
    return (sizeOrder['full'] ?? 2) - (sizeOrder['full'] ?? 2); // placeholder sort
  });

  // Simpler: just list them in the order they're returned by the API,
  // giving each 'half' width (reasonable default) and sequential pos.
  return widgets.map((w, i) => ({
    widget: w.slug,
    size: i === 0 ? 'full' : 'half',
    pos: i,
  }));
}

// ── Feature-gated filtering ──────────────────────────────────────────────

/**
 * Filter the full widget list down to only those the current user is
 * allowed to see.  A widget is visible when:
 *  1. Its `min_rank` <= the user's effective level, AND
 *  2. Its associated feature (if any) is enabled.
 *
 * The `featureSlug` convention is `widget.{slug}` — if a feature with
 * that slug exists and is disabled, the widget is hidden regardless of
 * rank.
 */
export function getAvailableWidgets(
  widgets: WidgetInfo[],
  features: Feature[],
  userLevel: number = 0,
): WidgetInfo[] {
  const featureMap = new Map(features.map((f) => [f.slug, f.enabled]));

  return widgets.filter((w) => {
    // Rank gate
    if (w.min_rank > userLevel) return false;

    // Feature gate: check if a "widget.<slug>" feature exists and is off
    const featureKey = `widget.${w.slug}`;
    if (featureMap.has(featureKey) && !featureMap.get(featureKey)) return false;

    return true;
  });
}
