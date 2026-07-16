import http.server
import json
import os
import urllib.error
import urllib.request

PLATFORM_URL = "https://push.example.com"
CLIENT_ID = "your-client-id"
API_KEY = "your-api-key"
VAPID_PUBLIC_KEY = "your-vapid-public-key"

PORT = 8000


class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/api/config":
            self.send_json(200, {"vapidPublicKey": VAPID_PUBLIC_KEY})
            return
        super().do_GET()

    def do_POST(self):
        if self.path == "/api/subscribe":
            self.handle_subscribe()
        elif self.path == "/api/send":
            self.handle_send()
        else:
            self.send_json(404, {"error": "not found"})

    def read_json_body(self):
        length = int(self.headers.get("Content-Length", 0) or 0)
        raw = self.rfile.read(length) if length else b"{}"
        try:
            return json.loads(raw or b"{}")
        except json.JSONDecodeError:
            return {}

    def send_json(self, status, data):
        body = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def forward_to_platform(self, path, payload):
        request = urllib.request.Request(
            PLATFORM_URL.rstrip("/") + path,
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Content-Type": "application/json",
                "Authorization": "Bearer " + API_KEY,
            },
            method="POST",
        )
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                raw = response.read()
                data = json.loads(raw) if raw else {}
                return response.status, data
        except urllib.error.HTTPError as error:
            raw = error.read()
            try:
                data = json.loads(raw) if raw else {}
            except json.JSONDecodeError:
                data = {"error": raw.decode("utf-8", "ignore")}
            return error.code, data
        except urllib.error.URLError as error:
            return 502, {"error": str(error.reason)}

    def handle_subscribe(self):
        body = self.read_json_body()
        payload = {
            "client_id": CLIENT_ID,
            "subscription": body.get("subscription"),
            "user_agent": body.get("user_agent"),
        }
        status, data = self.forward_to_platform("/subscriptions", payload)
        self.send_json(status, data)

    def handle_send(self):
        body = self.read_json_body()
        payload = {
            "client_id": CLIENT_ID,
            "to_all": True,
            "ttl": 3600,
            "payload": {
                "title": body.get("title"),
                "body": body.get("body"),
                "icon": body.get("icon"),
                "data": {
                    "url": body.get("url", "/"),
                    "id": "demo-" + os.urandom(4).hex(),
                },
            },
        }
        status, data = self.forward_to_platform("/send", payload)
        self.send_json(status, data)


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    server = http.server.HTTPServer(("0.0.0.0", PORT), Handler)
    print(f"Serving on http://localhost:{PORT}")
    server.serve_forever()
