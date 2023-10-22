-- Add migration script here

INSERT INTO users (user_id, username, password_hash)
VALUES (
    '3389aee0-1427-41cc-975d-c094e93c055d',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$/UBnXtgoYBiaUPthc9RtVA$Fw6pOwda4N2XH5M8ib/1QFgIAU8yZ+yp9AMycAk4vBM'
);