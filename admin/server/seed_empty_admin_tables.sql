-- FeelPay Admin Dashboard Comprehensive Seed Script
BEGIN;

-- 1. admin_users
INSERT INTO admin_users (id, email, password_hash, name, role, role_id, is_active, mfa_enabled, status, last_login_at, created_at, updated_at)
VALUES 
  ('cd317088-a9ef-495c-ab3e-2a88603c54ac', 'aditya.sharma@feelpay.com', '$2b$12$e8Y6YxV7g2z2GzK9zV7g2uO0G8w3G9m7l3J6s2E6l0A8q7W5b1C4e', 'Aditya Sharma', 'Super Admin', '8780932f-a8f8-42d0-8c08-b3926f65c5df', true, true, 'active', NOW() - INTERVAL '12 minutes', NOW() - INTERVAL '90 days', NOW()),
  ('f7986939-ab81-4b70-a2d1-7253bbf7f0e1', 'harsh.patel@feelpay.com', '$2b$12$e8Y6YxV7g2z2GzK9zV7g2uO0G8w3G9m7l3J6s2E6l0A8q7W5b1C4e', 'Harsh Patel', 'Payment Ops Admin', '6b110772-f4e7-4d36-8e2b-7d613d69745c', true, true, 'active', NOW() - INTERVAL '1 hour', NOW() - INTERVAL '60 days', NOW()),
  ('4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', 'vanya.iyer@feelpay.com', '$2b$12$e8Y6YxV7g2z2GzK9zV7g2uO0G8w3G9m7l3J6s2E6l0A8q7W5b1C4e', 'Vanya Iyer', 'Finance Admin', '5cf22415-d82f-41de-943e-88f9abaa97a4', true, true, 'active', NOW() - INTERVAL '3 hours', NOW() - INTERVAL '45 days', NOW()),
  ('32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', 'rohan.deshmukh@feelpay.com', '$2b$12$e8Y6YxV7g2z2GzK9zV7g2uO0G8w3G9m7l3J6s2E6l0A8q7W5b1C4e', 'Rohan Deshmukh', 'Risk Admin', '13485b7e-c70f-40b7-af02-e0f89aca245f', true, true, 'active', NOW() - INTERVAL '5 hours', NOW() - INTERVAL '30 days', NOW()),
  ('9dc43202-47e4-4b59-981d-9d43b3568469', 'kiran.rao@feelpay.com', '$2b$12$e8Y6YxV7g2z2GzK9zV7g2uO0G8w3G9m7l3J6s2E6l0A8q7W5b1C4e', 'Kiran Rao', 'Compliance Admin', 'ecd75c67-42f6-4d68-bd44-010c2eb65695', true, true, 'active', NOW() - INTERVAL '8 hours', NOW() - INTERVAL '20 days', NOW())
ON CONFLICT (id) DO UPDATE SET
  name = EXCLUDED.name,
  email = EXCLUDED.email,
  role = EXCLUDED.role,
  role_id = EXCLUDED.role_id,
  is_active = EXCLUDED.is_active,
  mfa_enabled = EXCLUDED.mfa_enabled,
  status = EXCLUDED.status,
  last_login_at = EXCLUDED.last_login_at;

-- 2. admin_user_roles
INSERT INTO admin_user_roles (admin_user_id, role_id, assigned_by_admin_id, assigned_at)
VALUES
  ('cd317088-a9ef-495c-ab3e-2a88603c54ac', '8780932f-a8f8-42d0-8c08-b3926f65c5df', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '90 days'),
  ('f7986939-ab81-4b70-a2d1-7253bbf7f0e1', '6b110772-f4e7-4d36-8e2b-7d613d69745c', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '60 days'),
  ('4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', '5cf22415-d82f-41de-943e-88f9abaa97a4', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '45 days'),
  ('32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', '13485b7e-c70f-40b7-af02-e0f89aca245f', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '30 days'),
  ('9dc43202-47e4-4b59-981d-9d43b3568469', 'ecd75c67-42f6-4d68-bd44-010c2eb65695', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '20 days')
ON CONFLICT (admin_user_id, role_id) DO NOTHING;

