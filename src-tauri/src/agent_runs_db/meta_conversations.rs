// Meta agent conversation persistence module
//
// Handles all database operations for meta agent conversations including:
// - Creating and updating conversations
// - Inserting and retrieving messages
// - Listing conversations with filters
// - Deleting conversations

use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::models::{ConversationQueryFilters, MetaConversationRecord, MetaMessageRecord};

/// Operations for meta agent conversation persistence
pub struct MetaConversationOps<'a> {
    db: &'a Arc<Mutex<Connection>>,
}

impl<'a> MetaConversationOps<'a> {
    pub fn new(db: &'a Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Create a new conversation record
    pub async fn create_conversation(&self, record: &MetaConversationRecord) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO meta_conversations
             (conversation_id, title, created_at, updated_at, message_count, is_archived, preview_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.conversation_id,
                record.title,
                record.created_at,
                record.updated_at,
                record.message_count,
                if record.is_archived { 1 } else { 0 },
                record.preview_text
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    /// Update an existing conversation's metadata
    pub async fn update_conversation(&self, record: &MetaConversationRecord) -> SqliteResult<()> {
        let db = self.db.lock().await;

        db.execute(
            "UPDATE meta_conversations SET
             title = ?1,
             updated_at = ?2,
             message_count = ?3,
             is_archived = ?4,
             preview_text = ?5
             WHERE conversation_id = ?6",
            params![
                record.title,
                record.updated_at,
                record.message_count,
                if record.is_archived { 1 } else { 0 },
                record.preview_text,
                record.conversation_id
            ],
        )?;

        Ok(())
    }

    /// Get a conversation by ID
    pub async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> SqliteResult<Option<MetaConversationRecord>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT id, conversation_id, title, created_at, updated_at, message_count, is_archived, preview_text
             FROM meta_conversations
             WHERE conversation_id = ?1",
        )?;

        let mut rows = stmt.query(params![conversation_id])?;

        if let Some(row) = rows.next()? {
            let is_archived_int: i32 = row.get(6)?;
            Ok(Some(MetaConversationRecord {
                id: Some(row.get(0)?),
                conversation_id: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                message_count: row.get(5)?,
                is_archived: is_archived_int != 0,
                preview_text: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List conversations with optional filters
    pub async fn list_conversations(
        &self,
        filters: ConversationQueryFilters,
    ) -> SqliteResult<Vec<MetaConversationRecord>> {
        let db = self.db.lock().await;

        let mut query = "SELECT id, conversation_id, title, created_at, updated_at, message_count, is_archived, preview_text
                         FROM meta_conversations WHERE 1=1"
            .to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if !filters.include_archived {
            query.push_str(" AND is_archived = 0");
        }

        if let Some(ref search) = filters.search_text {
            query.push_str(" AND (title LIKE ? OR preview_text LIKE ?)");
            let search_pattern = format!("%{}%", search);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        query.push_str(" ORDER BY updated_at DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let is_archived_int: i32 = row.get(6)?;
            Ok(MetaConversationRecord {
                id: Some(row.get(0)?),
                conversation_id: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                message_count: row.get(5)?,
                is_archived: is_archived_int != 0,
                preview_text: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// Delete a conversation and all its messages
    pub async fn delete_conversation(&self, conversation_id: &str) -> SqliteResult<()> {
        let db = self.db.lock().await;

        // Delete messages first (foreign key)
        db.execute(
            "DELETE FROM meta_messages WHERE conversation_id = ?1",
            params![conversation_id],
        )?;

        // Delete conversation
        db.execute(
            "DELETE FROM meta_conversations WHERE conversation_id = ?1",
            params![conversation_id],
        )?;

        Ok(())
    }

    /// Rename a conversation
    pub async fn rename_conversation(
        &self,
        conversation_id: &str,
        new_title: &str,
    ) -> SqliteResult<()> {
        let db = self.db.lock().await;

        let now = chrono::Utc::now().timestamp_millis();

        db.execute(
            "UPDATE meta_conversations SET title = ?1, updated_at = ?2 WHERE conversation_id = ?3",
            params![new_title, now, conversation_id],
        )?;

        Ok(())
    }

    /// Archive/unarchive a conversation
    pub async fn set_archived(&self, conversation_id: &str, archived: bool) -> SqliteResult<()> {
        let db = self.db.lock().await;

        db.execute(
            "UPDATE meta_conversations SET is_archived = ?1 WHERE conversation_id = ?2",
            params![if archived { 1 } else { 0 }, conversation_id],
        )?;

        Ok(())
    }

    // ========================================================================
    // Message Operations
    // ========================================================================

    /// Insert a new message into a conversation
    pub async fn insert_message(&self, record: &MetaMessageRecord) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO meta_messages
             (conversation_id, message_index, role, content, image_data, tool_calls, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.conversation_id,
                record.message_index,
                record.role,
                record.content,
                record.image_data,
                record.tool_calls,
                record.timestamp
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    /// Get all messages for a conversation, ordered by message_index
    pub async fn get_messages(
        &self,
        conversation_id: &str,
    ) -> SqliteResult<Vec<MetaMessageRecord>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT id, conversation_id, message_index, role, content, image_data, tool_calls, timestamp
             FROM meta_messages
             WHERE conversation_id = ?1
             ORDER BY message_index ASC",
        )?;

        let rows = stmt.query_map(params![conversation_id], |row| {
            Ok(MetaMessageRecord {
                id: Some(row.get(0)?),
                conversation_id: row.get(1)?,
                message_index: row.get(2)?,
                role: row.get(3)?,
                content: row.get(4)?,
                image_data: row.get(5)?,
                tool_calls: row.get(6)?,
                timestamp: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// Get the count of messages in a conversation
    #[allow(dead_code)]
    pub async fn get_message_count(&self, conversation_id: &str) -> SqliteResult<u32> {
        let db = self.db.lock().await;

        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM meta_messages WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        )?;

        Ok(count as u32)
    }

    /// Update conversation metadata after adding a message
    pub async fn update_conversation_after_message(
        &self,
        conversation_id: &str,
        preview_text: Option<&str>,
        title: Option<&str>,
    ) -> SqliteResult<()> {
        let db = self.db.lock().await;

        let now = chrono::Utc::now().timestamp_millis();

        // Get current message count
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM meta_messages WHERE conversation_id = ?1",
            params![conversation_id],
            |row| row.get(0),
        )?;

        // Build dynamic update
        let mut updates: Vec<String> = vec![
            "updated_at = ?1".to_string(),
            "message_count = ?2".to_string(),
        ];
        let mut params_list: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(now), Box::new(count as u32)];

        if let Some(preview) = preview_text {
            updates.push("preview_text = ?3".to_string());
            params_list.push(Box::new(preview.to_string()));
        }

        if let Some(t) = title {
            let idx = params_list.len() + 1;
            updates.push(format!("title = ?{}", idx));
            params_list.push(Box::new(t.to_string()));
        }

        let query = format!(
            "UPDATE meta_conversations SET {} WHERE conversation_id = ?{}",
            updates.join(", "),
            params_list.len() + 1
        );
        params_list.push(Box::new(conversation_id.to_string()));

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params_list.iter().map(|p| p.as_ref()).collect();
        db.execute(&query, param_refs.as_slice())?;

        Ok(())
    }

    /// Cleanup old conversations (older than days_to_keep)
    pub async fn cleanup_old_conversations(&self, days_to_keep: i64) -> SqliteResult<usize> {
        let db = self.db.lock().await;

        let cutoff = chrono::Utc::now().timestamp_millis() - (days_to_keep * 24 * 60 * 60 * 1000);

        // Get IDs of old conversations
        let mut stmt = db.prepare(
            "SELECT conversation_id FROM meta_conversations WHERE updated_at < ?1 AND is_archived = 0",
        )?;
        let old_ids: Vec<String> = stmt
            .query_map(params![cutoff], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Delete messages and conversations
        for id in &old_ids {
            db.execute(
                "DELETE FROM meta_messages WHERE conversation_id = ?1",
                params![id],
            )?;
        }

        let deleted = db.execute(
            "DELETE FROM meta_conversations WHERE updated_at < ?1 AND is_archived = 0",
            params![cutoff],
        )?;

        Ok(deleted)
    }
}
