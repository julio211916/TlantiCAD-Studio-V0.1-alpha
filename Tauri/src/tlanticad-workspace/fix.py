import sys
with open('/Users/juliocesar/Desktop/gesto/crates/tlanticad-db/src/repository.rs', 'r') as f:
    text = f.read()

# remove my previous insertion
text = text.replace("""    /// Eliminar proyecto y dependencias
    pub async fn delete(&self, id: Id) -> Result<()> {
        let mut tx = self.db.pool().begin().await.map_err(|e| TlantiError::Database(e.to_string()))?;
        
        sqlx::query("DELETE FROM projects WHERE id = ?1")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await.map_err(|e| TlantiError::Database(e.to_string()))?;
            
        tx.commit().await.map_err(|e| TlantiError::Database(e.to_string()))?;
        Ok(())
    }
}""", "}")

with open('/Users/juliocesar/Desktop/gesto/crates/tlanticad-db/src/repository.rs', 'w') as f:
    f.write(text)