-- 3. admin_notifications
INSERT INTO admin_notifications (
  id, severity, type, title, message, related_entity_type, related_entity_id, 
  status, assigned_to_admin_id, created_at, updated_at, source, dedupe_key, 
  first_seen_at, last_seen_at, occurrence_count, sla_due_at
) VALUES
  (
    'e1000000-0000-0000-0000-000000000001', 'critical', 'pci_dss_audit_due', 
    'PCI-DSS Annual AOC Expiry Warning', 
    'PCI-DSS AOC certification review window opens in 14 days. Evidence submission required.', 
    'compliance_control', '6668f411-07db-4d3f-bfc6-95e111efa6eb', 
    'unread', '9dc43202-47e4-4b59-981d-9d43b3568469', 
    NOW() - INTERVAL '3 hours', NOW(), 'compliance_engine', 'pci_dss_aoc_2026', 
    NOW() - INTERVAL '3 hours', NOW() - INTERVAL '30 minutes', 3, NOW() + INTERVAL '2 days'
  ),
  (
    'e1000000-0000-0000-0000-000000000002', 'warning', 'provider_latency_spike', 
    'HDFC PG Gateway Latency Spike Detected', 
    'P99 latency crossed 1250ms threshold during evening settlement cycle. Failover routing active.', 
    'provider', '10000000-0000-0000-0000-000000000001', 
    'in_progress', 'f7986939-ab81-4b70-a2d1-7253bbf7f0e1', 
    NOW() - INTERVAL '1 hour', NOW(), 'telemetry_slo', 'hdfc_p99_latency_surge', 
    NOW() - INTERVAL '1 hour', NOW() - INTERVAL '10 minutes', 5, NOW() + INTERVAL '4 hours'
  ),
  (
    'e1000000-0000-0000-0000-000000000003', 'warning', 'risk_review_escalated', 
    'High Velocity Alert on Merchant Aditya Retail', 
    'Transaction #c6a66666 exceeded single-session velocity threshold. Flagged for risk review.', 
    'payment_order', 'c6a66666-6666-4666-8666-666666666666', 
    'unread', '32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', 
    NOW() - INTERVAL '4 hours', NOW(), 'risk_engine', 'velocity_alert_c6a6', 
    NOW() - INTERVAL '4 hours', NOW() - INTERVAL '2 hours', 1, NOW() + INTERVAL '12 hours'
  ),
  (
    'e1000000-0000-0000-0000-000000000004', 'info', 'settlement_batch_completed', 
    'Daily T+1 Settlement Batch Executed Successfully', 
    'Settlement batch #b1000000 with 3 transfers totaling ₹41,20,000 processed without errors.', 
    'payout_batch', 'b1000000-0000-0000-0000-000000000001', 
    'resolved', '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', 
    NOW() - INTERVAL '6 hours', NOW(), 'settlement_worker', 'settlement_batch_b1', 
    NOW() - INTERVAL '6 hours', NOW() - INTERVAL '5 hours', 1, NOW() + INTERVAL '24 hours'
  ),
  (
    'e1000000-0000-0000-0000-000000000005', 'info', 'api_key_rotated', 
    'Production Webhook Secret Rotated', 
    'Webhook verification secret successfully rotated by Super Admin. Zero dropped webhooks.', 
    'admin_user', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'resolved', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    NOW() - INTERVAL '1 day', NOW(), 'security_audit', 'webhook_secret_rotation', 
    NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day', 1, NOW() + INTERVAL '48 hours'
  )
ON CONFLICT (id) DO NOTHING;

