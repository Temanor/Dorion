/**
 * Functions for window controls
 */
export function close() {
  window.__TAURI__.invoke('close')
}

export function minimize() {
  window.__TAURI__.invoke('minimize')
}

export function toggleMaximize() {
  window.__TAURI__.invoke('toggle_maximize')
}

export async function setMaximizeIcon() {
  if (await window.__TAURI__.window.appWindow.isMaximized()) {
    const topmax = document.querySelector('#topmax') as HTMLDivElement
    topmax.classList.add('maximized')
  } else {
    const topmax = document.querySelector('#topmax') as HTMLDivElement
    topmax.classList.remove('maximized')
  }
}

export function applyNotificationCount() {
  const { invoke } = window.__TAURI__
  const title = document.querySelector('title') as HTMLTitleElement
  const notifs = title.innerHTML.startsWith('•') ? -1 : title.innerHTML?.match(/\((.*)\)/)?.[1]

  if (!notifs) {
    invoke('notif_count', {
      amount: 0,
    })

    return
  }

  invoke('notif_count', {
    amount: Number(notifs),
  })
}