ALTER TABLE servers ADD CONSTRAINT valid_port CHECK ( port >= 1 AND port <= 65535 );