-- 4. compliance_evidence
INSERT INTO compliance_evidence (
  id, control_id, file_reference, file_name, uploaded_by_admin_id, note, created_at
) VALUES
  (
    'f1000000-0000-0000-0000-000000000001', 
    '6668f411-07db-4d3f-bfc6-95e111efa6eb', 
    's3://feelpay-compliance-vault/evidence/pci_dss_aoc_signed_2026.pdf', 
    'pci_dss_aoc_signed_2026.pdf', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'Signed Attestation of Compliance from Qualified Security Assessor (QSA)', 
    NOW() - INTERVAL '20 days'
  ),
  (
    'f1000000-0000-0000-0000-000000000002', 
    'b1939e7d-451c-4435-9f73-91052c92f175', 
    's3://feelpay-compliance-vault/evidence/zero_cvv_retention_audit.pdf', 
    'zero_cvv_retention_audit.pdf', 
    '9dc43202-47e4-4b59-981d-9d43b3568469', 
    'Automated database and log scanner verification confirming 0 CVV records', 
    NOW() - INTERVAL '15 days'
  ),
  (
    'f1000000-0000-0000-0000-000000000003', 
    'd4a93464-1fb1-4729-b707-ceb9d55c5d45', 
    's3://feelpay-compliance-vault/evidence/rbi_tokenization_audit_report.pdf', 
    'rbi_tokenization_audit_report.pdf', 
    '9dc43202-47e4-4b59-981d-9d43b3568469', 
    'RBI mandate card-on-file tokenization architecture and HSM vault verification', 
    NOW() - INTERVAL '10 days'
  ),
  (
    'f1000000-0000-0000-0000-000000000004', 
    '3df6b6a8-a992-42cc-b3c7-4919a3b998ea', 
    's3://feelpay-compliance-vault/evidence/aws_kms_envelope_encryption_attestation.pdf', 
    'aws_kms_envelope_encryption_attestation.pdf', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'AES-256-GCM envelope encryption key policy and annual rotation verification', 
    NOW() - INTERVAL '5 days'
  ),
  (
    'f1000000-0000-0000-0000-000000000005', 
    '41bb0a9e-f8bd-42d5-9acc-89fd3f9063ac', 
    's3://feelpay-compliance-vault/evidence/rbac_least_privilege_quarterly_review.pdf', 
    'rbac_least_privilege_quarterly_review.pdf', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'Q3 admin access review sign-off and role matrix validation', 
    NOW() - INTERVAL '2 days'
  )
ON CONFLICT (id) DO NOTHING;

-- 5. compliance_tasks
INSERT INTO compliance_tasks (
  id, title, category, status, owner_admin_id, due_at, completed_at, created_at
) VALUES
  (
    'f2000000-0000-0000-0000-000000000001', 
    'Annual PCI-DSS SAQ-D Assessment & Attestation', 
    'PCI', 'completed', '9dc43202-47e4-4b59-981d-9d43b3568469', 
    NOW() - INTERVAL '10 days', NOW() - INTERVAL '12 days', NOW() - INTERVAL '40 days'
  ),
  (
    'f2000000-0000-0000-0000-000000000002', 
    'External Network Vulnerability Assessment & Penetration Test', 
    'Security', 'completed', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    NOW() - INTERVAL '5 days', NOW() - INTERVAL '6 days', NOW() - INTERVAL '30 days'
  ),
  (
    'f2000000-0000-0000-0000-000000000003', 
    'Q3 Multi-Factor Authentication Enforcement Verification', 
    'Access', 'in_progress', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    NOW() + INTERVAL '7 days', NULL, NOW() - INTERVAL '14 days'
  ),
  (
    'f2000000-0000-0000-0000-000000000004', 
    'RBI Tokenization & Data Localization Audit', 
    'Regulatory', 'completed', '9dc43202-47e4-4b59-981d-9d43b3568469', 
    NOW() - INTERVAL '2 days', NOW() - INTERVAL '3 days', NOW() - INTERVAL '21 days'
  ),
  (
    'f2000000-0000-0000-0000-000000000005', 
    'Quarterly Access Control & Inactive User Pruning', 
    'Access', 'open', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    NOW() + INTERVAL '14 days', NULL, NOW() - INTERVAL '3 days'
  )
ON CONFLICT (id) DO NOTHING;

-- 6. compliance_packet_exports
INSERT INTO compliance_packet_exports (
  id, period, status, requested_by_admin_id, file_reference, created_at, expires_at
) VALUES
  (
    'f3000000-0000-0000-0000-000000000001', '2026-Q3', 'ready', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    's3://feelpay-compliance-packets/FeelPay_Compliance_Packet_2026_Q3.zip', 
    NOW() - INTERVAL '5 days', NOW() + INTERVAL '25 days'
  ),
  (
    'f3000000-0000-0000-0000-000000000002', '2026-Q2', 'completed', 
    '9dc43202-47e4-4b59-981d-9d43b3568469', 
    's3://feelpay-compliance-packets/FeelPay_Compliance_Packet_2026_Q2.zip', 
    NOW() - INTERVAL '95 days', NOW() - INTERVAL '5 days'
  ),
  (
    'f3000000-0000-0000-0000-000000000003', '2026-08-Monthly', 'ready', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    's3://feelpay-compliance-packets/FeelPay_Compliance_Packet_2026_08.zip', 
    NOW() - INTERVAL '12 days', NOW() + INTERVAL '18 days'
  )
