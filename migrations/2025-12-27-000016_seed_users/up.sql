-- Contraseñas hasheadas con Argon2id (m=65536, t=3, p=4)

-- Usuario superadmin (Password: admin123)
INSERT INTO users (username, email, password_hash, role)
VALUES (
    'superadmin',
    'superadmin@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$/JWssXfsvez6ReA2Ptt7mA$772MdE2wg2ccdUG+n7306wcg0Gd/vifsma2JGvlBB4o',
    'superadmin'
);

-- Usuario admin (Password: admin123)
INSERT INTO users (username, email, password_hash, role)
VALUES (
    'admin',
    'admin@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$BWDMiawBsE3COODfedJWAw$OjKLgMtzuypNQMdRMI0bDRDozJ0FKkDeTxMKb5Ga4ro',
    'admin'
);

-- Credenciales de prueba:
-- superadmin/admin: admin123