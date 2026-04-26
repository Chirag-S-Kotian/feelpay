import requests
import json

BASE_URL = "http://127.0.0.1:8080"

def create_test_merchant():
    url = f"{BASE_URL}/v1/merchants/register"
    payload = {
        "name": "Test Merchant",
        "email": "test@feelpay.com",
        "password": "password123",
        "business_name": "Test Store",
        "business_type": "individual"
    }
    
    try:
        response = requests.post(url, json=payload)
        if response.status_code == 201:
            print("Successfully created test merchant:")
            print(json.dumps(response.json(), indent=2))
        elif response.status_code == 409:
            print("Merchant already exists. Proceeding...")
        else:
            print(f"Failed to create merchant: {response.status_code}")
            print(response.text)
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    create_test_merchant()