ON CONFLICT (id) DO NOTHING;

-- 7. access_reviews & access_review_items
INSERT INTO access_reviews (
  id, review_period, status, started_by_admin_id, completed_by_admin_id, 
  started_at, completed_at, due_at, scope, reviewer_admin_id
) VALUES
  (
    'a1000000-0000-0000-0000-000000000001', '2026-Q3 Quarterly Access Review', 'completed', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    NOW() - INTERVAL '15 days', NOW() - INTERVAL '2 days', NOW() - INTERVAL '1 day', 
    'All Internal Admin, Finance, and Risk Operational Roles', 'cd317088-a9ef-495c-ab3e-2a88603c54ac'
  ),
  (
    'a1000000-0000-0000-0000-000000000002', '2026-Q4 Upcoming Privilege Review', 'open', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NULL, 
    NOW() - INTERVAL '1 day', NULL, NOW() + INTERVAL '30 days', 
    'High Privilege Production Access & API Key Scopes', 'cd317088-a9ef-495c-ab3e-2a88603c54ac'
  )
ON CONFLICT (id) DO NOTHING;

INSERT INTO access_review_items (
  id, review_id, admin_user_id, role, decision, note, 
  reviewed_by_admin_id, reviewed_at, recommended_action, risk_reason
) VALUES
  (
    'a2000000-0000-0000-0000-000000000001', 'a1000000-0000-0000-0000-000000000001', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 'Super Admin', 'retained', 
    'Root platform operations administrator. MFA verified via hardware security key.', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '3 days', 'retain_access', 'none'
  ),
  (
    'a2000000-0000-0000-0000-000000000002', 'a1000000-0000-0000-0000-000000000001', 
    'f7986939-ab81-4b70-a2d1-7253bbf7f0e1', 'Payment Ops Admin', 'retained', 
    'Active on provider routing and webhook management. Least privilege verified.', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '3 days', 'retain_access', 'none'
  ),
  (
    'a2000000-0000-0000-0000-000000000003', 'a1000000-0000-0000-0000-000000000001', 
    '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', 'Finance Admin', 'retained', 
    'Reconciliation and settlement authority verified. Active weekly usage.', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '2 days', 'retain_access', 'none'
  ),
  (
    'a2000000-0000-0000-0000-000000000004', 'a1000000-0000-0000-0000-000000000001', 
    '32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', 'Risk Admin', 'retained', 
    'Risk rules and dispute queue manager. MFA compliant.', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '2 days', 'retain_access', 'none'
  ),
  (
    'a2000000-0000-0000-0000-000000000005', 'a1000000-0000-0000-0000-000000000001', 
    '9dc43202-47e4-4b59-981d-9d43b3568469', 'Compliance Admin', 'retained', 
    'Audit and evidence overseer. Role appropriate for current responsibilities.', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '2 days', 'retain_access', 'none'
  )
ON CONFLICT (id) DO NOTHING;

