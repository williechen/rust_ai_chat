// 新增 apps/desktop_pet/frontend/tests/duplicate-preview-contract.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const script = html.match(/<script>([\s\S]*?)<\/script>/)?.[1];
assert.ok(script, "缺少 Tauri bridge script");

test("重複檔案 IPC 只接受後端授權 root，不傳入任意路徑", async () => {
  const calls = [];
  const window = {
    __TAURI__: {
      core: {
        invoke: async (...args) => {
          calls.push(args);
          return {
            groups: [{
              digestSha256: "abc",
              sizeBytes: 4,
              itemIds: ["item-000000", "item-000001"],
            }],
            warnings: [],
          };
        },
      },
      event: { listen: async () => () => {} },
    },
  };
  vm.runInNewContext(script, { window });
  assert.equal(typeof window.scanDuplicatePreview, "function");
  const result = await window.scanDuplicatePreview("/etc");
  assert.equal(result.groups.length, 1);
  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], "scan_duplicate_preview");
  assert.equal(calls[0].length, 1, "不可把任意 path 傳給後端");
});