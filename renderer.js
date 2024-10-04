window.addEventListener('DOMContentLoaded', async () => {
  const peerUrl = document.querySelector('#peerUrl');
  const message = document.querySelector('#message');

  const results = document.querySelector('#results');

  const print = (...args) => {
    for (const arg of args) {
      results.insertBefore(
        document.createTextNode(arg + '\n'),
        results.childNodes[0],
      );
    }
  }

  document.querySelector('#sendForm')
    .addEventListener('submit', async (evt) => {
      evt.preventDefault();
      try {
        await window.hiRust.send(peerUrl.value, message.value);
        print(`sent ${message.value.length} characters to ${peerUrl.value}`);
      } catch (e) {
        print('send error: ' + e);
      }
      message.value = '';
    });

  print('Node.js Version: ' + window.hiRust.nodeVersion());
  print('Chrome Version: ' + window.hiRust.chromeVersion());
  print('Electron Version: ' + window.hiRust.electronVersion());

  setInterval(async () => {
    const eventList = await window.hiRust.getEvents();
    for (const evt of eventList) {
      print('Event: ' + evt);
    }
  }, 1000);

  try {
    print('Connecting...');
    const peerUrl = await window.hiRust.connect('wss://sbd.holo.host');
    print('Connected: ' + peerUrl);
  } catch (e) {
    print('Error: ' + e);
  }
});
