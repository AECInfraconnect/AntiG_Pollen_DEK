(function initPollekEditorDetector(global) {
  function visibleText(element) {
    if (!element) return "";
    if (
      element instanceof HTMLTextAreaElement ||
      element instanceof HTMLInputElement
    ) {
      return element.value || "";
    }
    return element.innerText || element.textContent || "";
  }

  function selectorList(profile, key, fallback) {
    const selectors = Array.isArray(profile?.[key]) ? profile[key] : fallback;
    return selectors.join(",");
  }

  function activeEditor(profile) {
    const active = document.activeElement;
    const selectors = selectorList(
      profile,
      "editorSelectors",
      global.PollekPromptGuardProviders.DEFAULT_EDITOR_SELECTORS,
    );
    if (active?.matches?.(selectors)) {
      return active;
    }
    return null;
  }

  function nearestEditor(target, profile) {
    if (!(target instanceof Element)) return null;
    const selectors = selectorList(
      profile,
      "editorSelectors",
      global.PollekPromptGuardProviders.DEFAULT_EDITOR_SELECTORS,
    );
    const direct = target.closest(selectors);
    if (direct) return direct;
    const form = target.closest("form");
    if (form) {
      const candidates = Array.from(form.querySelectorAll(selectors));
      return candidates.find((candidate) => visibleText(candidate).trim()) ?? null;
    }
    return activeEditor(profile);
  }

  function likelySendButton(target, profile) {
    if (!(target instanceof Element)) return false;
    const selectors =
      profile?.sendButtonSelectors ??
      global.PollekPromptGuardProviders.DEFAULT_SEND_BUTTON_SELECTORS;
    return selectors.some((selector) => Boolean(target.closest(selector)));
  }

  global.PollekEditorDetector = {
    visibleText,
    nearestEditor,
    likelySendButton,
  };
})(globalThis);
