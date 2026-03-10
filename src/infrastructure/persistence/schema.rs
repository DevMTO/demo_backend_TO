// @generated automatically by Diesel CLI.

diesel::table! {
    activity_logs (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        #[max_length = 50]
        username -> Nullable<Varchar>,
        #[max_length = 30]
        action_type -> Varchar,
        #[max_length = 50]
        action -> Varchar,
        #[max_length = 50]
        entity_type -> Varchar,
        entity_id -> Nullable<Int4>,
        description -> Nullable<Text>,
        old_values -> Nullable<Jsonb>,
        new_values -> Nullable<Jsonb>,
        changed_fields -> Nullable<Jsonb>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        #[max_length = 20]
        status -> Varchar,
        error_message -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    agencias (id) {
        id -> Int4,
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
        encargado -> Nullable<Int4>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        pago_anticipado -> Bool,
        #[max_length = 20]
        tipo_vencimiento -> Nullable<Varchar>,
    }
}

diesel::table! {
    cadenas_hoteleras (id) {
        id -> Int4,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        media -> Nullable<Jsonb>,
        encargado -> Nullable<Int4>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        paleta_colores -> Nullable<Jsonb>,
    }
}

diesel::table! {
    conductores (id) {
        id -> Int4,
        id_persona -> Int4,
        id_transporte -> Nullable<Int4>,
        #[max_length = 20]
        nro_brevete -> Varchar,
        tiene_soat -> Bool,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        is_active -> Bool,
    }
}

diesel::table! {
    entrada_precios (id) {
        id -> Int4,
        id_entrada -> Int4,
        #[max_length = 30]
        tipo_precio -> Varchar,
        edad_minima -> Int4,
        edad_maxima -> Nullable<Int4>,
        precio -> Numeric,
        #[max_length = 100]
        descripcion -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
    }
}

diesel::table! {
    entradas (id) {
        id -> Int4,
        #[max_length = 200]
        nombre -> Varchar,
        descripcion -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        tours_asociados -> Nullable<Jsonb>,
        boleto_turistico -> Bool,
    }
}

diesel::table! {
    file_entradas (id) {
        id -> Int4,
        id_entrada -> Int4,
        cantidad -> Int4,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        id_file_tour -> Int4,
        id_entrada_precio -> Nullable<Int4>,
        #[max_length = 20]
        status -> Varchar,
        cancelaciones -> Array<Nullable<Int4>>,
    }
}

diesel::table! {
    file_guias (id) {
        id -> Int4,
        id_guia -> Int4,
        #[max_length = 30]
        rol -> Nullable<Varchar>,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        #[max_length = 20]
        estado_confirmacion -> Varchar,
        confirmado_at -> Nullable<Timestamptz>,
        motivo_rechazo -> Nullable<Text>,
        id_file_tour -> Int4,
        #[max_length = 20]
        status -> Varchar,
    }
}

diesel::table! {
    file_pasajeros (id) {
        id -> Int4,
        id_file -> Int4,
        id_persona -> Nullable<Int4>,
        #[max_length = 10]
        asiento -> Nullable<Varchar>,
        #[max_length = 30]
        tipo_pasajero -> Nullable<Varchar>,
        notas -> Nullable<Text>,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        #[max_length = 60]
        nacionalidad -> Nullable<Varchar>,
        edad -> Nullable<Int4>,
        #[max_length = 20]
        status -> Varchar,
    }
}

diesel::table! {
    file_restaurantes (id) {
        id -> Int4,
        id_restaurante -> Int4,
        #[max_length = 30]
        tipo_servicio -> Nullable<Varchar>,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        precio -> Nullable<Numeric>,
        id_file_tour -> Int4,
        #[max_length = 20]
        status -> Varchar,
    }
}

diesel::table! {
    file_tours (id) {
        id -> Int4,
        id_file -> Int4,
        id_tour -> Int4,
        orden -> Int4,
        precio_aplicado -> Nullable<Numeric>,
        notas -> Nullable<Text>,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        fecha_tour -> Nullable<Date>,
        #[max_length = 30]
        turno_tour -> Nullable<Varchar>,
        #[max_length = 200]
        lugar_recojo -> Nullable<Varchar>,
        hora_recojo -> Nullable<Time>,
        #[max_length = 30]
        status -> Varchar,
        geo_recojo -> Nullable<Jsonb>,
        nro_pasajeros -> Nullable<Int4>,
    }
}

diesel::table! {
    file_vehiculos (id) {
        id -> Int4,
        id_vehiculo -> Int4,
        id_conductor -> Nullable<Int4>,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        capacidad_asignada -> Int4,
        id_file_tour -> Int4,
        #[max_length = 20]
        status -> Varchar,
    }
}

diesel::table! {
    files (id) {
        id -> Int4,
        id_entidad -> Int4,
        fecha_inicio -> Date,
        fecha_fin -> Date,
        notas -> Nullable<Text>,
        #[max_length = 30]
        status -> Varchar,
        monto_total -> Numeric,
        monto_pagado -> Numeric,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        is_active -> Bool,
        nro_pasajeros -> Int4,
        #[max_length = 50]
        file_code -> Nullable<Varchar>,
        deadline_confirmacion -> Nullable<Timestamptz>,
        #[max_length = 50]
        entidad -> Nullable<Varchar>,
    }
}

diesel::table! {
    guias (id) {
        id -> Int4,
        id_persona -> Int4,
        #[max_length = 30]
        nro_carnet -> Varchar,
        idiomas -> Nullable<Jsonb>,
        especialidades -> Nullable<Jsonb>,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        is_active -> Bool,
    }
}

diesel::table! {
    hoteles (id) {
        id -> Int4,
        id_cadena -> Int4,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 50]
        categoria -> Nullable<Varchar>,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        direccion -> Nullable<Text>,
        #[max_length = 100]
        ciudad -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
    }
}

