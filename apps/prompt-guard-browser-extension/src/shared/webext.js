(function initPollekWebExt(global) {
  const api = global.browser || global.chrome;
  const usesPromiseApi = Boolean(global.browser && api === global.browser);

  function runtimeError() {
    const error = api.runtime?.lastError;
    return error ? new Error(error.message) : null;
  }

  function storageGet(defaults) {
    if (usesPromiseApi) {
      return api.storage.local.get(defaults);
    }

    return new Promise((resolve, reject) => {
      try {
        api.storage.local.get(defaults, (items) => {
          const error = runtimeError();
          if (error) reject(error);
          else resolve(items);
        });
      } catch (error) {
        reject(error);
      }
    });
  }

  function storageSet(values) {
    if (usesPromiseApi) {
      return api.storage.local.set(values);
    }

    return new Promise((resolve, reject) => {
      try {
        api.storage.local.set(values, () => {
          const error = runtimeError();
          if (error) reject(error);
          else resolve();
        });
      } catch (error) {
        reject(error);
      }
    });
  }

  function sendMessage(message) {
    if (usesPromiseApi) {
      return api.runtime.sendMessage(message);
    }

    return new Promise((resolve, reject) => {
      try {
        api.runtime.sendMessage(message, (response) => {
          const error = runtimeError();
          if (error) reject(error);
          else resolve(response);
        });
      } catch (error) {
        reject(error);
      }
    });
  }

  global.PollekWebExt = {
    api,
    storageGet,
    storageSet,
    sendMessage,
  };
})(globalThis);
