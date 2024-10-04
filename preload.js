const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('hiRust', {
  nodeVersion: () => process.versions.node,
  chromeVersion: () => process.versions.chrome,
  electronVersion: () => process.versions.electron,
  connect: (sig_url) => ipcRenderer.invoke('connect', sig_url),
  getEvents: () => ipcRenderer.invoke('getEvents'),
  send: (peer_url, data) => ipcRenderer.invoke('send', peer_url, data),
})
