import requests


class FeelPay:
    def __init__(self, api_key: str, base_url: str = "https://api.feelpay.in/v1", api_version: str = "2026-04-28"):
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.api_version = api_version

    def request(self, method: str, path: str, json=None):
        response = requests.request(
            method,
            f"{self.base_url}{path}",
            headers={
                "x-api-key": self.api_key,
                "x-api-version": self.api_version,
                "content-type": "application/json",
            },
            json=json,
            timeout=30,
        )
        response.raise_for_status()
        return response.json() if response.content else None

    def create_payment(self, **payload):
        return self.request("POST", "/payments", payload)

    def get_payment(self, payment_id: str):
        return self.request("GET", f"/payments/{payment_id}")
