-- Notifications Table
-- Notificaciones del sistema

CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    
    -- Tipo de notificación
    notification_type VARCHAR(50) NOT NULL, -- info, warning, error, success, system
    
    -- Categoría de la notificación
    category VARCHAR(50) NOT NULL, -- auth, crud, system, alert
    
    -- Título y contenido
    title VARCHAR(200) NOT NULL,
    message TEXT NOT NULL,
    
    -- Referencia a entidad relacionada (opcional)
    entity_type VARCHAR(50),
    entity_id INTEGER,
    
    -- Datos adicionales en JSON
    metadata JSONB,
    
    -- Prioridad de la notificación
    priority VARCHAR(20) NOT NULL DEFAULT 'normal', -- low, normal, high, urgent
    
    -- Roles objetivo (si es null, es para todos)
    target_roles JSONB, -- ["superadmin", "admin"]
    
    -- Usuario específico objetivo (si aplica)
    target_user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    
    -- Fecha de expiración (opcional)
    expires_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- User Notifications Table
-- Relación entre usuarios y notificaciones (para tracking de lectura)

CREATE TABLE notification_users (
    id SERIAL PRIMARY KEY,
    notification_id INTEGER NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Estado de lectura
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    
    -- Si fue descartada
    is_dismissed BOOLEAN NOT NULL DEFAULT FALSE,
    dismissed_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Evitar duplicados
    UNIQUE(notification_id, user_id)
);

-- Índices para búsquedas comunes
CREATE INDEX idx_notifications_type ON notifications(notification_type);
CREATE INDEX idx_notifications_category ON notifications(category);
CREATE INDEX idx_notifications_priority ON notifications(priority);
CREATE INDEX idx_notifications_target_user ON notifications(target_user_id);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);
CREATE INDEX idx_notifications_expires_at ON notifications(expires_at);
CREATE INDEX idx_notifications_target_roles ON notifications USING gin(target_roles jsonb_path_ops);

CREATE INDEX idx_notification_users_user ON notification_users(user_id);
CREATE INDEX idx_notification_users_notification ON notification_users(notification_id);
CREATE INDEX idx_notification_users_unread ON notification_users(user_id, is_read) WHERE is_read = FALSE;
