-- Agregar campo boleto_turistico a entradas para distinguir entradas de tipo BT
ALTER TABLE entradas ADD COLUMN boleto_turistico BOOLEAN NOT NULL DEFAULT false;
CREATE INDEX idx_entradas_boleto_turistico ON entradas (boleto_turistico);
