import sys

with open('/Users/juliocesar/Desktop/gesto/crates/tlanticad-db/src/repository.rs', 'r') as f:
    lines = f.readlines()

delete_fn = """
    /// Eliminar proyecto y dependencias
    pub async fn delete(&self, id: Id) -> Result<()> {
        let mut tx = self.db.pool().begin().await.map_err(|e| TlantiError::Database(e.to_string()))?;
        
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await.map_err(|e| TlantiError::Database(e.to_string()))?;
            
        tx.commit().await.map_err(|e| TlantiError::Database(e.to_string()))?;
        Ok(())
    }
}
"""

lines[231] = delete_fn + "\n"

with open('/Users/juliocesar/Desktop/gesto/crates/tlanticad-db/src/repository.rs', 'w') as f:
    f.writelines(lines)
