const { Registry } = require("rage-edit");

const windowsHandler = async () => {
  await Registry.set(
    "HKCU\\Software\\WhatsApp Caller\\Capabilities",
    "ApplicationName",
    "WhatsApp Caller"
  );
  await Registry.set(
    "HKCU\\Software\\WhatsApp Caller\\Capabilities",
    "ApplicationDescription",
    "WhatsApp Caller"
  );

  await Registry.set(
    "HKCU\\Software\\WhatsApp Caller\\Capabilities\\URLAssociations",
    "tel",
    "WhatsApp Caller.tel"
  );
  await Registry.set(
    "HKCU\\Software\\WhatsApp Caller\\Capabilities\\URLAssociations",
    "callto",
    "WhatsApp Caller.callto"
  );

  await Registry.set(
    "HKCU\\Software\\Classes\\WhatsApp Caller.tel\\DefaultIcon",
    "",
    process.execPath
  );
  await Registry.set(
    "HKCU\\Software\\Classes\\WhatsApp Caller.callto\\DefaultIcon",
    "",
    process.execPath
  );

  await Registry.set(
    "HKCU\\Software\\Classes\\WhatsApp Caller.tel\\shell\\open\\command",
    "",
    `"${process.execPath}" "%1"`
  );
  await Registry.set(
    "HKCU\\Software\\Classes\\WhatsApp Caller.callto\\shell\\open\\command",
    "",
    `"${process.execPath}" "%1"`
  );

  await Registry.set(
    "HKCU\\Software\\RegisteredApplications",
    "WhatsApp Caller",
    "Software\\WhatsApp Caller\\Capabilities"
  );
};

module.exports = windowsHandler;
