const status = document.querySelector('#status');
const invoke = window.__TAURI__?.core?.invoke;
const listen = window.__TAURI__?.event?.listen;

function render(snapshot) {
  status.textContent = `狀態：${snapshot.state}（版本 ${snapshot.revision}）`;
}

if (!invoke || !listen) {
  status.textContent = '請從 Tauri 視窗啟動';
} else {
  listen('pet://state-changed', () => invoke('get_pet_state').then(render));
  invoke('get_pet_state').then(render).catch(error => { status.textContent = String(error); });
  document.querySelectorAll('[data-command]').forEach(button => {
    button.addEventListener('click', async () => {
      try {
        render(await invoke('send_pet_command', { command: button.dataset.command }));
      } catch (error) {
        status.textContent = `命令失敗：${error}`;
      }
    });
  });
}