diesel::table! {
    notification_users (id) {
        id -> Int4,
        notification_id -> Int4,
        user_id -> Int4,
        is_read -> Bool,
        read_at -> Nullable<Timestamptz>,
        is_dismissed -> Bool,
        dismissed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    notifications (id) {
        id -> Int4,
        #[max_length = 50]
        notification_type -> Varchar,
        #[max_length = 50]
        category -> Varchar,
        #[max_length = 200]
        title -> Varchar,
        message -> Text,
        #[max_length = 50]
        entity_type -> Nullable<Varchar>,
        entity_id -> Nullable<Int4>,
        metadata -> Nullable<Jsonb>,
        #[max_length = 20]
        priority -> Varchar,
        target_roles -> Nullable<Jsonb>,
        target_user_id -> Nullable<Int4>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        created_by -> Nullable<Int4>,
    }
}

diesel::table! {
    pagos_files (id) {
        id -> Int4,
        id_file -> Int4,
        id_entidad -> Int4,
        monto_total -> Numeric,
        monto_pagado -> Numeric,
        #[max_length = 30]
        estado -> Varchar,
        fecha_vencimiento -> Nullable<Date>,
        comprobante_url -> Nullable<Text>,
        comprobante_key -> Nullable<Text>,
        verificado_por -> Nullable<Int4>,
        verificado_at -> Nullable<Timestamptz>,
        notas -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        id_file_tour -> Nullable<Int4>,
        #[max_length = 30]
        tipo_registro -> Varchar,
        monto_saldo_favor -> Nullable<Numeric>,
        saldo_autorizado -> Bool,
        saldo_autorizado_por -> Nullable<Int4>,
        saldo_autorizado_at -> Nullable<Timestamptz>,
        entradas -> Bool,
        entrada_precio -> Nullable<Numeric>,
        cuota -> Nullable<Int2>,
        #[max_length = 50]
        entidad -> Nullable<Varchar>,
    }
}

diesel::table! {
    pagos_proveedores (id) {
        id -> Int4,
        #[max_length = 20]
        tipo_proveedor -> Varchar,
        id_transporte -> Nullable<Int4>,
        id_restaurante -> Nullable<Int4>,
        id_guia -> Nullable<Int4>,
        id_file_tour -> Nullable<Int4>,
        id_file_vehiculo -> Nullable<Int4>,
        id_file_restaurante -> Nullable<Int4>,
        id_file_guia -> Nullable<Int4>,
        monto_total -> Numeric,
        #[max_length = 20]
        estado -> Varchar,
        fecha_pago -> Nullable<Timestamptz>,
        comprobante_url -> Nullable<Text>,
        comprobante_key -> Nullable<Text>,
        notas -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        pagado_by -> Nullable<Int4>,
        id_entrada -> Nullable<Int4>,
        id_file_entrada -> Nullable<Int4>,
        monto_pagado -> Nullable<Numeric>,
    }
}

diesel::table! {
    personas (id) {
        id -> Int4,
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
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
    }
}

diesel::table! {
    restaurantes (id) {
        id -> Int4,
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
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        encargado -> Nullable<Int4>,
    }
}

diesel::table! {
    tours (id) {
        id -> Int4,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 200]
        lugar_inicio -> Nullable<Varchar>,
        #[max_length = 200]
        lugar_fin -> Nullable<Varchar>,
        detalles -> Nullable<Jsonb>,
        itinerario -> Nullable<Jsonb>,
        precio_base -> Numeric,
        duracion_dias -> Nullable<Int4>,
        media -> Nullable<Jsonb>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        #[max_length = 100]
        tipo_tour -> Nullable<Varchar>,
        horarios -> Nullable<Jsonb>,
        tiene_restaurante -> Bool,
        geo_inicio -> Nullable<Jsonb>,
        geo_fin -> Nullable<Jsonb>,
        geo_ruta -> Nullable<Jsonb>,
    }
}

