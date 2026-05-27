ALTER TABLE audit_log DROP CONSTRAINT audit_log_client_id_fkey;
ALTER TABLE audit_log ADD CONSTRAINT audit_log_client_id_fkey
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE CASCADE;

ALTER TABLE audit_log DROP CONSTRAINT audit_log_actor_key_id_fkey;
ALTER TABLE audit_log ADD CONSTRAINT audit_log_actor_key_id_fkey
    FOREIGN KEY (actor_key_id) REFERENCES api_keys(id) ON DELETE CASCADE;
