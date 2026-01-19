-- ========================================================================
-- SEED: Usuarios del sistema para pruebas
-- ========================================================================
-- Contraseñas hasheadas con Argon2id (m=65536, t=3, p=4)

-- Usuario superadmin (Password: admin123)
INSERT INTO users (username, email, password_hash, role, status)
VALUES (
    'superadmin',
    'superadmin@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$/JWssXfsvez6ReA2Ptt7mA$772MdE2wg2ccdUG+n7306wcg0Gd/vifsma2JGvlBB4o',
    'superadmin',
    'activo'
);

-- Usuario admin (Password: admin123)
INSERT INTO users (username, email, password_hash, role, status)
VALUES (
    'admin',
    'admin@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$BWDMiawBsE3COODfedJWAw$OjKLgMtzuypNQMdRMI0bDRDozJ0FKkDeTxMKb5Ga4ro',
    'admin',
    'activo'
);

-- Usuario agencia (Password: usuario123)
INSERT INTO users (username, email, password_hash, role, status)
VALUES (
    'agencia',
    'agencia@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$pQn2mpRNvRiTmWlHmZm8Ew$rZIfnmWL3a21uy1IPdZilP2zYN7PjYjfoawPpEFUuTM',
    'agencia',
    'activo'
);

-- Usuario restaurante (Password: usuario123)
INSERT INTO users (username, email, password_hash, role, status)
VALUES (
    'restaurante',
    'restaurante@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$CNf+FSBuQz7G5Qv1dQ36pg$UH7wWB4MHgN6OApyVC6awZM6TQLpRTZg4L+Y59bYIxE',
    'restaurante',
    'activo'
);

-- ========================================================================
-- Credenciales de prueba:
-- superadmin/admin: admin123
-- restaurante/agencia: usuario123
-- ========================================================================
