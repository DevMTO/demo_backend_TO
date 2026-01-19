-- Eliminar usuarios de prueba
DELETE FROM users WHERE username IN ('superadmin', 'admin', 'restaurante', 'agencia');

-- Reiniciar la secuencia de IDs
ALTER SEQUENCE users_id_seq RESTART WITH 1;
