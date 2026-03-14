// Boot script: removes loading screen when app is ready, with fallback timeout
(function() {
  var hidden = false;

  function hideLoading() {
    if (hidden) return;
    hidden = true;
    var loading = document.getElementById('loadingScreen');
    var app = document.getElementById('appMain');
    if (loading) {
      loading.style.opacity = '0';
      loading.style.pointerEvents = 'none';
      setTimeout(function() {
        loading.style.display = 'none';
        if (app) app.style.display = '';
      }, 400);
    } else if (app) {
      app.style.display = '';
    }
  }

  // Listen for app-ready event from app.js (fired after initSettings completes)
  document.addEventListener('app-ready', hideLoading);

  // Fallback: hide loading screen after 5s even if app-ready never fires
  var fallbackDelay = 5000;
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function() {
      setTimeout(hideLoading, fallbackDelay);
    });
  } else {
    setTimeout(hideLoading, fallbackDelay);
  }
})();
