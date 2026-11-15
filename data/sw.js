/*
 * SPDX-License-Identifier: GPL-3.0-only
 * Copyright (c) 2026 Robert Longbottom (@thomaslr)
 * Commercial licensing available — see LICENSE for details.
 */

// TimeStation Service Worker (Network-First App Shell, API bypass)
const CACHE_NAME = 'timestation-pwa-v3';

const STATIC_ASSETS = [
  '/',
  '/style.css',
  '/app.js',
  '/favicon.svg',
  '/icon-192.png',
  '/icon-512.png',
  '/manifest.json'
];

// Install: Pre-cache critical App Shell assets sequentially to avoid
// overwhelming the ESP32 HTTP socket pool, then activate immediately.
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(async (cache) => {
      for (const asset of STATIC_ASSETS) {
        try {
          await cache.add(asset);
        } catch (err) {
          console.warn('[PWA] Pre-cache skipped:', asset, err);
        }
      }
    }).then(() => self.skipWaiting())
  );
});

// Activate: Purge old cache versions and claim clients immediately.
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) =>
      Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      )
    ).then(() => self.clients.claim())
  );
});

// Fetch: Bypass non-GET, cross-origin, and /api/ routes; Network-First for App Shell.
self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') {
    return;
  }

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) {
    return;
  }

  if (
    url.pathname.startsWith('/api/') ||
    url.pathname === '/setup' ||
    url.pathname === '/hotspot-detect.html' ||
    url.pathname === '/generate_204' ||
    url.pathname === '/gen_204' ||
    url.pathname === '/ncsi.txt' ||
    url.pathname === '/connecttest.txt'
  ) {
    return;
  }

  event.respondWith(
    fetch(request)
      .then((networkResponse) => {
        if (networkResponse && networkResponse.status === 200) {
          const responseClone = networkResponse.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(request, responseClone);
          });
        }
        return networkResponse;
      })
      .catch(() =>
        caches.match(request).then((cachedResponse) => {
          if (cachedResponse) {
            return cachedResponse;
          }
          if (request.mode === 'navigate') {
            return caches.match('/');
          }
          return Promise.reject(new Error('Offline and asset not cached'));
        })
      )
  );
});

