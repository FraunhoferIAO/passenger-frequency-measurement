CREATE DATABASE paxcounter;
\c paxcounter;

SET TIME ZONE "Europe/Berlin";

CREATE TABLE ttn_payloads ( 
    id VARCHAR PRIMARY KEY, 
    device_id VARCHAR, 
    dev_addr VARCHAR, 
    payload VARCHAR, 
    received_at TIMESTAMPTZ 
); 

CREATE MATERIALIZED VIEW ttn_payloads_decoded 
AS SELECT 
    id, 
    device_id, 
    dev_addr, 
    (ttn_payload_decoder(payload)).f1 as wifi, 
    (ttn_payload_decoder(payload)).f2 as bluetooth, 
    received_at 
FROM ttn_payloads; 

CREATE USER connector_rw WITH PASSWORD 'connector_rw';
GRANT CONNECT ON DATABASE paxcounter TO connector_rw;
GRANT INSERT ON ttn_payloads TO connector_rw; 

CREATE USER dashboard_ro WITH PASSWORD 'dashboard_ro';
GRANT CONNECT ON DATABASE paxcounter TO dashboard_ro;
GRANT SELECT ON ttn_payloads_decoded TO dashboard_ro; 