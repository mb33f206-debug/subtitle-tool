// Boot script: removes loading screen after delay, independent of app.js module
(function() {
  function hideLoading() {
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

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function() {
      setTimeout(hideLoading, 1500);
    });
  } else {
    setTimeout(hideLoading, 1500);
  }
})();
