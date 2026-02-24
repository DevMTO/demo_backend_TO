-- Agregar columna cancelaciones a file_entradas para historial de transferencias BT
ALTER TABLE file_entradas ADD COLUMN cancelaciones INTEGER[] NOT NULL DEFAULT '{}';
