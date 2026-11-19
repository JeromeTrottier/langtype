#!/usr/bin/env node
// Validation screenshots for pull requests (docs/DEVELOPMENT.md).
//
//   node client/scripts/screenshot.mjs --out .devloop-pr/screenshots \
//     --page / --name spaces --page /settings --name settings \
//     --run "cargo test -p mnemo-core" --name core-tests
//
// --page <path>  Starts the server and dashboard on free ports (fresh data folder), opens the
//                path in headless Chromium and saves <name>.png. Add --dark for a dark variant.
// --run <cmd>    Runs a command and saves its output, rendered like a terminal, as <name>.png.
//                Exits non-zero if the command fails (the screenshot is still saved).
import { chromium } from "playwright";
import { spawn, spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

function parseArgs(argv) {
  const shots = [];
  let out = ".devloop-pr/screenshots";
  let dark = false;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--out") out = argv[++i];
    else if (a === "--dark") dark = true;
    else if (a === "--page") shots.push({ kind: "page", target: argv[++i] });
    else if (a === "--run") shots.push({ kind: "run", target: argv[++i] });
    else if (a === "--name") {
      if (!shots.length) throw new Error("--name must follow --page or --run");
      shots[shots.length - 1].name = argv[++i];
    } else throw new Error(`unknown argument ${a}`);
  }
  if (!shots.length) throw new Error("nothing to capture: pass --page or --run");
  shots.forEach((s, i) => {
    s.name ??=
      s.kind === "page"
        ? s.target.replace(/[^a-z0-9]+/gi, "-").replace(/^-|-$/g, "") || "home"
        : `run-${i + 1}`;
  });
  return { out: resolve(process.cwd(), out), shots, dark };
}

const freePort = () =>
  new Promise((ok, fail) => {
    const srv = createServer();
    srv.listen(0, "127.0.0.1", () => {
      const { port } = srv.address();
      srv.close(() => ok(port));
    });
    srv.on("error", fail);
  });

async function waitFor(url, headers = {}, timeoutMs = 120_000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url, { headers });
      if (res.ok) return;
    } catch {
      /* not up yet */
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`timed out waiting for ${url}`);
}

function start(cmd, args, env) {
  const child = spawn(cmd, args, {
    cwd: root,
    env: { ...process.env, ...env },
    stdio: "ignore",
    detached: true,
  });
  return () => {
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch {
      /* already gone */
    }
  };
}

const escapeHtml = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

function terminalHtml(cmd, output, ok) {
  return `<!doctype html><meta charset="utf-8"><body style="margin:0;background:#0f1115">
<div style="font:13px/1.45 ui-monospace,Menlo,monospace;color:#d6dae0;padding:18px 22px;white-space:pre-wrap;max-width:1100px">
<div style="color:#8ea6ff">$ ${escapeHtml(cmd)}</div>${escapeHtml(output)}
<div style="margin-top:10px;color:${ok ? "#7ee2a8" : "#ff8e8e"}">${ok ? "✔ exit 0" : "✘ failed"}</div></div></body>`;
}

async function main() {
  const { out, shots, dark } = parseArgs(process.argv.slice(2));
  mkdirSync(out, { recursive: true });
  const browser = await chromium.launch();
  const stops = [];
  let failed = false;
  try {
    let base = null;
    if (shots.some((s) => s.kind === "page")) {
      const [apiPort, webPort] = [await freePort(), await freePort()];
      const data = mkdtempSync(join(tmpdir(), "mnemo-shot-"));
      stops.push(() => rmSync(data, { recursive: true, force: true }));
      stops.push(
        start("cargo", ["run", "-q", "-p", "mnemo-api", "--", "serve"], {
          PORT: String(apiPort),
          DATA_DIR: data,
          ALLOWED_ORIGINS: `http://localhost:${webPort}`,
        }),
      );
      stops.push(
        start(
          "pnpm",
          ["--dir", "client", "exec", "vite", "--port", String(webPort), "--strictPort"],
          {
            VITE_PORT: String(webPort),
            MNEMO_API_URL: `http://127.0.0.1:${apiPort}`,
          },
        ),
      );
      await waitFor(`http://127.0.0.1:${apiPort}/api/v1/health`, { Host: `localhost:${apiPort}` });
      base = `http://localhost:${webPort}`;
      await waitFor(base);
    }
    for (const shot of shots) {
      const file = join(out, `${shot.name}.png`);
      if (shot.kind === "page") {
        for (const scheme of dark ? ["light", "dark"] : ["light"]) {
          const page = await browser.newPage({
            viewport: { width: 1280, height: 800 },
            colorScheme: scheme,
          });
          await page.goto(base + shot.target, { waitUntil: "networkidle" });
          const path = scheme === "dark" ? file.replace(/\.png$/, "-dark.png") : file;
          await page.screenshot({ path, fullPage: true });
          await page.close();
          console.log(`saved ${path}`);
        }
      } else {
        const res = spawnSync("sh", ["-c", `${shot.target} 2>&1`], {
          cwd: process.cwd(),
          encoding: "utf8",
          maxBuffer: 1 << 26,
        });
        const ok = res.status === 0;
        failed ||= !ok;
        const lines = (res.stdout ?? "").trimEnd().split("\n");
        const shown = lines.length > 60 ? ["…", ...lines.slice(-60)].join("\n") : lines.join("\n");
        const page = await browser.newPage({ viewport: { width: 1140, height: 400 } });
        await page.setContent(terminalHtml(shot.target, "\n" + shown, ok));
        await page.screenshot({ path: file, fullPage: true });
        await page.close();
        console.log(`saved ${file}${ok ? "" : " (command failed)"}`);
      }
    }
  } finally {
    await browser.close();
    for (const stop of stops.reverse()) stop();
  }
  process.exit(failed ? 1 : 0);
}

main().catch((err) => {
  console.error(err.message ?? err);
  process.exit(2);
});
