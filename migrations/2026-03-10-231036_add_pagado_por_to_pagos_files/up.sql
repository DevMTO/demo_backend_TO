-- Agrega campos de auditoría de pago y modificación
ALTER TABLE pagos_files
    ADD COLUMN pagado_por  INTEGER REFERENCES users(id),
    ADD COLUMN pagado_at   TIMESTAMPTZ,
    ADD COLUMN updated_by  INTEGER REFERENCES users(id);

