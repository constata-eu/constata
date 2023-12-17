DROP TABLE vc_requirements;
DROP TABLE vc_prompts;
DROP TABLE vc_requests;
DROP TYPE vc_request_state; 
ALTER TABLE orgs DROP COLUMN use_verifier;
DELETE FROM access_tokens WHERE kind = 'vc_request';