-- 8. security_events
INSERT INTO security_events (
  id, event_type, severity, actor_admin_id, entity_type, entity_id, 
  ip_address, user_agent, summary, created_at
) VALUES
  (
    'a1000000-0000-0000-0000-000000000001', 'admin_login_success', 'info', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 'admin_user', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    '103.21.244.15', 'Mozilla/5.0 (X11; Linux x86_64) Chrome/131.0.0.0', 
    '{"action": "admin_session_created", "mfa_method": "totp", "result": "success"}'::jsonb, 
    NOW() - INTERVAL '12 minutes'
  ),
  (
    'a1000000-0000-0000-0000-000000000002', 'credentials_rotated', 'low', 
    'f7986939-ab81-4b70-a2d1-7253bbf7f0e1', 'provider_credential', '10000000-0000-0000-0000-000000000001', 
    '103.21.244.22', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/131.0.0.0', 
    '{"provider": "hdfc", "credential_type": "api_secret", "rotation_policy": "quarterly"}'::jsonb, 
    NOW() - INTERVAL '4 hours'
  ),
  (
    'a1000000-0000-0000-0000-000000000003', 'role_assigned', 'medium', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', 'admin_role', 'ecd75c67-42f6-4d68-bd44-010c2eb65695', 
    '103.21.244.15', 'Mozilla/5.0 (X11; Linux x86_64) Chrome/131.0.0.0', 
    '{"assigned_user": "kiran.rao@feelpay.com", "role": "Compliance Admin"}'::jsonb, 
    NOW() - INTERVAL '20 days'
  ),
  (
    'a1000000-0000-0000-0000-000000000004', 'rate_limit_triggered', 'medium', 
    NULL, 'api_endpoint', NULL, 
    '198.51.100.74', 'python-requests/2.31.0', 
    '{"endpoint": "/v1/charges", "limit": "100/min", "count": 142, "action": "throttled"}'::jsonb, 
    NOW() - INTERVAL '8 hours'
  ),
  (
    'a1000000-0000-0000-0000-000000000005', 'export_masking_applied', 'info', 
    '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', 'export_job', NULL, 
    '103.21.244.40', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)', 
    '{"export_type": "settlements", "masked_fields": ["account_number", "card_bin"]}'::jsonb, 
    NOW() - INTERVAL '1 day'
  )
ON CONFLICT (id) DO NOTHING;

-- 9. payout_batches_company & payout_batch_items_company
INSERT INTO payout_batches_company (
  id, currency, total_amount, item_count, status, created_by_admin_id, 
  created_at, processed_at
) VALUES
  (
    'b1000000-0000-0000-0000-000000000001', 'INR', 4120000, 3, 'paid', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() - INTERVAL '6 hours', NOW() - INTERVAL '5 hours 45 minutes'
  ),
  (
    'b1000000-0000-0000-0000-000000000002', 'INR', 1850000, 2, 'processing', 
    '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', NOW() - INTERVAL '1 hour', NULL
  ),
  (
    'b1000000-0000-0000-0000-000000000003', 'INR', 980000, 1, 'queued', 
    '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', NOW() - INTERVAL '15 minutes', NULL
  )
ON CONFLICT (id) DO NOTHING;

INSERT INTO payout_batch_items_company (
  id, batch_id, payout_id, business_account_id, amount, status
) VALUES
  (
    'b2000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000001', 
    '50000000-0000-0000-0000-000000000001', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    2500000, 'paid'
  ),
  (
    'b2000000-0000-0000-0000-000000000002', 'b1000000-0000-0000-0000-000000000001', 
    '50000000-0000-0000-0000-000000000002', '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', 
    1500000, 'paid'
  ),
  (
    'b2000000-0000-0000-0000-000000000003', 'b1000000-0000-0000-0000-000000000001', 
    'a0000000-0000-0000-0000-000000000104', '32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', 
    120000, 'paid'
  )
ON CONFLICT (id) DO NOTHING;

