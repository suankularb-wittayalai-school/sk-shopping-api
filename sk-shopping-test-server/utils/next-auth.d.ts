import NextAuth, { DefaultSession, DefaultJWT } from "next-auth";

declare module "next-auth" {
  /**
   * Returned by `useSession`, `getSession` and received as a prop on the `SessionProvider` React Context
   */
  interface Session {
    access_token: string;
    user: {
      /** The user's postal address. */
      access_token: string;
    } & DefaultSession["user"];
  }

  type JWT = {
    access_token: string;
  } & DefaultJWT;
}
