self.addEventListener("install", () => {
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

self.addEventListener("push", (event) => {
  let payload = { title: "Notification", body: "New message" };

  if (event.data) {
    try {
      const text = event.data.text();
      try {
        payload = Object.assign(payload, JSON.parse(text));
      } catch (_) {
        payload.body = text;
      }
    } catch (e) {
      console.error("[sw] failed to read push data", e);
    }
  }

  const options = {
    body: payload.body || "New message",
    icon: payload.icon || "/icon.png",
    badge: payload.badge || "/icon.png",
    image: payload.image,
    data: payload.data || {},
    tag: payload.tag,
    renotify: payload.renotify === true,
    requireInteraction: payload.requireInteraction === true,
    actions: payload.actions || [],
    dir: payload.dir || "auto",
    lang: payload.lang || "en-US",
    vibrate: payload.vibrate || [200, 100, 200],
    timestamp: payload.timestamp || Date.now(),
  };

  event.waitUntil(self.registration.showNotification(payload.title, options));
});

self.addEventListener("notificationclick", (event) => {
  event.notification.close();

  const targetUrl = (event.notification.data && event.notification.data.url)
      ? event.notification.data.url
      : "/";

  event.waitUntil(
      (async () => {
        const resolvedUrl = new URL(targetUrl, self.location.origin).href;
        const allClients = await self.clients.matchAll({ type: "window", includeUncontrolled: true });
        for (const client of allClients) {
          if (client.url === resolvedUrl && "focus" in client) {
            await client.focus();
            return;
          }
        }
        await self.clients.openWindow(resolvedUrl);
      })(),
  );
});

self.addEventListener("notificationclose", (event) => {
  if (event.notification && event.notification.data && event.notification.data.track) {
    fetch(event.notification.data.track, {
      method: "POST",
      body: JSON.stringify({
        event: "notification_close",
        notif_id: event.notification.data.id,
      }),
    }).catch(() => {});
  }
});