-- 10. admin_scheduled_reports
INSERT INTO admin_scheduled_reports (
  id, report_type, filters_json, frequency, recipients, format, status, 
  created_by_admin_id, next_run_at, last_run_at, created_at
) VALUES
  (
    'a2000000-0000-0000-0000-000000000001', 'daily_transaction_summary', 
    '{"status": "captured", "currency": "INR"}'::jsonb, 'daily', 
    '["finance-ops@feelpay.com", "aditya.sharma@feelpay.com"]'::jsonb, 'CSV', 'active', 
    'cd317088-a9ef-495c-ab3e-2a88603c54ac', NOW() + INTERVAL '14 hours', NOW() - INTERVAL '10 hours', NOW() - INTERVAL '30 days'
  ),
  (
    'a2000000-0000-0000-0000-000000000002', 'reconciliation_discrepancy_digest', 
    '{"min_discrepancy_amount": 1000}'::jsonb, 'weekly', 
    '["reconciliation-team@feelpay.com", "vanya.iyer@feelpay.com"]'::jsonb, 'XLSX', 'active', 
    '4717ffc1-d4b9-4df6-8f32-2c70ab3b4aa4', NOW() + INTERVAL '3 days', NOW() - INTERVAL '4 days', NOW() - INTERVAL '25 days'
  ),
  (
    'a2000000-0000-0000-0000-000000000003', 'pci_compliance_security_audit', 
    '{"category": "PCI", "include_evidence": true}'::jsonb, 'monthly', 
    '["compliance@feelpay.com", "kiran.rao@feelpay.com"]'::jsonb, 'PDF', 'active', 
    '9dc43202-47e4-4b59-981d-9d43b3568469', NOW() + INTERVAL '18 days', NOW() - INTERVAL '12 days', NOW() - INTERVAL '60 days'
  ),
  (
    'a2000000-0000-0000-0000-000000000004', 'provider_latency_sla_performance', 
    '{"window": "7d", "providers": ["razorpay", "hdfc", "cashfree"]}'::jsonb, 'weekly', 
    '["tech-ops@feelpay.com", "harsh.patel@feelpay.com"]'::jsonb, 'PDF', 'active', 
    'f7986939-ab81-4b70-a2d1-7253bbf7f0e1', NOW() + INTERVAL '5 days', NOW() - INTERVAL '2 days', NOW() - INTERVAL '15 days'
  )
ON CONFLICT (id) DO NOTHING;

-- 11. metric_samples
INSERT INTO metric_samples (id, metric_name, labels_json, value, captured_at)
VALUES
  ('a3000000-0000-0000-0000-000000000001', 'payment.success_rate', '{"provider": "razorpay", "channel": "upi"}'::jsonb, 99.45, NOW() - INTERVAL '5 minutes'),
  ('a3000000-0000-0000-0000-000000000002', 'payment.success_rate', '{"provider": "hdfc", "channel": "cards"}'::jsonb, 98.70, NOW() - INTERVAL '5 minutes'),
  ('a3000000-0000-0000-0000-000000000003', 'api.latency_p99', '{"service": "checkout_gateway"}'::jsonb, 142.50, NOW() - INTERVAL '5 minutes'),
  ('a3000000-0000-0000-0000-000000000004', 'api.latency_p99', '{"service": "admin_server"}'::jsonb, 68.20, NOW() - INTERVAL '5 minutes'),
  ('a3000000-0000-0000-0000-000000000005', 'webhook.delivery_success_rate', '{"topic": "payment.captured"}'::jsonb, 99.85, NOW() - INTERVAL '5 minutes'),
  ('a3000000-0000-0000-0000-000000000006', 'database.query_duration_ms', '{"pool": "neon_pooler"}'::jsonb, 14.80, NOW() - INTERVAL '5 minutes'),
  ('a3000000-0000-0000-0000-000000000007', 'payment.success_rate', '{"provider": "razorpay", "channel": "upi"}'::jsonb, 99.30, NOW() - INTERVAL '30 minutes'),
  ('a3000000-0000-0000-0000-000000000008', 'payment.success_rate', '{"provider": "hdfc", "channel": "cards"}'::jsonb, 98.60, NOW() - INTERVAL '30 minutes'),
  ('a3000000-0000-0000-0000-000000000009', 'api.latency_p99', '{"service": "checkout_gateway"}'::jsonb, 150.10, NOW() - INTERVAL '30 minutes'),
  ('a3000000-0000-0000-0000-000000000010', 'webhook.delivery_success_rate', '{"topic": "refund.processed"}'::jsonb, 100.0, NOW() - INTERVAL '30 minutes')
ON CONFLICT (id) DO NOTHING;

