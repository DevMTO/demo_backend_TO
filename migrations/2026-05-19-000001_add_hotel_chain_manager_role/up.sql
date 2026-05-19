-- Renombrar el rol `hoteles_gerente` (que actualmente significa "gerente de cadena")
-- a `hoteles_gerente_cadena`. El nombre `hoteles_gerente` queda libre para representar
-- al nuevo gerente de un hotel concreto.

ALTER TABLE users
    ALTER COLUMN role TYPE VARCHAR(30);

UPDATE users
SET role = 'hoteles_gerente_cadena'
WHERE role = 'hoteles_gerente';
