import { useState, useRef, useEffect } from 'react';
import { api } from '../../shared/api';
import type { AmirChatResponse } from '../../shared/types';

type ChatMessage = {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  commands?: unknown[];
};

export function AiChat({ onExecuted }: { onExecuted: () => void }) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAiChat, setShowAiChat] = useState(false);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [expandedCmd, setExpandedCmd] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  async function send() {
    const text = input.trim();
    if (!text || loading) return;
    setError(null);

    const userMsg: ChatMessage = {
      id: `user-${Date.now()}`,
      role: 'user',
      content: text,
    };
    setMessages(prev => [...prev, userMsg]);
    setInput('');
    setLoading(true);

    try {
      const response: AmirChatResponse = await api.amirChat({
        session_id: sessionId,
        text,
        selected_ids: null,
      });

      if (!sessionId) {
        setSessionId(response.session_id);
      }

      const assistantMsg: ChatMessage = {
        id: `assistant-${Date.now()}`,
        role: 'assistant',
        content: response.reply || response.summary,
        commands: response.commands?.length ? response.commands : undefined,
      };
      setMessages(prev => [...prev, assistantMsg]);

      if (response.commands?.length) {
        let execError: string | null = null;
        for (const cmd of response.commands) {
          try {
            await api.executeOmcom(cmd);
          } catch (e) {
            execError = `Command failed: ${e instanceof Error ? e.message : String(e)}`;
            break;
          }
        }
        if (execError) setError(execError);
        onExecuted();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }

    setLoading(false);
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
    if (e.key === 'Escape') {
      setShowAiChat(false);
    }
  }

  function newConversation() {
    setMessages([]);
    setSessionId(null);
    setError(null);
    setExpandedCmd(null);
  }

  function toggleCommands(id: string) {
    setExpandedCmd(expandedCmd === id ? null : id);
  }

  return (
    <div className="aichat-container">
      {showAiChat && (
        <div className="aichat-panel">
          <div className="aichat-header">
            <span>AI Assistant</span>
            <div className="aichat-actions">
              <button onClick={newConversation} className="aichat-btn" title="New conversation">New</button>
              <button onClick={() => setShowAiChat(false)} className="aichat-btn close" title="Close">&times;</button>
            </div>
          </div>

          <div className="aichat-body">
            {messages.length === 0 && !error && (
              <div className="aichat-empty">
                Ask AMIR to help with your model.
                <div className="aichat-hints">
                  <button className="hint-chip" onClick={() => setInput('create a wall')}>Create a wall</button>
                  <button className="hint-chip" onClick={() => setInput('create a column')}>Create a column</button>
                  <button className="hint-chip" onClick={() => setInput('undo')}>Undo</button>
                  <button className="hint-chip" onClick={() => setInput('how many objects do I have?')}>Query model</button>
                </div>
              </div>
            )}

            {messages.map(msg => (
              <div key={msg.id} className={`aichat-message ${msg.role}`}>
                <div className="aichat-avatar">{msg.role === 'user' ? 'You' : 'AI'}</div>
                <div className="aichat-bubble">
                  <div className="aichat-content">{msg.content}</div>
                  {msg.commands && msg.commands.length > 0 && (
                    <div className="aichat-commands">
                      <button
                        className="aichat-cmd-toggle"
                        onClick={() => toggleCommands(msg.id)}
                      >
                        {expandedCmd === msg.id ? 'Hide' : 'Show'} commands ({msg.commands.length})
                      </button>
                      {expandedCmd === msg.id && (
                        <pre className="aichat-cmd-json">
                          {JSON.stringify(msg.commands, null, 2)}
                        </pre>
                      )}
                    </div>
                  )}
                </div>
              </div>
            ))}

            {loading && (
              <div className="aichat-message assistant">
                <div className="aichat-avatar">AI</div>
                <div className="aichat-bubble">
                  <div className="aichat-loading">
                    <span className="aichat-dot" />
                    <span className="aichat-dot" />
                    <span className="aichat-dot" />
                  </div>
                </div>
              </div>
            )}

            {error && <div className="aichat-error">{error}</div>}
            <div ref={messagesEndRef} />
          </div>

          <div className="aichat-input-row">
            <textarea
              ref={inputRef}
              className="aichat-input"
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ask AMIR something…"
              rows={2}
              disabled={loading}
            />
            <button
              className="aichat-send"
              onClick={send}
              disabled={loading || !input.trim()}
            >
              {loading ? '…' : 'Send'}
            </button>
          </div>
        </div>
      )}

      <button
        className={`aichat-toggle ${showAiChat ? 'active' : ''}`}
        onClick={() => setShowAiChat(!showAiChat)}
        title="AI Assistant"
      >
        {showAiChat ? '✕' : 'AI'}
      </button>
    </div>
  );
}
