import type { components } from "./types.gen";

export type Health = components["schemas"]["Health"];

/** Base path of the API; the Vite dev server proxies it to the local server. */
const BASE = "/api/v1";

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, init);
  if (!res.ok) throw new ApiError(res.status, await res.text());
  return (await res.json()) as T;
}

export const api = {
  health: () => request<Health>("/health"),
};
