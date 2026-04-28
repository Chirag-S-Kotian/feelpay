<?php

namespace FeelPay;

class Client
{
    public function __construct(
        private string $apiKey,
        private string $baseUrl = 'https://api.feelpay.in/v1',
        private string $apiVersion = '2026-04-28'
    ) {}

    public function request(string $method, string $path, ?array $payload = null): array
    {
        $ch = curl_init(rtrim($this->baseUrl, '/') . $path);
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_CUSTOMREQUEST => $method,
            CURLOPT_HTTPHEADER => [
                'content-type: application/json',
                'x-api-key: ' . $this->apiKey,
                'x-api-version: ' . $this->apiVersion,
            ],
            CURLOPT_POSTFIELDS => $payload === null ? null : json_encode($payload),
        ]);
        $body = curl_exec($ch);
        $status = curl_getinfo($ch, CURLINFO_RESPONSE_CODE);
        curl_close($ch);

        if ($status >= 400) {
            throw new \RuntimeException('FeelPay API error ' . $status . ': ' . $body);
        }

        return $body ? json_decode($body, true) : [];
    }

    public function createPayment(array $payload): array
    {
        return $this->request('POST', '/payments', $payload);
    }
}
