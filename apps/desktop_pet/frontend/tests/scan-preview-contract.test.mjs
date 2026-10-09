import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const script = html.match(/<script>([\s\S]*?)<\/script>/)?.[1];
assert.ok(script);
test("scanFilePreview uses read-only command without path", async () => {
  const calls = [];
  const window = { __TAURI__: {
    core: { invoke: async (...args) => {
      calls.push(args);
      return { files: [], duplicateSizeCandidates: [], warnings: [] };
    }},
    event: { listen: async () => () => {} }
  }};
  vm.runInNewContext(script, { window });
  assert.equal(typeof window.scanFilePreview, "function");
  const result = await window.scanFilePreview();
  assert.equal(result.files.length, 0);
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], "scan_file_preview");
  assert.equal(calls[0].length, 1);
});