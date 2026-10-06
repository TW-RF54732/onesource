(() => {
  const link = document.getElementById('install-destination');
  if (link) location.replace(link.getAttribute('href') + location.search + location.hash);
})();
