// TimeStation Minimal Service Worker for PWA Offline / Standalone Installation
const CACHE_NAME = 'timestation-pwa-v1';

self.addEventListener('install', (event) => {
    self.skipWaiting();
});

self.addEventListener('activate', (event) => {
    event.waitUntil(clients.claim());
});

self.addEventListener('fetch', (event) => {
    // Network-first pass-through ensuring real-time telemetry and control
    event.respondWith(
        fetch(event.request).catch(() => {
            // Offline fallback for root navigation if Wi-Fi link drops momentarily
            if (event.request.mode === 'navigate') {
                return caches.match('/');
            }
            return Promise.reject('Offline');
        })
    );
});
