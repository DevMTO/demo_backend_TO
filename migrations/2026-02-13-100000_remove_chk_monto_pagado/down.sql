-- Re-add the constraint (might fail if new data violates it, which is expected)
ALTER TABLE pagos_files ADD CONSTRAINT chk_monto_pagado CHECK (monto_pagado >= 0 AND monto_pagado <= monto_total);
