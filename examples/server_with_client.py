import http.server
import json
import os

# openapi-generator-cli generate -i openapi.yaml -g python -o ~/push_client/ --package-name push_client
# pip install ~/push_client/
from push_client import (
    ApiClient,
    Configuration,
    SubscriptionsApi,
    SendApi,
    CreateSubscriptionRequest,
    SubscriptionData,
    SendRequest,
    ApiException
)

PLATFORM_URL = "https://push.example.com"
CLIENT_ID = "your-client-id"
API_KEY = "your-api-key"
VAPID_PUBLIC_KEY = "your-vapid-public-key"

PORT = 8000

api_config = Configuration(host=PLATFORM_URL, access_token=API_KEY)
api_client = ApiClient(api_config)
subscriptions_api = SubscriptionsApi(api_client)
send_api = SendApi(api_client)


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
        body = data if isinstance(data, bytes) else json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def send_api_exception(self, error: ApiException):
        try:
            data = json.loads(error.body) if error.body else {"error": str(error)}
        except json.JSONDecodeError:
            data = {"error": error.body}
        self.send_json(error.status or 502, data)

    def handle_subscribe(self):
        body = self.read_json_body()
        request = CreateSubscriptionRequest(
            client_id=CLIENT_ID,
            subscription=SubscriptionData.from_dict(body.get("subscription", {})),
            user_agent=body.get("user_agent"),
        )
        try:
            response = subscriptions_api.create_subscription(request)
        except ApiException as error:
            self.send_api_exception(error)
            return
        self.send_json(201, response.to_json().encode("utf-8"))

    def handle_send(self):
        body = self.read_json_body()
        request = SendRequest(
            to_all=True,
            ttl=3600,
            payload={
                "title": body.get("title"),
                "body": body.get("body"),
                "icon": body.get("icon"),
                "data": {
                    "url": body.get("url", "/"),
                    "id": "demo-" + os.urandom(4).hex(),
                },
            },
        )
        try:
            response = send_api.send_push(request)
        except ApiException as error:
            self.send_api_exception(error)
            return
        self.send_json(200, response.to_json().encode("utf-8"))


if __name__ == "__main__":
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    server = http.server.HTTPServer(("0.0.0.0", PORT), Handler)
    print(f"Serving on http://localhost:{PORT}")
    server.serve_forever()
