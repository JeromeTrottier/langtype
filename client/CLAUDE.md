# client

React + Vite + TypeScript, managed with **pnpm** (`pnpm --dir client …`).

- UI text only in `src/i18n/locales/en.json`.
- API types come from `src/api/types.gen.ts` (generated; don't edit). Call the API through `src/api/client.ts`.
- Features in `src/features/<name>/`; tests next to components (`*.test.tsx`).
