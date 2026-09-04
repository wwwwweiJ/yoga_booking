import { post } from "./client";

// Same shape the email/password login returns (see Login.tsx).
interface LoginResponse {
  token: string;
  pid: string;
  name: string;
  is_verified: boolean;
}

// Exchange a LIFF-issued LINE id token (plus the studio's public token) for our
// own JWT. The backend verifies the id token with LINE, then finds-or-creates
// the user in that studio.
export function lineLogin(body: {
  id_token: string;
  organization_token: string;
}): Promise<LoginResponse> {
  return post<LoginResponse>("/api/auth/line", body);
}
