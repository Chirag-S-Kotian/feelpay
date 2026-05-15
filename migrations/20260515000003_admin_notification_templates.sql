ALTER TABLE notification_templates
    ADD COLUMN IF NOT EXISTS environment VARCHAR(10) NOT NULL DEFAULT 'test',
    ADD COLUMN IF NOT EXISTS sms_body TEXT,
    ADD COLUMN IF NOT EXISTS default_email_provider_id UUID REFERENCES integration_providers(id),
    ADD COLUMN IF NOT EXISTS default_sms_provider_id UUID REFERENCES integration_providers(id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_notification_templates_event_channel_environment
    ON notification_templates(event_type, channel, environment);
