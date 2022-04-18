const { Registry } = require('rage-edit');

const windowsHandler = (async () => {
  await Registry.set('HKCU\\Software\\Your app name\\Capabilities', 'ApplicationName', 'WhatsApp Caller');
  await Registry.set('HKCU\\Software\\Your app name\\Capabilities', 'ApplicationDescription', 'WhatsApp Caller');

  await Registry.set('HKCU\\Software\\Your app name\\Capabilities\\URLAssociations', 'tel', 'WhatsApp Caller.tel');
  await Registry.set('HKCU\\Software\\Your app name\\Capabilities\\URLAssociations', 'callto', 'WhatsApp Caller.callto');

  await Registry.set('HKCU\\Software\\Classes\\Your app name.tel\\DefaultIcon', '', process.execPath);

  await Registry.set('HKCU\\Software\\Classes\\Your app name.tel\\shell\\open\\command', '', `"${process.execPath}" "%1"`);

  await Registry.set('HKCU\\Software\\RegisteredApplications', 'WhatsApp Caller', 'Software\\WhatsApp Caller\\Capabilities');
});

module.exports = windowsHandler;

