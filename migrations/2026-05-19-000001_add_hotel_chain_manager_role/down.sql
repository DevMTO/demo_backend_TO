-- Revierte el rename del rol gerente de cadena.
-- ATENCIÓN: si se crearon usuarios `hoteles_gerente` (hotel-level) después
-- del up, este down los colapsa con los cadena-managers — no hay forma
-- determinista de distinguirlos sin metadata extra.

UPDATE users
SET role = 'hoteles_gerente'
WHERE role = 'hoteles_gerente_cadena';

ALTER TABLE users
    ALTER COLUMN role TYPE VARCHAR(20);
