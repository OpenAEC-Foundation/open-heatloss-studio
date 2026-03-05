/**
 * OIDC authentication via oidc-spa.
 *
 * Provides a pre-configured oidc-spa instance with Authentik as identity provider.
 * No auto-login: users can use the app without logging in (demo mode).
 */
import { z } from "zod";
import { oidcSpa } from "oidc-spa/react-spa";

/** Decoded ID token schema — validates Authentik claims. */
const decodedIdTokenSchema = z.object({
  sub: z.string(),
  email: z.string().optional(),
  name: z.string().optional(),
  preferred_username: z.string().optional(),
});

/** Decoded ID token type. */
export type DecodedIdToken = z.infer<typeof decodedIdTokenSchema>;

/** Pre-configured oidc-spa utils (no auto-login). */
export const {
  OidcInitializationGate,
  useOidc,
  getOidc,
  bootstrapOidc,
  withLoginEnforced,
} = oidcSpa
  .withExpectedDecodedIdTokenShape({
    decodedIdTokenSchema,
  })
  .createUtils();
