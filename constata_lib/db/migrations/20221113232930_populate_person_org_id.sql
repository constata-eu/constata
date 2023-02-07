UPDATE persons set org_id = id;
ALTER TABLE persons ALTER COLUMN org_id SET NOT NULL;

