// Imports
import { APIError } from "@/utils/types";
import qs from "qs";
import { sift } from "radash";

/**
 * API fetching boilerplate. Note that this contacts the Club Registry API, not
 * the Supabase API.
 *
 * @param path The path (without queries) to fetch from.
 * @param query An object to parse into the query in the fetch URL.
 * @param options Initiation options for fetch.
 * @param sessionToken The Supabase session token of the user.
 *
 * @returns A JSON representation of a Fetch Response.
 */
export async function fetchAPI<Data extends {} = {}>(
  path: string,
  query?: {},
  options?: RequestInit,
  sessionToken?: string
): Promise<{
  api_version: string;
  data: Data;
  error: APIError | null;
  meta: null;
}> {
  // console.log("fetchAPI", {
  //   url: sift([
  //     process.env.NEXT_PUBLIC_API_URL,
  //     path,
  //     query && "?" + qs.stringify(query, { encode: false }),
  //   ]).join(""),
  //   option: options
  //     ? {
  //         ...options,
  //         ...(sessionToken
  //           ? {
  //               headers: {
  //                 ...options.headers,
  //                 Authorization: `Bearer ${sessionToken}`,
  //               },
  //             }
  //           : {}),
  //       }
  //     : undefined,
  // });
  let res = await fetch(
    sift([
      process.env.NEXT_PUBLIC_API_URL,
      // "http://127.0.0.1:8000",
      path,
      query && "?" + qs.stringify(query, { encode: false }),
    ]).join(""),
    options
      ? {
          ...options,
          ...(sessionToken
            ? {
                headers: {
                  ...options.headers,
                  Authorization: `Bearer ${sessionToken}`,
                },
              }
            : {}),
        }
      : undefined
  ).catch((err) => {
    console.error(err);
    throw new Error(err);
  });
  if (!res.ok) {
    throw new Error(await res.text());
  }

  return await res.json();
}
