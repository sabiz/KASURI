/**
 * Configuration for SvelteKit.
 * Tauri doesn't have a Node.js server to do proper SSR
 * so we will use adapter-static to prerender the app (SSG)
 * See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
 */

/** Enable prerendering for the application */
export const prerender = true;

/** Disable server-side rendering */
export const ssr = false;
