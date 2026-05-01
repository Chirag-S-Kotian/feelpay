-- Seed system roles and permissions
INSERT INTO roles (name, description, is_system) 
VALUES ('merchant_owner', 'Full access to merchant features', true)
ON CONFLICT DO NOTHING;

INSERT INTO roles (name, description, is_system) 
VALUES ('merchant_admin', 'Administrative access to merchant features', true)
ON CONFLICT DO NOTHING;

INSERT INTO roles (name, description, is_system) 
VALUES ('merchant_staff', 'Operational access for merchant team members', true)
ON CONFLICT DO NOTHING;

INSERT INTO roles (name, description, is_system) 
VALUES ('merchant_viewer', 'Read-only access for merchant team members', true)
ON CONFLICT DO NOTHING;

INSERT INTO roles (name, description, is_system) 
VALUES ('merchant_developer', 'Developer access to API keys and logs', true)
ON CONFLICT DO NOTHING;