diesel::table! {
    transportes (id) {
        id -> Int4,
        #[max_length = 200]
        nombre -> Varchar,
        #[max_length = 11]
        ruc -> Varchar,
        #[max_length = 20]
        telefono -> Nullable<Varchar>,
        #[max_length = 255]
        correo -> Nullable<Varchar>,
        direccion -> Nullable<Text>,
        encargado -> Nullable<Int4>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        media -> Nullable<Jsonb>,
        paleta_colores -> Nullable<Jsonb>,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 255]
        token_hash -> Varchar,
        #[max_length = 255]
        refresh_token_hash -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        refresh_expires_at -> Nullable<Timestamptz>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        #[max_length = 100]
        device_fingerprint -> Nullable<Varchar>,
        is_active -> Bool,
        last_activity -> Nullable<Timestamptz>,
        revoked_at -> Nullable<Timestamptz>,
        #[max_length = 50]
        revoked_reason -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        remember_me -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        id_persona -> Nullable<Int4>,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Text,
        #[max_length = 20]
        role -> Varchar,
        id_entidad -> Nullable<Int4>,
        last_login -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        is_active -> Bool,
    }
}

diesel::table! {
    vehiculos (id) {
        id -> Int4,
        id_transporte -> Int4,
        #[max_length = 100]
        nombre -> Varchar,
        #[max_length = 100]
        modelo -> Nullable<Varchar>,
        #[max_length = 10]
        placa -> Varchar,
        capacidad -> Int4,
        #[max_length = 20]
        status -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        created_by -> Nullable<Int4>,
        updated_by -> Nullable<Int4>,
        is_active -> Bool,
    }
}

diesel::joinable!(activity_logs -> users (user_id));
diesel::joinable!(agencias -> personas (encargado));
diesel::joinable!(cadenas_hoteleras -> personas (encargado));
diesel::joinable!(conductores -> personas (id_persona));
diesel::joinable!(conductores -> transportes (id_transporte));
diesel::joinable!(entrada_precios -> entradas (id_entrada));
diesel::joinable!(file_entradas -> entrada_precios (id_entrada_precio));
diesel::joinable!(file_entradas -> entradas (id_entrada));
diesel::joinable!(file_entradas -> file_tours (id_file_tour));
diesel::joinable!(file_entradas -> users (created_by));
diesel::joinable!(file_guias -> file_tours (id_file_tour));
diesel::joinable!(file_guias -> guias (id_guia));
diesel::joinable!(file_guias -> users (created_by));
diesel::joinable!(file_pasajeros -> files (id_file));
diesel::joinable!(file_pasajeros -> personas (id_persona));
diesel::joinable!(file_pasajeros -> users (created_by));
diesel::joinable!(file_restaurantes -> file_tours (id_file_tour));
diesel::joinable!(file_restaurantes -> restaurantes (id_restaurante));
diesel::joinable!(file_restaurantes -> users (created_by));
diesel::joinable!(file_tours -> files (id_file));
diesel::joinable!(file_tours -> tours (id_tour));
diesel::joinable!(file_tours -> users (created_by));
diesel::joinable!(file_vehiculos -> conductores (id_conductor));
diesel::joinable!(file_vehiculos -> file_tours (id_file_tour));
diesel::joinable!(file_vehiculos -> users (created_by));
diesel::joinable!(file_vehiculos -> vehiculos (id_vehiculo));
diesel::joinable!(guias -> personas (id_persona));
diesel::joinable!(hoteles -> cadenas_hoteleras (id_cadena));
diesel::joinable!(notification_users -> notifications (notification_id));
diesel::joinable!(notification_users -> users (user_id));
diesel::joinable!(pagos_files -> file_tours (id_file_tour));
diesel::joinable!(pagos_files -> files (id_file));
diesel::joinable!(pagos_proveedores -> entradas (id_entrada));
diesel::joinable!(pagos_proveedores -> file_entradas (id_file_entrada));
diesel::joinable!(pagos_proveedores -> file_guias (id_file_guia));
diesel::joinable!(pagos_proveedores -> file_restaurantes (id_file_restaurante));
diesel::joinable!(pagos_proveedores -> file_tours (id_file_tour));
diesel::joinable!(pagos_proveedores -> file_vehiculos (id_file_vehiculo));
diesel::joinable!(pagos_proveedores -> guias (id_guia));
diesel::joinable!(pagos_proveedores -> restaurantes (id_restaurante));
diesel::joinable!(pagos_proveedores -> transportes (id_transporte));
diesel::joinable!(restaurantes -> personas (encargado));
diesel::joinable!(transportes -> personas (encargado));
diesel::joinable!(user_sessions -> users (user_id));
diesel::joinable!(vehiculos -> transportes (id_transporte));

diesel::allow_tables_to_appear_in_same_query!(
    activity_logs,
    agencias,
    cadenas_hoteleras,
    conductores,
    entrada_precios,
    entradas,
    file_entradas,
    file_guias,
    file_pasajeros,
    file_restaurantes,
    file_tours,
    file_vehiculos,
    files,
    guias,
    hoteles,
    notification_users,
    notifications,
    pagos_files,
    pagos_proveedores,
    personas,
    restaurantes,
    tours,
    transportes,
    user_sessions,
    users,
    vehiculos,
);
