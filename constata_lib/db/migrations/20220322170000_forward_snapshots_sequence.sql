SELECT setval('story_snapshots_id_seq', (select greatest(count(*), 1000) from story_snapshots) , true);