-- 12. risk_reviews, risk_signals, risk_decisions
INSERT INTO risk_reviews (
  id, payment_order_id, risk_rule_id, status, risk_level, notes, 
  created_at, updated_at, assigned_to_admin_id, decision_note, resolved_at
) VALUES
  (
    'a4000000-0000-0000-0000-000000000001', 
    'c6a66666-6666-4666-8666-666666666666', 
    '10000000-0000-0000-0000-000000000001', 
    'resolved', 'medium', 'Transaction amount ₹899 triggered velocity pattern check. Merchant profile clean.', 
    NOW() - INTERVAL '4 hours', NOW() - INTERVAL '1 hour', 
    '32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', 'Approved after customer device fingerprint match verified.', NOW() - INTERVAL '1 hour'
  ),
  (
    'a4000000-0000-0000-0000-000000000002', 
    'c4a44444-4444-4444-8444-444444444444', 
    '10000000-0000-0000-0000-000000000005', 
    'in_review', 'high', 'Refund request ₹1,500 submitted within 2 minutes of payment confirmation.', 
    NOW() - INTERVAL '2 hours', NOW() - INTERVAL '30 minutes', 
    '32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', NULL, NULL
  ),
  (
    'a4000000-0000-0000-0000-000000000003', 
    'c5a55555-5555-4555-8555-555555555555', 
    '10000000-0000-0000-0000-000000000003', 
    'pending', 'low', 'Standard review queue item for high-ticket merchant daily cap.', 
    NOW() - INTERVAL '1 hour', NOW() - INTERVAL '1 hour', 
    '32eb3555-6ec4-4112-b2e4-03fe1dcb42d9', NULL, NULL
  )
ON CONFLICT (id) DO NOTHING;

INSERT INTO risk_signals (
  id, payment_id, business_account_id, signal_type, signal_value, severity, 
  created_at, rule_id, scope, entity_type, entity_id, signal_payload_redacted
) VALUES
  (
    'a5000000-0000-0000-0000-000000000001', 
    'c6a66666-6666-4666-8666-666666666666', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'ip_velocity_normal', '3 transactions / 24h', 'low', 
    NOW() - INTERVAL '4 hours', '10000000-0000-0000-0000-000000000001', 
    'merchant', 'payment_order', 'c6a66666-6666-4666-8666-666666666666', 
    '{"asn": "AS45609", "country": "IN", "vpn_detected": false}'::jsonb
  ),
  (
    'a5000000-0000-0000-0000-000000000002', 
    'c4a44444-4444-4444-8444-444444444444', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'rapid_refund_request', 'Delta 118s after capture', 'high', 
    NOW() - INTERVAL '2 hours', '10000000-0000-0000-0000-000000000005', 
    'merchant', 'payment_order', 'c4a44444-4444-4444-8444-444444444444', 
    '{"amount": 150000, "refund_reason": "customer_cancellation"}'::jsonb
  ),
  (
    'a5000000-0000-0000-0000-000000000003', 
    'c5a55555-5555-4555-8555-555555555555', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'device_fingerprint_verified', 'Trusted Android 14 Client', 'info', 
    NOW() - INTERVAL '1 hour', '10000000-0000-0000-0000-000000000003', 
    'customer', 'payment_order', 'c5a55555-5555-4555-8555-555555555555', 
    '{"browser": "Chrome Mobile", "hardware_concurrency": 8, "screen": "1080x2400"}'::jsonb
  )
ON CONFLICT (id) DO NOTHING;

INSERT INTO risk_decisions (
  id, payment_id, business_account_id, decision, score, matched_rules, reason, created_at
) VALUES
  (
    'a6000000-0000-0000-0000-000000000001', 
    'c6a66666-6666-4666-8666-666666666666', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'allow', 12, '["High Amount Velocity"]'::jsonb, 
    'Velocity score 12 is within normal bounds for tier-1 verified merchant.', 
    NOW() - INTERVAL '4 hours'
  ),
  (
    'a6000000-0000-0000-0000-000000000002', 
    'c4a44444-4444-4444-8444-444444444444', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'review', 74, '["High Refund Rate Alert"]'::jsonb, 
    'Rapid refund flagged for manual operational sign-off.', 
    NOW() - INTERVAL '2 hours'
  ),
  (
    'a6000000-0000-0000-0000-000000000003', 
    'c5a55555-5555-4555-8555-555555555555', 'cd317088-a9ef-495c-ab3e-2a88603c54ac', 
    'allow', 8, '[]'::jsonb, 
    'Customer passed 3DS authentication with low biometric velocity anomaly.', 
    NOW() - INTERVAL '1 hour'
  )
ON CONFLICT (id) DO NOTHING;

-- 13. Update platform_settings support_phone if empty
UPDATE platform_settings 
SET value = '+91 80 4567 8900', updated_at = NOW() 
WHERE key = 'support_phone';

COMMIT;
