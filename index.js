"use strict";
const path = require("path");
const { app, BrowserWindow, Menu, protocol, shell } = require("electron");
/// const {autoUpdater} = require('electron-updater');
const { is } = require("electron-util");
const unhandled = require("electron-unhandled");
const debug = require("electron-debug");
const contextMenu = require("electron-context-menu");
const config = require("./config.js");
const menu = require("./menu.js");
const windows = require("./windows.js");
const { autoUpdater } = require("electron-updater");
const { parsePhoneNumber } = require("libphonenumber-js/mobile");

unhandled();
contextMenu();
if (is.windows) {
  windows().then();
}

// Note: Must match `build.appId` in package.json
app.setAppUserModelId("me.lucasfreitas.WhatsAppCaller");
if (process.defaultApp) {
  if (process.argv.length >= 2) {
    app.setAsDefaultProtocolClient("tel", process.execPath, [
      path.resolve(process.argv[1]),
    ]);
    app.setAsDefaultProtocolClient("callto", process.execPath, [
      path.resolve(process.argv[1]),
    ]);
  }
} else {
  app.setAsDefaultProtocolClient("tel");
  app.setAsDefaultProtocolClient("callto");
}

// Prevent window from being garbage collected
let mainWindow;

const createMainWindow = async () => {
  const win = new BrowserWindow({
    title: app.name,
    show: false,
    width: 1,
    height: 1,
  });

  win.on("ready-to-show", () => {
    // win.show();
  });

  win.on("closed", () => {
    // Dereference the window
    // For multiple windows store them in an array
    mainWindow = undefined;
  });

  return win;
};

// Prevent multiple instances of the app
if (!app.requestSingleInstanceLock()) {
  app.quit();
}

app.on("second-instance", () => {
  if (mainWindow) {
    if (mainWindow.isMinimized()) {
      mainWindow.restore();
    }

    mainWindow.show();
  }
});

app.on("window-all-closed", () => {
  app.quit();
});

app.on("activate", async () => {
  if (!mainWindow) {
    mainWindow = await createMainWindow();
  }
});

const parseNumberManual = (number) => {
  const cleanRegex = new RegExp(/^(\+)|\D/, "g");
  let cleanNumber = String(number).trim();
  cleanNumber.replace(/^(tel|callto):\/\//, "");
  cleanNumber = cleanNumber.replace(cleanRegex, "$1");
  if (cleanNumber.startsWith("0")) {
    cleanNumber = "49" + cleanNumber.substr(1);
  }
  return cleanNumber;
};

const parseNumber = (number) => {
  const tryWithDE = parsePhoneNumber(number, "DE");
  const tryWithBR = parsePhoneNumber(number, "BR");
  if (tryWithDE.isValid() && tryWithDE.getType() === 'MOBILE') {
    return String(tryWithDE.number).replace('+', '')
  }
  if (tryWithBR.isValid() && tryWithBR.getType() === 'MOBILE') {
    return String(tryWithBR.number).replace('+', '')
  }
};

app.on("open-url", async (event, url) => {
  await handleWhatsAppLink(mainWindow, parseNumber(url));
  app.quit();
});

app.on("ready", async () => {
  await autoUpdater.checkForUpdatesAndNotify();
});

autoUpdater.on("update-downloaded", (info) => {
  autoUpdater.quitAndInstall();
});

const handleWhatsAppLink = async (win, phoneNumber) => {
  if (protocol.isProtocolRegistered("whatsapp")) {
    await shell.openExternal(`whatsapp://send?phone=${phoneNumber}`);
  } else {
    shell.openExternal(`https://api.whatsapp.com/send/?phone=${phoneNumber}`);
  }
};

const noop = (request, callback) => {};

(async () => {
  await app.whenReady();
  protocol.registerStringProtocol("callto", noop);
  protocol.registerStringProtocol("tel", noop);
  Menu.setApplicationMenu(menu);
  mainWindow = await createMainWindow();
})();
