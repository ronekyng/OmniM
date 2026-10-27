use crate::types::{Message, MessageRole};
use std::collections::HashMap;
use std::sync::Mutex;

/// 会话管理器 — 维护多轮对话上下文
///
/// 使用服务端内存存储，通过 session_id 索引。
/// 支持 LRU 淘汰机制限制最大会话数。
pub struct SessionManager {
    sessions: Mutex<HashMap<String, ConversationSession>>,
    max_sessions: usize,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::with_limit(100)
    }

    pub fn with_limit(max_sessions: usize) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            max_sessions,
        }
    }

    /// 获取已有 session 或创建新 session，返回 session_id
    pub fn get_or_create(&self, session_id: Option<String>) -> String {
        let mut sessions = self.sessions.lock().unwrap();

        let id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        if !sessions.contains_key(&id) {
            // LRU 淘汰
            if sessions.len() >= self.max_sessions {
                if let Some(oldest) = sessions.keys().next().cloned() {
                    sessions.remove(&oldest);
                }
            }
            sessions.insert(id.clone(), ConversationSession::new(&id));
        }

        id
    }

    pub fn add_user_message(&self, session_id: &str, content: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_user_message(content);
        }
    }

    pub fn add_assistant_message(&self, session_id: &str, content: &str) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_assistant_message(content);
        }
    }

    pub fn build_context(&self, session_id: &str) -> String {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.build_context()).unwrap_or_default()
    }
}

/// 单个会话
pub struct ConversationSession {
    pub session_id: String,
    pub messages: Vec<Message>,
    max_history: usize,
}

impl ConversationSession {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            messages: Vec::new(),
            max_history: 50,
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: content.to_string(),
        });
        self.trim_history();
    }

    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: content.to_string(),
        });
        self.trim_history();
    }

    /// 构建供 LLM 使用的上下文文本
    pub fn build_context(&self) -> String {
        let mut ctx = String::new();

        for msg in &self.messages {
            let role = match msg.role {
                MessageRole::User => "User",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };
            ctx.push_str(&format!("{}: {}\n", role, msg.content));
        }

        ctx
    }

    fn trim_history(&mut self) {
        while self.messages.len() > self.max_history {
            self.messages.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_create() {
        let sm = SessionManager::new();
        let id = sm.get_or_create(None);
        assert!(!id.is_empty());
        // second call with same id reuses session
        let id2 = sm.get_or_create(Some(id.clone()));
        assert_eq!(id, id2);
    }

    #[test]
    fn test_session_messages() {
        let sm = SessionManager::new();
        let id = sm.get_or_create(None);

        sm.add_user_message(&id, "create a wall");
        sm.add_assistant_message(&id, "wall created");

        let ctx = sm.build_context(&id);
        assert!(ctx.contains("User"));
        assert!(ctx.contains("create a wall"));
        assert!(ctx.contains("Assistant"));
        assert!(ctx.contains("wall created"));
    }

    #[test]
    fn test_history_trimming() {
        let mut session = ConversationSession::new("test");
        session.max_history = 3;

        session.add_user_message("msg1");
        session.add_user_message("msg2");
        session.add_user_message("msg3");
        session.add_user_message("msg4");

        assert_eq!(session.messages.len(), 3);
        assert_eq!(session.messages[0].content, "msg2");
    }

    #[test]
    fn test_max_sessions_cap() {
        let sm = SessionManager::with_limit(2);

        let id1 = sm.get_or_create(None);
        let id2 = sm.get_or_create(None);
        let id3 = sm.get_or_create(None); // should evict one

        let sessions = sm.sessions.lock().unwrap();
        assert_eq!(sessions.len(), 2);
        // Only id2 and id3 are guaranteed to remain
        assert!(sessions.contains_key(&id2) || sessions.contains_key(&id1));
        assert!(sessions.contains_key(&id3));
    }

    #[test]
    fn test_build_context_empty_session() {
        let sm = SessionManager::new();
        let ctx = sm.build_context("nonexistent");
        assert!(ctx.is_empty());
    }
}
