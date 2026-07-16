function urlBase64ToUint8Array(base64String) {
  const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
  const rawData = atob(base64);
  const outputArray = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; i++) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

const logEl = document.getElementById("log");
const statusDot = document.getElementById("status-dot");
const statusText = document.getElementById("status-text");
const subscribeBtn = document.getElementById("subscribe-btn");
const sendBtn = document.getElementById("send-btn");
const iconInputs = document.querySelectorAll('input[name="icon"]');

let vapidPublicKey = null;

function log(message, kind) {
  const line = document.createElement("div");
  line.className = "log-line" + (kind ? " log-" + kind : "");
  const time = new Date().toLocaleTimeString();
  line.textContent = "[" + time + "] " + message;
  logEl.prepend(line);
}

function setStatus(connected, label) {
  statusDot.classList.toggle("dot-on", connected);
  statusText.textContent = label;
}

function selectedIcon() {
  for (const input of iconInputs) {
    if (input.checked) return input.value;
  }
  return "";
}

async function loadConfig() {
  const res = await fetch("/api/config");
  const data = await res.json();
  vapidPublicKey = data.vapidPublicKey;
}

async function subscribe() {
  if (!("serviceWorker" in navigator) || !("PushManager" in window)) {
    log("Push notifications are not supported in this browser", "error");
    return;
  }

  subscribeBtn.disabled = true;
  try {
    if (!vapidPublicKey) {
      log("Fetching VAPID public key from server...");
      await loadConfig();
    }

    log("Registering service worker...");
    const reg = await navigator.serviceWorker.register("./sw.js");
    await navigator.serviceWorker.ready;

    log("Requesting notification permission...");
    const permission = await Notification.requestPermission();
    if (permission !== "granted") {
      log("Permission denied: " + permission, "error");
      setStatus(false, "Permission denied");
      subscribeBtn.disabled = false;
      return;
    }

    log("Subscribing with VAPID public key...");
    const subscription = await reg.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: urlBase64ToUint8Array(vapidPublicKey),
    });

    log("Registering subscription on the server...");
    const res = await fetch("/api/subscribe", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        subscription: subscription.toJSON(),
        user_agent: navigator.userAgent,
      }),
    });

    const data = await res.json();
    if (!res.ok) {
      throw new Error(data.error ? JSON.stringify(data.error) : `Server responded with ${res.status}`);
    }

    log("Subscribed successfully", "success");
    setStatus(true, "Subscribed");
    sendBtn.disabled = false;
  } catch (err) {
    log("Subscription failed: " + err.message, "error");
    setStatus(false, "Not subscribed");
  } finally {
    subscribeBtn.disabled = false;
  }
}

async function sendPush() {
  const title = document.getElementById("push-title").value.trim() || "Untitled notification";
  const bodyText = document.getElementById("push-body").value.trim() || "";
  const url = document.getElementById("push-url").value.trim() || "/";
  const icon = selectedIcon();

  sendBtn.disabled = true;
  try {
    log("Sending push via server...");
    const res = await fetch("/api/send", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title, body: bodyText, icon, url }),
    });

    const data = await res.json();
    if (!res.ok) {
      throw new Error(data.error ? JSON.stringify(data.error) : `Server responded with ${res.status}`);
    }

    log("Push request accepted by platform", "success");
  } catch (err) {
    log("Send failed: " + err.message, "error");
  } finally {
    sendBtn.disabled = false;
  }
}

subscribeBtn.addEventListener("click", subscribe);
sendBtn.addEventListener("click", sendPush);

if (!("serviceWorker" in navigator) || !("PushManager" in window)) {
  setStatus(false, "Not supported");
  subscribeBtn.disabled = true;
} else {
  setStatus(false, "Not subscribed");
}
