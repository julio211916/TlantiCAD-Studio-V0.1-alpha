//! Database schema definitions

/// All table creation SQL statements
pub const CREATE_TABLES: &[&str] = &[
    // Clinics table
    r#"
    CREATE TABLE IF NOT EXISTS clinics (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        legal_name TEXT,
        address TEXT NOT NULL,
        city TEXT NOT NULL,
        state TEXT,
        postal_code TEXT,
        country TEXT DEFAULT 'Mexico',
        phone TEXT NOT NULL,
        phone_secondary TEXT,
        email TEXT,
        website TEXT,
        tax_id TEXT,
        logo_url TEXT,
        chair_count INTEGER DEFAULT 1,
        timezone TEXT DEFAULT 'America/Mexico_City',
        currency TEXT DEFAULT 'MXN',
        default_tax_rate TEXT DEFAULT '16',
        settings TEXT,
        is_main INTEGER DEFAULT 0,
        active INTEGER DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Users table
    r#"
    CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        username TEXT NOT NULL UNIQUE,
        email TEXT NOT NULL UNIQUE,
        password_hash TEXT,
        pin_hash TEXT,
        first_name TEXT NOT NULL,
        last_name TEXT NOT NULL,
        role TEXT NOT NULL,
        phone TEXT,
        specialty TEXT,
        license_number TEXT,
        professional_id TEXT,
        clinic_ids TEXT,
        photo_url TEXT,
        signature_path TEXT,
        calendar_color TEXT,
        active INTEGER DEFAULT 1,
        email_verified INTEGER DEFAULT 0,
        last_login TEXT,
        two_factor_enabled INTEGER DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Patients table
    r#"
    CREATE TABLE IF NOT EXISTS patients (
        id TEXT PRIMARY KEY,
        patient_number TEXT NOT NULL UNIQUE,
        first_name TEXT NOT NULL,
        last_name TEXT NOT NULL,
        middle_name TEXT,
        birth_date TEXT NOT NULL,
        gender TEXT NOT NULL,
        phone TEXT NOT NULL,
        phone_secondary TEXT,
        email TEXT,
        address TEXT,
        id_document TEXT,
        id_document_type TEXT,
        occupation TEXT,
        workplace TEXT,
        emergency_contact TEXT,
        allergies TEXT,
        medical_history TEXT,
        referral_source TEXT,
        notes TEXT,
        photo_url TEXT,
        insurance TEXT,
        preferred_reminder TEXT,
        active INTEGER DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Procedures/Services catalog
    r#"
    CREATE TABLE IF NOT EXISTS procedures (
        id TEXT PRIMARY KEY,
        code TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        description TEXT,
        category TEXT NOT NULL,
        default_price TEXT NOT NULL,
        min_price TEXT,
        duration_minutes INTEGER DEFAULT 30,
        per_tooth INTEGER DEFAULT 0,
        per_quadrant INTEGER DEFAULT 0,
        per_arch INTEGER DEFAULT 0,
        required_products TEXT,
        warranty_months INTEGER,
        notes TEXT,
        active INTEGER DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Appointments table
    r#"
    CREATE TABLE IF NOT EXISTS appointments (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        doctor_id TEXT NOT NULL REFERENCES users(id),
        datetime TEXT NOT NULL,
        duration_minutes INTEGER NOT NULL,
        chair_number INTEGER,
        clinic_id TEXT REFERENCES clinics(id),
        status TEXT NOT NULL DEFAULT 'scheduled',
        reason TEXT,
        procedures TEXT,
        notes TEXT,
        internal_notes TEXT,
        confirmation_sent INTEGER DEFAULT 0,
        reminder_sent INTEGER DEFAULT 0,
        checked_in_at TEXT,
        started_at TEXT,
        completed_at TEXT,
        cancel_reason TEXT,
        recurring_group_id TEXT,
        color TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        created_by TEXT NOT NULL REFERENCES users(id)
    )
    "#,
    
    // Treatment plans table
    r#"
    CREATE TABLE IF NOT EXISTS treatment_plans (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        created_by TEXT NOT NULL REFERENCES users(id),
        name TEXT NOT NULL,
        description TEXT,
        status TEXT NOT NULL DEFAULT 'draft',
        total_estimated TEXT NOT NULL DEFAULT '0',
        total_discount TEXT NOT NULL DEFAULT '0',
        total_final TEXT NOT NULL DEFAULT '0',
        approved_at TEXT,
        signature_path TEXT,
        notes TEXT,
        valid_until TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Treatment plan items
    r#"
    CREATE TABLE IF NOT EXISTS treatment_plan_items (
        id TEXT PRIMARY KEY,
        treatment_plan_id TEXT NOT NULL REFERENCES treatment_plans(id) ON DELETE CASCADE,
        procedure_id TEXT NOT NULL REFERENCES procedures(id),
        tooth_number INTEGER,
        surfaces TEXT,
        quadrant INTEGER,
        quantity INTEGER DEFAULT 1,
        unit_price TEXT NOT NULL,
        discount TEXT DEFAULT '0',
        total TEXT NOT NULL,
        priority INTEGER DEFAULT 0,
        notes TEXT,
        status TEXT DEFAULT 'planned'
    )
    "#,
    
    // Treatments (performed)
    r#"
    CREATE TABLE IF NOT EXISTS treatments (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        appointment_id TEXT REFERENCES appointments(id),
        treatment_plan_id TEXT REFERENCES treatment_plans(id),
        procedure_id TEXT NOT NULL REFERENCES procedures(id),
        doctor_id TEXT NOT NULL REFERENCES users(id),
        tooth_number INTEGER,
        surfaces TEXT,
        quadrant INTEGER,
        status TEXT NOT NULL DEFAULT 'planned',
        price TEXT NOT NULL,
        discount TEXT DEFAULT '0',
        final_price TEXT NOT NULL,
        notes TEXT,
        planned_date TEXT,
        completed_at TEXT,
        warranty_until TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Odontogram entries
    r#"
    CREATE TABLE IF NOT EXISTS odontogram_entries (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        tooth_number INTEGER NOT NULL,
        surface_conditions TEXT,
        primary_condition TEXT NOT NULL DEFAULT 'healthy',
        treatment_status TEXT,
        is_primary INTEGER DEFAULT 0,
        mobility INTEGER,
        notes TEXT,
        updated_at TEXT NOT NULL,
        updated_by TEXT NOT NULL REFERENCES users(id),
        UNIQUE(patient_id, tooth_number)
    )
    "#,

    // Odontogram history
    r#"
    CREATE TABLE IF NOT EXISTS odontogram_history (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        tooth_number INTEGER NOT NULL,
        previous_condition TEXT NOT NULL,
        new_condition TEXT NOT NULL,
        change_reason TEXT,
        changed_by TEXT NOT NULL REFERENCES users(id),
        changed_at TEXT NOT NULL
    )
    "#,

    // Periodontograms (stored as JSON payload)
    r#"
    CREATE TABLE IF NOT EXISTS periodontograms (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL UNIQUE REFERENCES patients(id),
        data TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        updated_by TEXT NOT NULL REFERENCES users(id)
    )
    "#,
    
    // Products (inventory)
    r#"
    CREATE TABLE IF NOT EXISTS products (
        id TEXT PRIMARY KEY,
        sku TEXT NOT NULL UNIQUE,
        barcode TEXT,
        name TEXT NOT NULL,
        description TEXT,
        category TEXT NOT NULL,
        unit TEXT NOT NULL,
        current_stock INTEGER DEFAULT 0,
        min_stock INTEGER DEFAULT 0,
        max_stock INTEGER,
        reorder_point INTEGER,
        unit_cost TEXT NOT NULL DEFAULT '0',
        unit_price TEXT NOT NULL DEFAULT '0',
        supplier_id TEXT REFERENCES suppliers(id),
        location TEXT,
        brand TEXT,
        manufacturer TEXT,
        taxable INTEGER DEFAULT 1,
        tax_rate TEXT,
        active INTEGER DEFAULT 1,
        image_url TEXT,
        notes TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Suppliers
    r#"
    CREATE TABLE IF NOT EXISTS suppliers (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        contact_name TEXT,
        phone TEXT NOT NULL,
        phone_secondary TEXT,
        email TEXT,
        address TEXT,
        city TEXT,
        tax_id TEXT,
        website TEXT,
        payment_terms INTEGER,
        notes TEXT,
        active INTEGER DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Stock movements
    r#"
    CREATE TABLE IF NOT EXISTS stock_movements (
        id TEXT PRIMARY KEY,
        product_id TEXT NOT NULL REFERENCES products(id),
        movement_type TEXT NOT NULL,
        quantity INTEGER NOT NULL,
        unit_cost TEXT NOT NULL,
        total_value TEXT NOT NULL,
        reference TEXT,
        related_id TEXT,
        batch_number TEXT,
        expiration_date TEXT,
        notes TEXT,
        user_id TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL
    )
    "#,

    // Quotes (presupuestos)
    r#"
    CREATE TABLE IF NOT EXISTS quotes (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        created_by TEXT NOT NULL REFERENCES users(id),
        notes TEXT,
        total TEXT NOT NULL DEFAULT '0',
        created_at TEXT NOT NULL
    )
    "#,

    // Quote items
    r#"
    CREATE TABLE IF NOT EXISTS quote_items (
        id TEXT PRIMARY KEY,
        quote_id TEXT NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
        description TEXT NOT NULL,
        quantity INTEGER NOT NULL DEFAULT 1,
        unit_price TEXT NOT NULL,
        total TEXT NOT NULL
    )
    "#,
    
    // Invoices
    r#"
    CREATE TABLE IF NOT EXISTS invoices (
        id TEXT PRIMARY KEY,
        invoice_number TEXT NOT NULL UNIQUE,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        clinic_id TEXT REFERENCES clinics(id),
        date TEXT NOT NULL,
        due_date TEXT,
        status TEXT NOT NULL DEFAULT 'draft',
        subtotal TEXT NOT NULL DEFAULT '0',
        tax TEXT NOT NULL DEFAULT '0',
        tax_rate TEXT NOT NULL DEFAULT '16',
        discount TEXT NOT NULL DEFAULT '0',
        total TEXT NOT NULL DEFAULT '0',
        amount_paid TEXT NOT NULL DEFAULT '0',
        balance TEXT NOT NULL DEFAULT '0',
        notes TEXT,
        internal_notes TEXT,
        cfdi_uuid TEXT,
        cfdi_status TEXT,
        created_by TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Invoice items
    r#"
    CREATE TABLE IF NOT EXISTS invoice_items (
        id TEXT PRIMARY KEY,
        invoice_id TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
        treatment_id TEXT REFERENCES treatments(id),
        product_id TEXT REFERENCES products(id),
        description TEXT NOT NULL,
        quantity INTEGER NOT NULL DEFAULT 1,
        unit_price TEXT NOT NULL,
        discount TEXT DEFAULT '0',
        total TEXT NOT NULL
    )
    "#,
    
    // Payments
    r#"
    CREATE TABLE IF NOT EXISTS payments (
        id TEXT PRIMARY KEY,
        invoice_id TEXT NOT NULL REFERENCES invoices(id),
        amount TEXT NOT NULL,
        payment_method TEXT NOT NULL,
        reference TEXT,
        date TEXT NOT NULL,
        authorization_code TEXT,
        terminal_id TEXT,
        notes TEXT,
        is_refund INTEGER DEFAULT 0,
        refund_reason TEXT,
        received_by TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL
    )
    "#,
    
    // Documents
    r#"
    CREATE TABLE IF NOT EXISTS documents (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        template_id TEXT REFERENCES document_templates(id),
        appointment_id TEXT REFERENCES appointments(id),
        document_type TEXT NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        file_path TEXT,
        mime_type TEXT,
        file_size INTEGER,
        signed INTEGER DEFAULT 0,
        signature_path TEXT,
        signature_date TEXT,
        signed_by TEXT,
        patient_visible INTEGER DEFAULT 0,
        notes TEXT,
        created_by TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Document templates
    r#"
    CREATE TABLE IF NOT EXISTS document_templates (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT,
        document_type TEXT NOT NULL,
        category TEXT,
        content TEXT NOT NULL,
        variables TEXT,
        header TEXT,
        footer TEXT,
        styles TEXT,
        active INTEGER DEFAULT 1,
        is_system INTEGER DEFAULT 0,
        created_by TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,
    
    // Clinical notes
    r#"
    CREATE TABLE IF NOT EXISTS clinical_notes (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        appointment_id TEXT REFERENCES appointments(id),
        user_id TEXT NOT NULL REFERENCES users(id),
        note_type TEXT NOT NULL,
        content TEXT NOT NULL,
        attachments TEXT,
        created_at TEXT NOT NULL
    )
    "#,
    
    // Audit log
    r#"
    CREATE TABLE IF NOT EXISTS audit_log (
        id TEXT PRIMARY KEY,
        user_id TEXT REFERENCES users(id),
        action TEXT NOT NULL,
        entity_type TEXT NOT NULL,
        entity_id TEXT,
        old_value TEXT,
        new_value TEXT,
        ip_address TEXT,
        timestamp TEXT NOT NULL
    )
    "#,
    
    // Settings
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        description TEXT,
        updated_at TEXT NOT NULL,
        updated_by TEXT REFERENCES users(id)
    )
    "#,
    
    // Events store
    r#"
    CREATE TABLE IF NOT EXISTS events (
        id TEXT PRIMARY KEY,
        event_type TEXT NOT NULL,
        payload TEXT NOT NULL,
        aggregate_id TEXT,
        aggregate_type TEXT,
        user_id TEXT,
        timestamp TEXT NOT NULL
    )
    "#,

    // Patient files (DICOM, STL, images, documents)
    r#"
    CREATE TABLE IF NOT EXISTS patient_files (
        id TEXT PRIMARY KEY,
        patient_id TEXT NOT NULL REFERENCES patients(id),
        file_type TEXT NOT NULL,
        file_name TEXT NOT NULL,
        file_path TEXT NOT NULL,
        file_size INTEGER NOT NULL DEFAULT 0,
        mime_type TEXT,
        category TEXT NOT NULL DEFAULT 'general',
        description TEXT,
        metadata TEXT,
        tooth_number INTEGER,
        appointment_id TEXT REFERENCES appointments(id),
        uploaded_by TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,

    // Support tickets
    r#"
    CREATE TABLE IF NOT EXISTS support_tickets (
        id TEXT PRIMARY KEY,
        subject TEXT NOT NULL,
        description TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'open',
        priority TEXT NOT NULL DEFAULT 'medium',
        customer_name TEXT NOT NULL,
        customer_email TEXT NOT NULL,
        assigned_to TEXT REFERENCES users(id),
        assigned_to_name TEXT,
        first_response_at TEXT,
        last_response_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    )
    "#,

    // Support ticket messages
    r#"
    CREATE TABLE IF NOT EXISTS support_ticket_messages (
        id TEXT PRIMARY KEY,
        ticket_id TEXT NOT NULL REFERENCES support_tickets(id) ON DELETE CASCADE,
        body TEXT NOT NULL,
        author_type TEXT NOT NULL,
        author_id TEXT,
        author_name TEXT NOT NULL,
        is_internal INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    )
    "#,

    // Canned responses for support agents
    r#"
    CREATE TABLE IF NOT EXISTS support_canned_responses (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        body TEXT NOT NULL,
        is_active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL
    )
    "#,
];

/// Indexes for better query performance
pub const CREATE_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_patients_number ON patients(patient_number)",
    "CREATE INDEX IF NOT EXISTS idx_patients_name ON patients(last_name, first_name)",
    "CREATE INDEX IF NOT EXISTS idx_patients_phone ON patients(phone)",
    "CREATE INDEX IF NOT EXISTS idx_patients_active ON patients(active)",
    
    "CREATE INDEX IF NOT EXISTS idx_appointments_patient ON appointments(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_appointments_doctor ON appointments(doctor_id)",
    "CREATE INDEX IF NOT EXISTS idx_appointments_datetime ON appointments(datetime)",
    "CREATE INDEX IF NOT EXISTS idx_appointments_status ON appointments(status)",
    
    "CREATE INDEX IF NOT EXISTS idx_treatments_patient ON treatments(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_treatments_appointment ON treatments(appointment_id)",
    "CREATE INDEX IF NOT EXISTS idx_treatments_status ON treatments(status)",

    "CREATE INDEX IF NOT EXISTS idx_odontogram_patient ON odontogram_entries(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_odontogram_history_patient ON odontogram_history(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_periodontograms_patient ON periodontograms(patient_id)",
    
    "CREATE INDEX IF NOT EXISTS idx_invoices_patient ON invoices(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_invoices_number ON invoices(invoice_number)",
    "CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status)",
    "CREATE INDEX IF NOT EXISTS idx_invoices_date ON invoices(date)",
    
    "CREATE INDEX IF NOT EXISTS idx_payments_invoice ON payments(invoice_id)",
    "CREATE INDEX IF NOT EXISTS idx_payments_date ON payments(date)",
    
    "CREATE INDEX IF NOT EXISTS idx_products_sku ON products(sku)",
    "CREATE INDEX IF NOT EXISTS idx_products_category ON products(category)",
    "CREATE INDEX IF NOT EXISTS idx_products_supplier ON products(supplier_id)",
    
    "CREATE INDEX IF NOT EXISTS idx_stock_movements_product ON stock_movements(product_id)",
    "CREATE INDEX IF NOT EXISTS idx_stock_movements_date ON stock_movements(created_at)",

    "CREATE INDEX IF NOT EXISTS idx_quotes_patient ON quotes(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_quote_items_quote ON quote_items(quote_id)",
    
    "CREATE INDEX IF NOT EXISTS idx_documents_patient ON documents(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_documents_type ON documents(document_type)",
    
    "CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp)",
    
    "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
    "CREATE INDEX IF NOT EXISTS idx_events_aggregate ON events(aggregate_type, aggregate_id)",
    "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp)",

    "CREATE INDEX IF NOT EXISTS idx_patient_files_patient ON patient_files(patient_id)",
    "CREATE INDEX IF NOT EXISTS idx_patient_files_type ON patient_files(file_type)",
    "CREATE INDEX IF NOT EXISTS idx_patient_files_category ON patient_files(category)",
    "CREATE INDEX IF NOT EXISTS idx_patient_files_tooth ON patient_files(tooth_number)",

    "CREATE INDEX IF NOT EXISTS idx_support_tickets_status ON support_tickets(status)",
    "CREATE INDEX IF NOT EXISTS idx_support_tickets_priority ON support_tickets(priority)",
    "CREATE INDEX IF NOT EXISTS idx_support_tickets_customer_email ON support_tickets(customer_email)",
    "CREATE INDEX IF NOT EXISTS idx_support_tickets_updated_at ON support_tickets(updated_at)",
    "CREATE INDEX IF NOT EXISTS idx_support_ticket_messages_ticket_id ON support_ticket_messages(ticket_id)",
];
