(() => {
  'use strict';
  const supported = value => {
    if (typeof value !== 'string') return null;
    const primary = value.toLowerCase().split(/[-_]/)[0];
    return primary === 'en' || primary === 'zh' ? primary : null;
  };
  if (location.pathname === '/' || location.pathname === '/index.html') {
    let saved;
    try { saved = supported(localStorage.getItem('onesource-language')); } catch { /* Storage is optional. */ }
    const preferences = navigator.languages?.length ? navigator.languages : [navigator.language];
    const language = saved || preferences.map(supported).find(Boolean) || 'en';
    location.replace(`/${language}/${location.search}${location.hash}`);
  }
  document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('[data-language]').forEach(link => {
      // Keep the native link working even when storage or JavaScript is unavailable.
      link.href = `/${link.dataset.language}/${location.search}${location.hash}`;
      link.addEventListener('click', () => {
        try { localStorage.setItem('onesource-language', link.dataset.language); } catch { /* Navigation still works. */ }
      });
    });
  });
})();
