/** Route matcher: only matches all-digit path segments.
 *  Lets /authors/[id=numeric] (curator profile pages) coexist with
 *  /authors/[name] (author bibliography) at the same route level. */
export function match(value: string): boolean {
  return /^\d+$/.test(value);
}
