export const publicApiUrl =
  typeof document !== "undefined" ? document.body.dataset.publicApiUrl ?? "" : "";

export function apiPath(path: string): string {
  return `${publicApiUrl}${path}`;
}
