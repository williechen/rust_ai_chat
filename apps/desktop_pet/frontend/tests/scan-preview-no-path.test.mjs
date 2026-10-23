import assert from "node:assert/strict";
import { URLSearchParams } from "node:url";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const location = {
  search: ""
};

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const script = html.match(/<script>([\s\S]*?)<\/script>/)?.[1];
assert.ok(script, "找不到前端 Tauri bridge script");

test("掃描預覽不得接受使用者指定路徑", async () => {
  const calls = [];
  const window = {
    __TAURI__: {
      core: { invoke: async (...args) => {
        calls.push(args);
        return { files: [], duplicateSizeCandidates: [], warnings: [] };
      }},
      event: { listen: async () => () => {} },
      location: location,
    },
  };
  vm.runInNewContext(script, { window, location, URLSearchParams });
  assert.equal(typeof window.scanFilePreview, "function");
  await window.scanFilePreview("/etc");
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], "scan_file_preview");
  assert.equal(calls[0].length, 1);
});