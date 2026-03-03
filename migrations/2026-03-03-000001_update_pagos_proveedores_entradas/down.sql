-- Revertir cambios en pagos_proveedores

-- Eliminar indices
DROP INDEX IF EXISTS idx_pagos_proveedores_entrada;
DROP INDEX IF EXISTS idx_pagos_proveedores_file_entrada;

-- Revertir constraints
ALTER TABLE pagos_proveedores DROP CONSTRAINT chk_proveedor_entidad;
ALTER TABLE pagos_proveedores ADD CONSTRAINT chk_proveedor_entidad CHECK (
    (tipo_proveedor = 'transporte' AND id_transporte IS NOT NULL) OR
    (tipo_proveedor = 'restaurante' AND id_restaurante IS NOT NULL) OR
    (tipo_proveedor = 'guia' AND id_guia IS NOT NULL)
);

ALTER TABLE pagos_proveedores DROP CONSTRAINT chk_tipo_proveedor;
ALTER TABLE pagos_proveedores ADD CONSTRAINT chk_tipo_proveedor
  CHECK (tipo_proveedor IN ('transporte', 'restaurante', 'guia'));

-- Eliminar columnas nuevas
ALTER TABLE pagos_proveedores DROP COLUMN IF EXISTS monto_pagado;
ALTER TABLE pagos_proveedores DROP COLUMN IF EXISTS id_file_entrada;
ALTER TABLE pagos_proveedores DROP COLUMN IF EXISTS id_entrada;

-- Revertir rename
ALTER TABLE pagos_proveedores RENAME COLUMN monto_total TO monto;
