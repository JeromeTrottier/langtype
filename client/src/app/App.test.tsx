import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

function renderApp(path = "/") {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={[path]}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

afterEach(() => vi.unstubAllGlobals());

describe("App shell", () => {
  it("shows the spaces page with translated text", () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() => new Promise(() => {})),
    );
    renderApp();
    expect(screen.getByRole("heading", { name: "Your spaces" })).toBeInTheDocument();
    expect(screen.getByText("Connecting to the local server…")).toBeInTheDocument();
  });

  it("reports a connected server", async () => {
    const body = { status: "ok", version: "0.1.0", schema_version: 1 };
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(JSON.stringify(body), { status: 200 })),
    );
    renderApp();
    expect(await screen.findByText("Local server connected (v0.1.0)")).toBeInTheDocument();
  });

  it("explains how to start a stopped server", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response("down", { status: 502 })),
    );
    renderApp();
    expect(await screen.findByRole("alert")).toHaveTextContent("isn't running");
  });

  it("routes to settings", () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() => new Promise(() => {})),
    );
    renderApp("/settings");
    expect(screen.getByRole("heading", { name: "Settings" })).toBeInTheDocument();
  });
});
