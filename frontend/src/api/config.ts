import { get } from "./client";

// Runtime config the SPA reads at boot (currently just the LINE LIFF id, held
// server-side in an env var so it can change without rebuilding the bundle).
export interface PublicConfig {
  liff_id: string;
}

export function getPublicConfig(): Promise<PublicConfig> {
  return get<PublicConfig>("/api/public/config");
}
