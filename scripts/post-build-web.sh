#!/bin/bash
# Post-build script for Flutter web — fixes service worker caching issues.
# Run AFTER `flutter build web` and BEFORE deploying.
#
# 1. Replaces Flutter's empty SW with a cleanup SW that self-destructs
# 2. Injects ?v=<timestamp> into main.dart.js reference for cache busting
set -euo pipefail

BUILD_DIR="${1:-frontend/build/web}"
TIMESTAMP=$(date +%s)

if [ ! -d "$BUILD_DIR" ]; then
  echo "Error: Build directory $BUILD_DIR not found. Run 'flutter build web' first." >&2
  exit 1
fi

# 1. Inject cleanup service worker (replaces old ones in browsers)
cat > "${BUILD_DIR}/flutter_service_worker.js" << 'EOF'
// Cleanup service worker: unregisters itself and clears all caches.
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', event => {
  event.waitUntil(
    (async () => {
      const cacheNames = await caches.keys();
      await Promise.all(cacheNames.map(name => caches.delete(name)));
      await self.clients.claim();
      await self.registration.unregister();
    })()
  );
});
EOF

# 2. Cache-bust main.dart.js with timestamp query param
sed -i "s|main\.dart\.js|main.dart.js?v=${TIMESTAMP}|g" "${BUILD_DIR}/flutter_bootstrap.js"

echo "Post-build: cleanup SW injected, main.dart.js cache-busted (v=${TIMESTAMP})"
