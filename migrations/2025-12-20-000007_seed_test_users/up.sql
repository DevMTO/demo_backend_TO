-- Migration: Seed users for testing
-- Usuarios de prueba con contraseñas hasheadas con Argon2id
-- Parámetros: m=65536, t=3, p=4 (según .env)

-- Insertar usuario administrador
-- Password: admin123 (hasheado con Argon2id)
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    display_name,
    role,
    email_verified,
    is_active,
    mfa_enabled
) VALUES (
    gen_random_uuid(),
    'admin',
    'admin@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$/JWssXfsvez6ReA2Ptt7mA$772MdE2wg2ccdUG+n7306wcg0Gd/vifsma2JGvlBB4o',
    'Administrador del Sistema',
    'superadmin',
    true,
    true,
    false
);

-- Insertar usuario subadmin
-- Password: admin123 (hasheado con Argon2id)
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    display_name,
    role,
    email_verified,
    is_active,
    mfa_enabled
) VALUES (
    gen_random_uuid(),
    'subadmin',
    'subadmin@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$BWDMiawBsE3COODfedJWAw$OjKLgMtzuypNQMdRMI0bDRDozJ0FKkDeTxMKb5Ga4ro',
    'Sub Administrador',
    'subadmin',
    true,
    true,
    false
);

-- Insertar usuario normal de prueba
-- Password: usuario123 (hasheado con Argon2id)
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    display_name,
    role,
    email_verified,
    is_active,
    mfa_enabled
) VALUES (
    gen_random_uuid(),
    'usuario',
    'usuario@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$pQn2mpRNvRiTmWlHmZm8Ew$rZIfnmWL3a21uy1IPdZilP2zYN7PjYjfoawPpEFUuTM',
    'Usuario de Prueba',
    'user',
    true,
    true,
    false
);

-- Insertar usuario viewer de prueba
-- Password: usuario123 (hasheado con Argon2id)
INSERT INTO users (
    id,
    username,
    email,
    password_hash,
    display_name,
    role,
    email_verified,
    is_active,
    mfa_enabled
) VALUES (
    gen_random_uuid(),
    'viewer',
    'viewer@sistema.local',
    '$argon2id$v=19$m=65536,t=3,p=4$CNf+FSBuQz7G5Qv1dQ36pg$UH7wWB4MHgN6OApyVC6awZM6TQLpRTZg4L+Y59bYIxE',
    'Viewer de Prueba',
    'viewer',
    false,
    true,
    false
);

-- Credenciales de prueba:
-- admin/subadmin: admin123
-- usuario/viewer: usuario123
