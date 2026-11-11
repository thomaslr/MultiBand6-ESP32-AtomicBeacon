/*
 * SPDX-License-Identifier: GPL-3.0-only
 * Copyright (c) 2026 Robert Longbottom (@thomaslr)
 * Commercial licensing available — see LICENSE for details.
 */

// TimeStation Minimal Service Worker for PWA Standalone Installation
// Intentionally avoids intercepting fetch requests so the single-threaded ESP32 HTTP
// server is never burdened by duplicate proxy fetches or synthetic offline rejections.
const CACHE_NAME = 'timestation-pwa-v2';

self.addEventListener('install', () => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.map((k) => caches.delete(k)))
    ).then(() => self.clients.claim())
  );
});

