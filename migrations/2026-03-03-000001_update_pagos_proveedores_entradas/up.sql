-- ========================================================================
-- Actualizar pagos_proveedores: renombrar monto, agregar entradas y monto_pagado
-- ========================================================================

-- Renombrar monto a monto_total
ALTER TABLE pagos_proveedores RENAME COLUMN monto TO monto_total;

-- Agregar nuevas columnas
ALTER TABLE pagos_proveedores ADD COLUMN id_entrada INTEGER REFERENCES entradas(id) ON DELETE SET NULL;
ALTER TABLE pagos_proveedores ADD COLUMN id_file_entrada INTEGER REFERENCES file_entradas(id) ON DELETE SET NULL;
ALTER TABLE pagos_proveedores ADD COLUMN monto_pagado DECIMAL(15,2);

-- Actualizar constraint tipo_proveedor para incluir 'entrada'
ALTER TABLE pagos_proveedores DROP CONSTRAINT chk_tipo_proveedor;
ALTER TABLE pagos_proveedores ADD CONSTRAINT chk_tipo_proveedor
  CHECK (tipo_proveedor IN ('transporte', 'restaurante', 'guia', 'entrada'));

-- Actualizar constraint entidad para incluir entrada
ALTER TABLE pagos_proveedores DROP CONSTRAINT chk_proveedor_entidad;
ALTER TABLE pagos_proveedores ADD CONSTRAINT chk_proveedor_entidad CHECK (
    (tipo_proveedor = 'transporte' AND id_transporte IS NOT NULL) OR
    (tipo_proveedor = 'restaurante' AND id_restaurante IS NOT NULL) OR
    (tipo_proveedor = 'guia' AND id_guia IS NOT NULL) OR
    (tipo_proveedor = 'entrada' AND id_entrada IS NOT NULL)
);

-- Indices
CREATE INDEX idx_pagos_proveedores_entrada ON pagos_proveedores(id_entrada);
CREATE INDEX idx_pagos_proveedores_file_entrada ON pagos_proveedores(id_file_entrada);
