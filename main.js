const {app, BrowserWindow, ipcMain} = require('electron');
const path = require('path');
const hiRust = require('./hi-rust/index.node');

function createWindow () {
  const mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  });

  mainWindow.loadFile('index.html');
}

app.whenReady().then(() => {
  ipcMain.handle('connect', async (_evt, sig_url) => {
    return await hiRust.connect(sig_url);
  });

  ipcMain.handle('getEvents', async (_evt) => {
    return await hiRust.getEvents();
  });

  ipcMain.handle('send', async (_evt, peer_url, data) => {
    return await hiRust.send(peer_url, data);
  });

  createWindow();

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  });
})

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') app.quit()
})
