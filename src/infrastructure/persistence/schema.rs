// @generated automatically by Diesel CLI.

diesel::table! {
    agencias (id) {
        id -> Uuid,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 11]
        ruc -> Varchar,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        direccion -> Nullable<Text>,
        paleta_colores -> Nullable<Jsonb>,
        media -> Nullable<Jsonb>,
        encargado -> Nullable<Uuid>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    conductores (id) {
        id -> Uuid,
        id_persona -> Uuid,
        id_transporte -> Nullable<Uuid>,
        #[max_length = 20]
        nro_brevete -> Varchar,
        tiene_soat -> Bool,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    entradas (id) {
        id -> Uuid,
        #[max_length = 200]
        nombre -> Varchar,
        precio -> Numeric,
        #[max_length = 200]
        ruta -> Nullable<Varchar>,
        #[max_length = 50]
        tipo -> Varchar,
        descripcion -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        #[max_length = 20]
        file_code -> Varchar,
        id_tour -> Uuid,
        id_agencia -> Uuid,
        guias -> Nullable<Jsonb>,
        pasajeros -> Nullable<Jsonb>,
        vehiculos -> Nullable<Jsonb>,
        restaurante -> Nullable<Jsonb>,
        entradas -> Nullable<Jsonb>,
        fechas -> Nullable<Jsonb>,
        #[max_length = 200]
        lugar_recojo -> Nullable<Varchar>,
        hora_recojo -> Nullable<Timestamptz>,
        notas -> Nullable<Text>,
        #[max_length = 30]
        status -> Varchar,
        monto_total -> Numeric,
        monto_pagado -> Numeric,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    guias (id) {
        id -> Uuid,
        id_persona -> Uuid,
        #[max_length = 30]
        nro_carnet -> Varchar,
        idiomas -> Nullable<Jsonb>,
        especialidades -> Nullable<Jsonb>,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    login_attempts (id) {
        id -> Uuid,
        #[max_length = 255]
        identifier -> Varchar,
        #[max_length = 45]
        ip_address -> Varchar,
        user_agent -> Nullable<Text>,
        success -> Bool,
        failure_reason -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_providers (id) {
        id -> Int4,
        user_id -> Uuid,
        #[max_length = 50]
        provider -> Varchar,
        #[max_length = 255]
        provider_user_id -> Varchar,
        access_token -> Nullable<Text>,
        refresh_token -> Nullable<Text>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    pagos (id) {
        id -> Uuid,
        id_file -> Uuid,
        #[max_length = 30]
        tipo_movimiento -> Varchar,
        #[max_length = 200]
        concepto -> Varchar,
        monto -> Numeric,
        #[max_length = 50]
        metodo_pago -> Nullable<Varchar>,
        #[max_length = 100]
        referencia -> Nullable<Varchar>,
        evidencia -> Nullable<Jsonb>,
        fecha_pago -> Timestamptz,
        notas -> Nullable<Text>,
        registrado_por -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    personas (id) {
        id -> Uuid,
        #[max_length = 30]
        tipo_documento -> Varchar,
        #[max_length = 20]
        nro_documento -> Varchar,
        #[max_length = 100]
        nombre -> Varchar,
        #[max_length = 100]
        apellidos -> Varchar,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        fecha_nacimiento -> Nullable<Date>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        session_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        used_at -> Nullable<Timestamptz>,
        is_revoked -> Bool,
    }
}

diesel::table! {
    restaurantes (id) {
        id -> Uuid,
        #[max_length = 200]
        nombre -> Varchar,
        direccion -> Text,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        tipo_atencion -> Nullable<Jsonb>,
        precio_promedio -> Nullable<Numeric>,
        capacidad -> Nullable<Int4>,
        horario -> Nullable<Jsonb>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    tours (id) {
        id -> Uuid,
        id_agencia -> Uuid,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 200]
        lugar_inicio -> Varchar,
        #[max_length = 200]
        lugar_fin -> Varchar,
        hora_inicio -> Nullable<Timestamptz>,
        hora_fin -> Nullable<Timestamptz>,
        detalles -> Nullable<Jsonb>,
        itinerario -> Nullable<Jsonb>,
        precio -> Numeric,
        duracion_dias -> Nullable<Int4>,
        max_personas -> Nullable<Int4>,
        media -> Nullable<Jsonb>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    transportes (id) {
        id -> Uuid,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 11]
        ruc -> Varchar,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        direccion -> Nullable<Text>,
        encargado -> Nullable<Uuid>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        #[max_length = 255]
        refresh_token_hash -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        refresh_expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        #[max_length = 100]
        device_fingerprint -> Nullable<Varchar>,
        is_active -> Bool,
        revoked_at -> Nullable<Timestamptz>,
        #[max_length = 50]
        revoked_reason -> Nullable<Varchar>,
        last_activity_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Text,
        #[max_length = 20]
        role -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
        id_persona -> Nullable<Uuid>,
        id_entidad -> Nullable<Uuid>,
        #[max_length = 200]
        nombre_entidad -> Nullable<Varchar>,
        #[max_length = 30]
        status -> Varchar,
    }
}

diesel::table! {
    vehiculos (id) {
        id -> Uuid,
        id_transporte -> Uuid,
        #[max_length = 100]
        nombre -> Varchar,
        #[max_length = 100]
        modelo -> Nullable<Varchar>,
        #[max_length = 10]
        placa -> Varchar,
        capacidad -> Int4,
        #[max_length = 20]
        status -> Varchar,
        media -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(agencias -> personas (encargado));
diesel::joinable!(conductores -> personas (id_persona));
diesel::joinable!(conductores -> transportes (id_transporte));
diesel::joinable!(files -> agencias (id_agencia));
diesel::joinable!(files -> tours (id_tour));
diesel::joinable!(files -> users (created_by));
diesel::joinable!(guias -> personas (id_persona));
diesel::joinable!(oauth_providers -> users (user_id));
diesel::joinable!(pagos -> files (id_file));
diesel::joinable!(pagos -> users (registrado_por));
diesel::joinable!(refresh_tokens -> user_sessions (session_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(tours -> agencias (id_agencia));
diesel::joinable!(transportes -> personas (encargado));
diesel::joinable!(user_sessions -> users (user_id));
diesel::joinable!(users -> personas (id_persona));
diesel::joinable!(vehiculos -> transportes (id_transporte));

diesel::allow_tables_to_appear_in_same_query!(
    agencias,
    conductores,
    entradas,
    files,
    guias,
    login_attempts,
    oauth_providers,
    pagos,
    personas,
    refresh_tokens,
    restaurantes,
    tours,
    transportes,
    user_sessions,
    users,
    vehiculos,
);
