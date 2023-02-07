INSERT INTO email_callbacks (document_id, address)
  SELECT wd.document_id, ea.id
    FROM witnessed_documents wd
    LEFT JOIN documents d ON wd.document_id = d.id
    LEFT JOIN email_addresses ea ON d.person_id = ea.person_id
    WHERE wd.document_id NOT IN (SELECT document_id FROM email_callbacks WHERE NOT cc);
