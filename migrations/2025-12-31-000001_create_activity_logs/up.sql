-- Activity Logs Table
-- Registra todas las acciones realizadas en el sistema

CREATE TABLE activity_logs (
    id SERIAL PRIMARY KEY,
    -- Usuario que realizó la acción
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    username VARCHAR(50),
    
    -- Tipo de acción (AUTH, CRUD, SYSTEM)
    action_type VARCHAR(30) NOT NULL,
    -- Acción específica (login, logout, create, update, delete, etc.)
    action VARCHAR(50) NOT NULL,
    
    -- Entidad afectada (users, agencias, guias, tours, etc.)
    entity_type VARCHAR(50) NOT NULL,
    -- ID del registro afectado
    entity_id INTEGER,
    
    -- Descripción legible de la acción
    description TEXT,
    
    -- Datos antes del cambio (para updates)
    old_values JSONB,
    -- Datos después del cambio (para creates/updates)
    new_values JSONB,
    -- Campos específicos que cambiaron
    changed_fields JSONB,
    
    -- Metadatos de la petición
    ip_address VARCHAR(45),
    user_agent TEXT,
    
    -- Resultado de la acción
    status VARCHAR(20) NOT NULL DEFAULT 'success', -- success, error, warning
    error_message TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Índices para búsquedas comunes
CREATE INDEX idx_activity_logs_user_id ON activity_logs(user_id);
CREATE INDEX idx_activity_logs_action_type ON activity_logs(action_type);
CREATE INDEX idx_activity_logs_entity ON activity_logs(entity_type, entity_id);
CREATE INDEX idx_activity_logs_created_at ON activity_logs(created_at);
CREATE INDEX idx_activity_logs_status ON activity_logs(status);

-- Índice GIN para búsquedas en JSONB
CREATE INDEX idx_activity_logs_new_values ON activity_logs USING gin(new_values jsonb_path_ops);
CREATE INDEX idx_activity_logs_changed_fields ON activity_logs USING gin(changed_fields jsonb_path_ops);
