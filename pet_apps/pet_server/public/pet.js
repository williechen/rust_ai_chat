const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const stateEl = document.querySelector("#state");
const revisionEl = document.querySelector("#revision");
const errorEl = document.querySelector("#error");

function render(snapshot) {
  stateEl.textContent = snapshot.state;
  revisionEl.textContent = snapshot.revision;
}

async function loadState() {
  const snapshot = await invoke("get_pet_state");
  render(snapshot);
}

async function send(type) {
  errorEl.textContent = "";

  try {
    const snapshot = await invoke(
      "send_pet_command",
      {
        command: type
      },
    );

    render(snapshot);
  } catch (error) {
    errorEl.textContent = String(error);
  }
}

for (const button of document.querySelectorAll(
  "[data-command]",
)) {
  button.addEventListener("click", () => {
    send(button.dataset.command);
  });
}

listen("pet://state-changed", (event) => {
  render(event.payload);
});

loadState().catch((error) => {
  errorEl.textContent = String(error);
});