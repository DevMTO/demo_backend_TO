ALTER TABLE pagos_proveedores ADD CONSTRAINT chk_pago_proveedor_estado CHECK (estado IN ('pendiente', 'pagado'));
