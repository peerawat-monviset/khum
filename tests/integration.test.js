import { test, expect, beforeAll, afterAll } from "bun:test";
import { spawn } from "child_process";

let serverProcess;
const PORT = "8099";
const BASE_URL = `http://127.0.0.1:${PORT}`;

beforeAll(async () => {
  // Spawn the compiled debug Rust backend binary for rapid local E2E iterations
  serverProcess = spawn("./target/debug/khum", [], {
    env: { ...process.env, PORT: PORT },
    cwd: process.cwd() // Ensure working directory matches project root for static assets
  });

  // Print server logs to test console for debugging
  serverProcess.stderr.on("data", (data) => console.error(`Server stderr: ${data}`));

  // Wait for the TCP socket to bind
  await new Promise(resolve => setTimeout(resolve, 800));
});

afterAll(() => {
  if (serverProcess) {
    serverProcess.kill("SIGKILL");
  }
});

test("GET / (Serve SolidJS Root HTML)", async () => {
  const res = await fetch(`${BASE_URL}/`);
  expect(res.status).toBe(200);
  const text = await res.text();
  expect(text).toContain('id="root"');
  expect(text).toContain('src="/main.js"');
});

test("GET /api/calculate (Calculates final prices)", async () => {
  const res = await fetch(`${BASE_URL}/api/calculate?basket=250&distance=3.5`);
  expect(res.status).toBe(200);
  expect(res.headers.get("content-type")).toContain("application/json");

  const data = await res.json();
  expect(data.best).toBeDefined();
  expect(data.providers).toBeArray();
  expect(data.providers.length).toBe(4);

  // Check structure of each provider calculated response
  for (const p of data.providers) {
    expect(p.key).toBeDefined();
    expect(p.name).toBeDefined();
    expect(p.final).toBeTypeOf("number");
    expect(p.original).toBeTypeOf("number");
  }
});

test("GET /api/metrics (JSON metrics)", async () => {
  const res = await fetch(`${BASE_URL}/api/metrics`);
  expect(res.status).toBe(200);
  expect(res.headers.get("content-type")).toContain("application/json");

  const data = await res.json();
  expect(data.mem_current_mb).toBeTypeOf("number");
  expect(data.mem_limit_mb).toBeTypeOf("number");
  expect(data.cpu_percent).toBeTypeOf("number");
});

test("GET /api/sysinfo (JSON system spec details)", async () => {
  const res = await fetch(`${BASE_URL}/api/sysinfo`);
  expect(res.status).toBe(200);
  expect(res.headers.get("content-type")).toContain("application/json");

  const data = await res.json();
  expect(data.os).toBeDefined();
  expect(data.kernel).toBeDefined();
  expect(data.cpu_model).toBeDefined();
});

test("GET /locales/th.json (JSON Thai translation assets)", async () => {
  const res = await fetch(`${BASE_URL}/locales/th.json`);
  expect(res.status).toBe(200);
  expect(res.headers.get("content-type")).toContain("application/json");

  const data = await res.json();
  expect(data.title).toBe("คุ้ม");
  expect(data.sysHost).toBe("Host");
});

test("GET /api/icon (Redirect to App Store CDN)", async () => {
  const res = await fetch(`${BASE_URL}/api/icon?provider=grab`, { redirect: "manual" });
  // Should return a temporary redirect (307)
  expect(res.status).toBe(307);
  expect(res.headers.get("location")).toContain("mzstatic.com");
});
