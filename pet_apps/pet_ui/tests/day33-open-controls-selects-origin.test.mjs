import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const script = html.match(/<script>([\s\S]*?)<\/script>/)?.[1];
assert.ok(script, "index.html must contain the real Tauri bridge script");

function boot(search, failSelection = false) {
  const calls = [];
  const location = { search };
  const window = {
    location,
    __TAURI__: {
      core: {
        invoke: async (name, args) => {
          calls.push([name, args]);
          if (name === "select_pet" && failSelection) {
            throw new Error("selection failed");
          }
          return name === "active_pet" ? { id: "fallback" } : undefined;
        }
      },
      event: { listen: async () => () => {} }
    }
  };
  vm.runInNewContext(script, { window, location, URLSearchParams });
  return { window, calls };
}

test("pet window button selects its own pet before opening main", async () => {
  const { window, calls } = boot("?view=pet&pet_id=pet-b");
  await window.openPetControls();
  assert.equal(calls.length, 2);
  assert.equal(calls[0][0], "select_pet");
  assert.equal(calls[0][1].id, "pet-b");
  assert.equal(calls[1][0], "open_control_window");
});

test("control window only focuses itself, never changes selection", async () => {
  const { window, calls } = boot("");
  await window.openPetControls();
  assert.deepEqual(calls.map(([name]) => name), ["open_control_window"]);
});

test("selection failure does not report successful opening", async () => {
  const { window, calls } = boot("?view=pet&pet_id=pet-b", true);
  await assert.rejects(window.openPetControls(), /selection failed/);
  assert.deepEqual(calls.map(([name]) => name), ["select_pet"]);
});