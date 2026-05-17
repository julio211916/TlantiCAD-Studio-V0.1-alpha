//! Excel export templates for each dental entity
//!
//! Predefined column layouts for: Patients, Appointments, Treatments,
//! Invoices, Payments, Inventory, Users

use crate::excel_writer::{ColumnFormat, ExportColumn};

pub fn patient_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "Nro Paciente".into(), field: "patient_number".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Nombre".into(), field: "first_name".into(), width: 18.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Apellido".into(), field: "last_name".into(), width: 18.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Email".into(), field: "email".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Teléfono".into(), field: "phone".into(), width: 15.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Fecha Nacimiento".into(), field: "birth_date".into(), width: 16.0, format_type: ColumnFormat::Date },
        ExportColumn { header: "Género".into(), field: "gender".into(), width: 10.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Dirección".into(), field: "address".into(), width: 30.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Seguro".into(), field: "insurance_provider".into(), width: 20.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Nro Seguro".into(), field: "insurance_number".into(), width: 16.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Alergias".into(), field: "allergies".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Medicamentos".into(), field: "medications".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Notas".into(), field: "notes".into(), width: 30.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Activo".into(), field: "active".into(), width: 8.0, format_type: ColumnFormat::Boolean },
        ExportColumn { header: "Creado".into(), field: "created_at".into(), width: 20.0, format_type: ColumnFormat::DateTime },
    ]
}

pub fn appointment_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "Paciente".into(), field: "patient_name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Doctor".into(), field: "doctor_name".into(), width: 20.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Fecha".into(), field: "date".into(), width: 14.0, format_type: ColumnFormat::Date },
        ExportColumn { header: "Hora Inicio".into(), field: "start_time".into(), width: 12.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Hora Fin".into(), field: "end_time".into(), width: 12.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Sillón".into(), field: "chair".into(), width: 10.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Estado".into(), field: "status".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Motivo".into(), field: "reason".into(), width: 30.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Notas".into(), field: "notes".into(), width: 30.0, format_type: ColumnFormat::Text },
    ]
}

pub fn treatment_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "Paciente".into(), field: "patient_name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Tratamiento".into(), field: "name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Diente".into(), field: "tooth_number".into(), width: 10.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Superficie".into(), field: "surface".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Estado".into(), field: "status".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Costo".into(), field: "cost".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Fecha Inicio".into(), field: "start_date".into(), width: 14.0, format_type: ColumnFormat::Date },
        ExportColumn { header: "Fecha Fin".into(), field: "end_date".into(), width: 14.0, format_type: ColumnFormat::Date },
        ExportColumn { header: "Doctor".into(), field: "doctor_name".into(), width: 20.0, format_type: ColumnFormat::Text },
    ]
}

pub fn invoice_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "Nro Factura".into(), field: "invoice_number".into(), width: 16.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Paciente".into(), field: "patient_name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Subtotal".into(), field: "subtotal".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Descuento".into(), field: "discount".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Impuesto".into(), field: "tax".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Total".into(), field: "total".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Pagado".into(), field: "paid".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Balance".into(), field: "balance".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Estado".into(), field: "status".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Fecha".into(), field: "date".into(), width: 14.0, format_type: ColumnFormat::Date },
    ]
}

pub fn payment_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "Nro Factura".into(), field: "invoice_number".into(), width: 16.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Paciente".into(), field: "patient_name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Monto".into(), field: "amount".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Método Pago".into(), field: "payment_method".into(), width: 16.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Referencia".into(), field: "reference".into(), width: 20.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Fecha".into(), field: "date".into(), width: 14.0, format_type: ColumnFormat::Date },
        ExportColumn { header: "Recibido Por".into(), field: "received_by".into(), width: 20.0, format_type: ColumnFormat::Text },
    ]
}

pub fn inventory_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "SKU".into(), field: "sku".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Producto".into(), field: "name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Categoría".into(), field: "category".into(), width: 18.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Precio".into(), field: "price".into(), width: 14.0, format_type: ColumnFormat::Currency },
        ExportColumn { header: "Stock Actual".into(), field: "current_stock".into(), width: 14.0, format_type: ColumnFormat::Number },
        ExportColumn { header: "Stock Mínimo".into(), field: "min_stock".into(), width: 14.0, format_type: ColumnFormat::Number },
        ExportColumn { header: "Unidad".into(), field: "unit".into(), width: 12.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Proveedor".into(), field: "supplier".into(), width: 20.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Activo".into(), field: "active".into(), width: 8.0, format_type: ColumnFormat::Boolean },
    ]
}

pub fn user_columns() -> Vec<ExportColumn> {
    vec![
        ExportColumn { header: "Nombre".into(), field: "full_name".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Email".into(), field: "email".into(), width: 25.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Rol".into(), field: "role".into(), width: 14.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Especialidad".into(), field: "specialty".into(), width: 20.0, format_type: ColumnFormat::Text },
        ExportColumn { header: "Activo".into(), field: "active".into(), width: 8.0, format_type: ColumnFormat::Boolean },
        ExportColumn { header: "Último Acceso".into(), field: "last_login".into(), width: 20.0, format_type: ColumnFormat::DateTime },
    ]
}
