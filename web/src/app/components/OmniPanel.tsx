import { useState, useRef, useEffect, useCallback } from 'react';
import { api } from '../../shared/api';
import type { AmirChatResponse } from '../../shared/types';

const JSON_KEY_RE = /("(?:[^"\\]|\\.)*")\s*:/g;
const JSON_STRING_RE = /("(?:[^"\\]|\\.)*")/g;
const JSON_NUMBER_RE = /\b(-?\d+\.?\d*(?:[eE][+-]?\d+)?)\b/g;
const JSON_KEYWORD_RE = /\b(true|false|null)\b/g;

function highlightJson(text: string): string {
  const escaped = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  let highlighted = escaped;
  const tokens: { start: number; end: number; cls: string }[] = [];

  JSON_KEY_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = JSON_KEY_RE.exec(escaped)) !== null) {
    tokens.push({ start: m.index, end: m.index + m[1].length, cls: 'hl-key' });
  }

  JSON_STRING_RE.lastIndex = 0;
  while ((m = JSON_STRING_RE.exec(escaped)) !== null) {
    const isKey = tokens.some(t => t.start === m!.index && t.cls === 'hl-key');
    if (!isKey) {
      tokens.push({ start: m.index, end: m.index + m[1].length, cls: 'hl-string' });
    }
  }

  JSON_NUMBER_RE.lastIndex = 0;
  while ((m = JSON_NUMBER_RE.exec(escaped)) !== null) {
    tokens.push({ start: m.index, end: m.index + m[1].length, cls: 'hl-number' });
  }

  JSON_KEYWORD_RE.lastIndex = 0;
  while ((m = JSON_KEYWORD_RE.exec(escaped)) !== null) {
    tokens.push({ start: m.index, end: m.index + m[1].length, cls: 'hl-keyword' });
  }

  tokens.sort((a, b) => a.start - b.start);

  const parts: string[] = [];
  let pos = 0;
  for (const t of tokens) {
    if (t.start > pos) {
      parts.push(escaped.slice(pos, t.start));
    }
    parts.push(`<span class="${t.cls}">${escaped.slice(t.start, t.end)}</span>`);
    pos = t.end;
  }
  if (pos < escaped.length) {
    parts.push(escaped.slice(pos));
  }

  return parts.join('');
}

// ── JSON-aware tab completion ──

const JSON_FIELDS_MAP: Record<string, string[]> = {
  root: ['version', 'id', 'action', 'target', 'operations'],
  target: ['object', 'geometry_id', 'record_id', 'object_id', 'from_id', 'to_id', 'work_order_id'],
  object: ['object_id'],
  create_object: ['type', 'name', 'description', 'geometry', 'properties', 'parent_id'],
  delete_object: [],
  copy_object: ['source_object_id', 'new_name', 'parent_id'],
  move_object: ['position', 'rotation'],
  array: ['source_object_id', 'pattern', 'count_x', 'count_y', 'count_z', 'spacing_x', 'spacing_y', 'spacing_z'],
  boolean: ['object_ids', 'operation'],
  transform_geometry: ['new_geometry'],
  set_property: ['name', 'value', 'unit', 'property_set'],
  delete_property: ['name', 'property_set'],
  remove_property_set: ['property_set'],
  add_geometry: ['geometry'],
  remove_geometry: [],
  set_material: ['material_id'],
  create_lifecycle_record: ['phase', 'record_type', 'name', 'status', 'dates'],
  delete_lifecycle_record: [],
  link_record_to_object: [],
  update_schedule: ['planned_start', 'planned_end', 'actual_start', 'actual_end'],
  update_cost: ['estimated_cost', 'actual_cost', 'currency'],
  update_progress: ['percent', 'status', 'description'],
  create_work_order: ['title', 'description', 'assignee', 'priority', 'due_date'],
  update_work_order: ['status', 'description', 'assignee', 'priority'],
  add_containment: ['parent_id'],
  remove_containment: [],
  add_connection: ['target_object_id', 'relation_type'],
  remove_connection: [],
  rebuild_features: ['component_id'],
  geometry: ['operations'],
  box: ['length', 'width', 'height'],
  position: ['x', 'y', 'z'],
  rotation: ['x', 'y', 'z', 'angle'],
  dates: ['start_date', 'end_date'],
};

const ACTION_NAME_TO_NUM: Record<string, number> = {
  create_object: 1, delete_object: 3, copy_object: 4, move_object: 5,
  add_geometry: 10, remove_geometry: 12, transform_geometry: 13, boolean: 14,
  set_property: 20, delete_property: 21, remove_property_set: 23,
  add_containment: 30, remove_containment: 31, add_connection: 32, remove_connection: 33,
  array: 40, set_material: 50,
  create_lifecycle_record: 70, delete_lifecycle_record: 72, link_record_to_object: 73,
  update_schedule: 75, update_cost: 76, update_progress: 77,
  create_work_order: 80, update_work_order: 81,
  rebuild_features: 113,
};

const NUM_TO_ACTION_NAME: Record<number, string> = Object.fromEntries(
  Object.entries(ACTION_NAME_TO_NUM).map(([k, v]) => [v, k])
);

const OBJECT_TYPE_MAP: Record<string, { code: number; name: string }> = {
  wall: { code: 100, name: 'Wall' },
  column: { code: 101, name: 'Column' },
  slab: { code: 102, name: 'Slab' },
  beam: { code: 103, name: 'Beam' },
  door: { code: 104, name: 'Door' },
  window: { code: 105, name: 'Window' },
  stair: { code: 106, name: 'Stair' },
  railing: { code: 107, name: 'Railing' },
  roof: { code: 108, name: 'Roof' },
  floor: { code: 109, name: 'Floor' },
  ceiling: { code: 110, name: 'Ceiling' },
  furniture: { code: 111, name: 'Furniture' },
  pipe: { code: 112, name: 'Pipe' },
  duct: { code: 113, name: 'Duct' },
  cable: { code: 114, name: 'Cable' },
  equipment: { code: 115, name: 'Equipment' },
  site: { code: 200, name: 'Site' },
  building: { code: 201, name: 'Building' },
  storey: { code: 202, name: 'Storey' },
  space: { code: 203, name: 'Space' },
  zone: { code: 204, name: 'Zone' },
};

function getJsonCompletionContext(input: string): { field: string; prefix: string; context: string[] } {
  const trimmed = input.trim();
  if (!trimmed) return { field: '', prefix: '', context: JSON_FIELDS_MAP.root };

  if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) {
    return { field: '', prefix: '', context: [] };
  }

  // Find all field names in the current scope
  const allFieldMatches = [...trimmed.matchAll(/"(\w+)"\s*:/g)];
  const lastField = allFieldMatches.length > 0 ? allFieldMatches[allFieldMatches.length - 1][1] : '';

  // Check if the cursor is at a position where we expect a new field name
  // (after {, after comma, or at the start of operations array)
  const endsWithOpenBrace = trimmed.endsWith('{') || trimmed.endsWith('{');
  const endsWithComma = trimmed.endsWith(',');
  const endsWithQuote = trimmed.endsWith('"');

  // Determine what scope we're in
  const lastOpenBrace = trimmed.lastIndexOf('{');
  const beforeLastBrace = trimmed.slice(0, lastOpenBrace);
  const scopeFieldMatch = [...beforeLastBrace.matchAll(/"(\w+)"\s*:\s*\{/g)];
  const scopeField = scopeFieldMatch.length > 0 ? scopeFieldMatch[scopeFieldMatch.length - 1][1] : '';

  // Check if we're inside an operations array
  const inOperations = /operations\s*:\s*\[/.test(trimmed) && !trimmed.includes(']}');

  // Complete the current field's value
  if (lastField && (endsWithComma || endsWithOpenBrace || endsWithQuote)) {
    // We're between fields, suggest next field in the current scope
    const currentScope = inOperations ? scopeField || 'create_object' : scopeField || 'root';
    const fieldContext = JSON_FIELDS_MAP[currentScope];
    if (fieldContext) {
      return { field: '', prefix: '', context: fieldContext };
    }
    return { field: '', prefix: '', context: JSON_FIELDS_MAP.root };
  }

  // Complete the last field's value
  if (lastField) {
    const fields = JSON_FIELDS_MAP[lastField];
    if (fields) {
      return { field: lastField, prefix: '', context: fields };
    }
    if (lastField === 'type') {
      return { field: 'type', prefix: '', context: Object.keys(OBJECT_TYPE_MAP).map(k => `${k} (${OBJECT_TYPE_MAP[k].code})`) };
    }
    if (lastField === 'action') {
      return { field: 'action', prefix: '', context: Object.entries(ACTION_NAME_TO_NUM).map(([k, v]) => `${v} (${k.toUpperCase()})`) };
    }
    if (lastField === 'pattern') {
      return { field: 'pattern', prefix: '', context: ['linear', 'radial', 'rectangular'] };
    }
    if (lastField === 'operation') {
      return { field: 'operation', prefix: '', context: ['0 (union)', '1 (difference)', '2 (intersection)'] };
    }
    if (lastField === 'priority') {
      return { field: 'priority', prefix: '', context: ['low', 'medium', 'high', 'critical'] };
    }
    if (lastField === 'is_cut' || lastField === 'suppressed') {
      return { field: lastField, prefix: '', context: ['true', 'false'] };
    }
    if (lastField === 'status') {
      return { field: 'status', prefix: '', context: ['active', 'completed', 'pending', 'cancelled'] };
    }
    return { field: lastField, prefix: '', context: fields || [] };
  }

  // No field name found yet - suggest based on scope
  if (inOperations) {
    return { field: '', prefix: '', context: Object.keys(ACTION_NAME_TO_NUM) };
  }
  return { field: '', prefix: '', context: JSON_FIELDS_MAP.root };
}

function getJsonCompletions(input: string): string[] {
  const context = getJsonCompletionContext(input);
  if (context.context.length === 0) return [];

  const trimmed = input.trim();
  // Determine if we're completing a field name (after { or ,) or a value (after :)
  const lastChar = trimmed[trimmed.length - 1];
  const completingField = trimmed.endsWith('{') || trimmed.endsWith(',') || trimmed.endsWith('"') || trimmed.endsWith('"');

  let matches = context.context;
  if (context.field === '' && completingField) {
    // Completing a field name - wrap in quotes
    matches = context.context.map(m => `"${m}"`);
  }

  // Filter by prefix
  const lastWord = trimmed.match(/"([^"]*)$/)?.[1] || '';
  const prefix = lastWord.toLowerCase();
  return matches.filter(item => {
    const clean = item.replace(/^"|"$/g, '').toLowerCase();
    return clean.startsWith(prefix);
  });
}

function insertCompletion(input: string, completion: string): string {
  const trimmed = input.trimEnd();
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    // JSON mode: replace the last incomplete field name or value
    const lastQuote = trimmed.lastIndexOf('"');
    if (lastQuote >= 0) {
      const beforeQuote = trimmed.slice(0, lastQuote);
      const afterQuote = trimmed.slice(lastQuote + 1);
      // Check if there's a closing quote
      const closeQuote = afterQuote.indexOf('"');
      if (closeQuote >= 0) {
        // Replace the content between quotes
        return beforeQuote + completion + afterQuote.slice(closeQuote);
      }
      // No closing quote - replace from the last quote
      const colonOrComma = beforeQuote.match(/[(:,]\s*$/);
      if (colonOrComma) {
        return beforeQuote + completion;
      }
      return beforeQuote + completion;
    }
    // No quote - append completion with a space
    if (trimmed.endsWith(',') || trimmed.endsWith('{') || trimmed.endsWith(':')) {
      return trimmed + ' ' + completion;
    }
    return trimmed + completion;
  }
  // Plain text mode: replace the last word
  const lastSpace = trimmed.lastIndexOf(' ');
  if (lastSpace >= 0) {
    return trimmed.slice(0, lastSpace + 1) + completion;
  }
  return completion;
}

type HistoryEntry = {
  id: string;
  command: string;
  result: string;
  success: boolean;
  timestamp: number;
};

type ChatMessage = {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  commands?: unknown[];
};

export function OmniPanel({ onExecuted }: { onExecuted: () => void }) {
  const [showPanel, setShowPanel] = useState(false);
  const [activeTab, setActiveTab] = useState<'console' | 'ai'>('console');

  // ── Console state ──
  const [consoleInput, setConsoleInput] = useState('');
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [executing, setExecuting] = useState(false);
  const [consoleError, setConsoleError] = useState<string | null>(null);
  const [showReference, setShowReference] = useState(false);
  const historyEndRef = useRef<HTMLDivElement>(null);

  // ── Tab completion & history navigation ──
  const [historyNavIdx, setHistoryNavIdx] = useState<number | null>(null);
  const [completionItems, setCompletionItems] = useState<string[]>([]);
  const [completionIdx, setCompletionIdx] = useState(0);
  const [savedInput, setSavedInput] = useState('');
  const completionBaseRef = useRef('');

  const historyRawRef = useRef<string[]>([]);

  // ── AI Chat state ──
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [aiInput, setAiInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [aiError, setAiError] = useState<string | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [expandedCmd, setExpandedCmd] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    historyEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [history]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // ── Console logic ──

  async function execute() {
    if (!consoleInput.trim()) return;
    setConsoleError(null);
    setExecuting(true);

    try {
      let commandObj: unknown;
      try {
        commandObj = JSON.parse(consoleInput);
      } catch {
        const trimmed = consoleInput.trim();
        const parts = trimmed.split(/\s+/);
        if (parts.length >= 2 && parts[0].toUpperCase() === 'CREATE_OBJECT') {
          const objType = parts[1];
          const objName = parts.slice(2).join(' ') || objType;
          commandObj = {
            version: '1.0',
            id: 'console-' + Date.now(),
            action: 1,
            operations: [{
              create_object: {
                type: objType.toUpperCase(),
                name: objName,
                description: '',
                geometry: null,
                properties: [],
                parent_id: '',
              }
            }]
          };
        } else if (parts.length >= 1 && parts[0].toUpperCase() === 'UNDO') {
          const response = await api.undo();
historyRawRef.current = [...historyRawRef.current, consoleInput];
      setHistory(prev => [...prev, {
        id: Date.now().toString(),
        command: consoleInput,
        result: response.message,
        success: response.success,
        timestamp: Date.now(),
      }]);
      onExecuted();
      setExecuting(false);
      return;
    } else if (parts.length >= 1 && parts[0].toUpperCase() === 'REDO') {
          const response = await api.redo();
          historyRawRef.current = [...historyRawRef.current, consoleInput];
          setHistory(prev => [...prev, {
            id: Date.now().toString(),
            command: consoleInput,
            result: response.message,
            success: response.success,
            timestamp: Date.now(),
          }]);
          onExecuted();
          setExecuting(false);
          return;
        } else {
          throw new Error('Invalid JSON or command. Use JSON format or simple: CREATE_OBJECT <Type> <Name>');
        }
      }

      if (typeof commandObj !== 'object' || commandObj === null) {
        throw new Error('Invalid JSON: expected an object');
      }

      const response = await api.executeOmcom(commandObj);
      historyRawRef.current = [...historyRawRef.current, consoleInput];
      setHistory(prev => [...prev, {
        id: Date.now().toString(),
        command: consoleInput,
        result: response.message || (response.success ? 'Success' : 'Failed'),
        success: response.success,
        timestamp: Date.now(),
      }]);
      if (response.success) {
        setConsoleInput('');
        onExecuted();
      }
    } catch (err) {
      setConsoleError(err instanceof Error ? err.message : String(err));
    }

    setExecuting(false);
  }

  function handleConsoleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      if (completionItems.length > 0) {
        applyCompletion(completionItems[completionIdx]);
        return;
      }
      execute();
      return;
    }

    if (e.key === 'Escape') {
      if (completionItems.length > 0) {
        setCompletionItems([]);
        return;
      }
      setShowPanel(false);
      return;
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      if (completionItems.length > 0) {
        const next = (completionIdx + 1) % completionItems.length;
        setCompletionIdx(next);
        setConsoleInput(insertCompletion(completionBaseRef.current, completionItems[next]));
        return;
      }

      const input = consoleInput.trim();

      // Try JSON-aware completion first
      const jsonMatches = getJsonCompletions(input);
      if (jsonMatches.length > 0) {
        completionBaseRef.current = input;
        if (jsonMatches.length === 1) {
          setConsoleInput(insertCompletion(input, jsonMatches[0]));
          setCompletionItems([]);
        } else {
          const sorted = jsonMatches.sort();
          setCompletionItems(sorted);
          setCompletionIdx(0);
          setConsoleInput(insertCompletion(input, sorted[0]));
        }
        return;
      }

      // Fall back to command name completion (plain text mode)
      const allNames = commandGroups.flatMap(g => g.commands.map(c => c.name));
      const matches = allNames.filter(n => n.toLowerCase().startsWith(input.toLowerCase()));
      if (matches.length === 1) {
        setConsoleInput(matches[0]);
        setCompletionItems([]);
      } else if (matches.length > 1) {
        const sorted = matches.sort();
        completionBaseRef.current = input;
        setCompletionItems(sorted);
        setCompletionIdx(0);
        setConsoleInput(sorted[0]);
      }
      return;
    }

    if (e.key === 'ArrowUp') {
      if (completionItems.length > 0) {
        e.preventDefault();
        const prev = (completionIdx - 1 + completionItems.length) % completionItems.length;
        setCompletionIdx(prev);
        setConsoleInput(completionItems[prev]);
        return;
      }
      const raw = historyRawRef.current;
      if (raw.length === 0) return;
      e.preventDefault();
      if (historyNavIdx === null) {
        setSavedInput(consoleInput);
        setHistoryNavIdx(raw.length - 1);
        setConsoleInput(raw[raw.length - 1]);
      } else {
        const next = Math.max(0, historyNavIdx - 1);
        setHistoryNavIdx(next);
        setConsoleInput(raw[next]);
      }
      return;
    }

    if (e.key === 'ArrowDown') {
      if (completionItems.length > 0) {
        e.preventDefault();
        const next = (completionIdx + 1) % completionItems.length;
        setCompletionIdx(next);
        setConsoleInput(completionItems[next]);
        return;
      }
      if (historyNavIdx === null) return;
      e.preventDefault();
      const raw = historyRawRef.current;
      if (historyNavIdx < raw.length - 1) {
        const next = historyNavIdx + 1;
        setHistoryNavIdx(next);
        setConsoleInput(raw[next]);
      } else {
        setHistoryNavIdx(null);
        setConsoleInput(savedInput);
      }
      return;
    }

    setCompletionItems([]);
    setHistoryNavIdx(null);
  }

  function applyCompletion(item: string) {
    setConsoleInput(item);
    setCompletionItems([]);
  }

  function clearHistory() {
    setHistory([]);
  }

  function loadTemplate(template: string) {
    setConsoleInput(template);
  }

  const templates = [
    { label: 'Create Wall', value: JSON.stringify({
      version: '1.0', id: 'tpl-' + Date.now(), action: 1,
      operations: [{ create_object: { type: 100, name: 'Wall-Template', description: '', geometry: { operations: [{ box: { length: 5.0, width: 0.3, height: 3.0 } }] }, properties: [], parent_id: '' } }]
    }, null, 2) },
    { label: 'Copy Object', value: JSON.stringify({
      version: '1.0', id: 'tpl-' + Date.now(), action: 4, target: { object: { object_id: '<ID>' } },
      operations: [{ copy_object: { source_object_id: '<ID>', new_name: 'Copy', parent_id: '' } }]
    }, null, 2) },
    { label: 'Array', value: JSON.stringify({
      version: '1.0', id: 'tpl-' + Date.now(), action: 40,
      operations: [{ array: { source_object_id: '<ID>', pattern: 'linear', count_x: 5, count_y: 1, count_z: 1, spacing_x: 2.0, spacing_y: 0, spacing_z: 0 } }]
    }, null, 2) },
  ];

  const commandGroups = [
    {
      group: 'Object Operations',
      commands: [
        { name: 'CreateObject', action: 1, syntax: 'Action: CREATE_OBJECT\nOp: create_object { type, name, description, geometry, properties, parent_id }', desc: 'Create a new object with optional geometry and properties.' },
        { name: 'DeleteObject', action: 3, syntax: 'Action: DELETE_OBJECT\nTarget: { object_id }\nOp: delete_object', desc: 'Delete an object by ID.' },
        { name: 'CopyObject', action: 4, syntax: 'Action: COPY_OBJECT\nTarget: { object_id }\nOp: copy_object { source_object_id, new_name, parent_id }', desc: 'Create a copy of an existing object.' },
        { name: 'MoveObject', action: 5, syntax: 'Action: MOVE_OBJECT\nTarget: { object_id }\nOp: move_object { position, rotation }', desc: 'Move/rotate an object to a new position.' },
        { name: 'Array', action: 40, syntax: 'Action: ARRAY\nOp: array { source_object_id, pattern, count_x/y/z, spacing_x/y/z }', desc: 'Create a linear/radial array of copies.' },
        { name: 'Boolean', action: 14, syntax: 'Action: BOOLEAN\nOp: boolean { object_ids, operation (0=union/1=difference/2=intersection) }', desc: 'Perform boolean operation on multiple objects.' },
        { name: 'TransformGeometry', action: 13, syntax: 'Action: TRANSFORM_GEOMETRY\nTarget: { object_id }\nOp: transform_geometry { new_geometry }', desc: 'Replace the geometry of an object.' },
      ],
    },
    {
      group: 'Property Operations',
      commands: [
        { name: 'SetProperty', action: 20, syntax: 'Action: SET_PROPERTY\nTarget: { object_id }\nOp: set_property { name, value, unit, property_set }', desc: 'Set a property on an object.' },
        { name: 'DeleteProperty', action: 21, syntax: 'Action: DELETE_PROPERTY\nTarget: { object_id }\nOp: delete_property { name, property_set }', desc: 'Delete a property from an object.' },
        { name: 'RemovePropertySet', action: 23, syntax: 'Action: REMOVE_PROPERTY_SET\nTarget: { object_id }\nOp: remove_property_set { property_set }', desc: 'Remove an entire property set.' },
      ],
    },
    {
      group: 'Geometry & Material',
      commands: [
        { name: 'AddGeometry', action: 10, syntax: 'Action: ADD_GEOMETRY\nTarget: { object_id }\nOp: create_geometry { geometry }', desc: 'Add a geometry representation to an object.' },
        { name: 'RemoveGeometry', action: 12, syntax: 'Action: REMOVE_GEOMETRY\nTarget: { geometry_id }\nOp: remove_geometry', desc: 'Remove a geometry representation by ID.' },
        { name: 'AssignMaterial', action: 50, syntax: 'Action: ASSIGN_MATERIAL\nTarget: { object_id }\nOp: set_material { material_id }', desc: 'Assign a material to an object.' },
      ],
    },
    {
      group: 'Lifecycle',
      commands: [
        { name: 'AddLifecycleRecord', action: 70, syntax: 'Action: ADD_LIFECYCLE\nOp: create_lifecycle_record { phase, record_type, name, status, dates }', desc: 'Add a lifecycle record (e.g. design review, construction log).' },
        { name: 'DeleteLifecycleRecord', action: 72, syntax: 'Action: DELETE_LIFECYCLE\nTarget: { record_id }\nOp: delete_lifecycle_record', desc: 'Delete a lifecycle record by ID.' },
        { name: 'LinkRecordToObject', action: 73, syntax: 'Action: LINK_RECORD\nTarget: { object_id, record_id }\nOp: link_record_to_object', desc: 'Link a lifecycle record to an object.' },
      ],
    },
    {
      group: 'Cost & Schedule',
      commands: [
        { name: 'UpdateSchedule', action: 75, syntax: 'Action: UPDATE_SCHEDULE\nTarget: { object_id }\nOp: update_schedule { planned_start, planned_end, actual_start, actual_end }', desc: 'Update schedule dates for an object.' },
        { name: 'UpdateCost', action: 76, syntax: 'Action: UPDATE_COST\nTarget: { object_id }\nOp: update_cost { estimated_cost, actual_cost, currency }', desc: 'Update cost information for an object.' },
        { name: 'UpdateProgress', action: 77, syntax: 'Action: UPDATE_PROGRESS\nTarget: { record_id }\nOp: update_progress { percent, status, description }', desc: 'Update progress percentage and status.' },
        { name: 'CreateWorkOrder', action: 80, syntax: 'Action: CREATE_WORK_ORDER\nOp: create_work_order { title, description, assignee, priority, due_date }', desc: 'Create a new work order.' },
        { name: 'UpdateWorkOrder', action: 81, syntax: 'Action: UPDATE_WORK_ORDER\nTarget: { work_order_id }\nOp: update_work_order { status, description, assignee, priority }', desc: 'Update an existing work order.' },
      ],
    },
    {
      group: 'Relations',
      commands: [
        { name: 'SetParent', action: 30, syntax: 'Action: SET_PARENT\nTarget: { object_id }\nOp: add_containment { parent_id }', desc: 'Set parent-child containment.' },
        { name: 'RemoveContainment', action: 31, syntax: 'Action: REMOVE_CONTAINMENT\nTarget: { object_id }\nOp: remove_containment', desc: 'Remove an object from its parent.' },
        { name: 'Connect', action: 32, syntax: 'Action: CONNECT\nTarget: { object_id }\nOp: add_connection { target_object_id, relation_type }', desc: 'Create a connection between two objects.' },
        { name: 'RemoveConnection', action: 33, syntax: 'Action: REMOVE_CONNECTION\nTarget: { from_id, to_id }\nOp: remove_connection', desc: 'Remove a connection between two objects.' },
      ],
    },
    {
      group: 'System',
      commands: [
        { name: 'Undo', action: -1, syntax: '"UNDO" (plain text) or undo N', desc: 'Undo the last N operations (plain text command, no JSON needed).' },
        { name: 'Redo', action: -2, syntax: '"REDO" (plain text)', desc: 'Redo the last undone operation (plain text command).' },
      ],
    },
  ];

  // ── AI Chat logic ──

  async function send() {
    const text = aiInput.trim();
    if (!text || loading) return;
    setAiError(null);

    const userMsg: ChatMessage = {
      id: `user-${Date.now()}`,
      role: 'user',
      content: text,
    };
    setMessages(prev => [...prev, userMsg]);
    setAiInput('');
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
        if (execError) setAiError(execError);
        onExecuted();
      }
    } catch (err) {
      setAiError(err instanceof Error ? err.message : String(err));
    }

    setLoading(false);
  }

  function handleAiKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
    if (e.key === 'Escape') {
      setShowPanel(false);
    }
  }

  function newConversation() {
    setMessages([]);
    setSessionId(null);
    setAiError(null);
    setExpandedCmd(null);
  }

  function toggleCommands(id: string) {
    setExpandedCmd(expandedCmd === id ? null : id);
  }

  function togglePanel(tab: 'console' | 'ai') {
    if (showPanel && activeTab === tab) {
      setShowPanel(false);
    } else {
      setShowPanel(true);
      setActiveTab(tab);
    }
    setConsoleError(null);
    setAiError(null);
  }

  return (
    <div className="omni-container">
      {showPanel && (
        <div className="omni-panel">
          <div className="omni-tabs">
            <button
              className={`omni-tab ${activeTab === 'console' ? 'active' : ''}`}
              onClick={() => setActiveTab('console')}
            >
              {'>_ Console'}
            </button>
            <button
              className={`omni-tab ${activeTab === 'ai' ? 'active' : ''}`}
              onClick={() => setActiveTab('ai')}
            >
              AI
            </button>
            <div className="omni-tab-spacer" />
            <button className="omni-btn close" onClick={() => setShowPanel(false)} title="Close">&times;</button>
          </div>

          {activeTab === 'console' ? (
            <div className="omni-body omni-body-console">
              {showReference ? (
                <div className="console-reference">
                  {commandGroups.map(group => (
                    <div key={group.group} className="ref-group">
                      <h4 className="ref-group-title">{group.group}</h4>
                      {group.commands.map(cmd => (
                        <div key={cmd.name} className="ref-command" onClick={() => {
                          const tpl = JSON.stringify({
                            version: '1.0',
                            id: 'ref-' + Date.now(),
                            action: cmd.action,
                            target: cmd.name === 'Undo' || cmd.name === 'Redo' ? undefined : { object: { object_id: '' } },
                            operations: [{ [cmd.name.charAt(0).toLowerCase() + cmd.name.slice(1)]: {} }],
                          }, null, 2);
                          setConsoleInput(tpl);
                        }}>
                          <div className="ref-cmd-name">{cmd.name}</div>
                          <div className="ref-cmd-desc">{cmd.desc}</div>
                          <pre className="ref-cmd-syntax">{cmd.syntax}</pre>
                        </div>
                      ))}
                    </div>
                  ))}
                </div>
              ) : (
                <>
                  <div className="console-history">
                    {history.length === 0 && (
                      <div className="console-empty">
                        Type a JSON Omcom command or use templates below. Press Ctrl+Enter to execute.
                      </div>
                    )}
                    {history.map(entry => (
                      <div key={entry.id} className={`console-entry ${entry.success ? 'success' : 'error'}`}>
                        <div className="entry-cmd">&gt; {entry.command}</div>
                        <div className="entry-result">{entry.result}</div>
                      </div>
                    ))}
                    <div ref={historyEndRef} />
                  </div>

                  <div className="console-templates">
                    {templates.map(t => (
                      <button key={t.label} onClick={() => loadTemplate(t.value)} className="template-btn">
                        {t.label}
                      </button>
                    ))}
                  </div>

                  <div className="console-input-row">
                    <div className="console-input-wrap">
                      <textarea
                        className="console-input"
                        value={consoleInput}
                        onChange={e => {
                          setConsoleInput(e.target.value);
                          setCompletionItems([]);
                          setHistoryNavIdx(null);
                        }}
                        onKeyDown={handleConsoleKeyDown}
                        placeholder='Enter JSON Omcom command, e.g. {"action":1,"operations":[{"create_object":{"type":100,"name":"Wall-1"}}]}'
                        rows={3}
                      />
                      <pre
                        className="console-input-highlight"
                        aria-hidden="true"
                        dangerouslySetInnerHTML={{
                          __html: consoleInput ? highlightJson(consoleInput) : ''
                        }}
                      />
                      {completionItems.length > 0 && (
                        <div className="console-completion-popup">
                          {completionItems.map((item, i) => (
                            <div
                              key={item}
                              className={`console-completion-item ${i === completionIdx ? 'selected' : ''}`}
                              onMouseDown={(e) => {
                                e.preventDefault();
                                applyCompletion(item);
                              }}
                            >
                              {item}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                    <button
                      className="console-execute"
                      onClick={execute}
                      disabled={executing || !consoleInput.trim()}
                    >
                      {executing ? '...' : 'Execute'}
                    </button>
                  </div>

                  {consoleError && <div className="console-error">{consoleError}</div>}
                </>
              )}
              <div className="omni-footer-actions">
                <button className="omni-btn" onClick={() => setShowReference(!showReference)}>
                  {showReference ? 'Console' : 'Reference'}
                </button>
                <button className="omni-btn" onClick={clearHistory}>Clear</button>
              </div>
            </div>
          ) : (
            <div className="omni-body">
              <div className="aichat-body">
                {messages.length === 0 && !aiError && (
                  <div className="aichat-empty">
                    Ask AMIR to help with your model.
                    <div className="aichat-hints">
                      <button className="hint-chip" onClick={() => setAiInput('create a wall')}>Create a wall</button>
                      <button className="hint-chip" onClick={() => setAiInput('create a column')}>Create a column</button>
                      <button className="hint-chip" onClick={() => setAiInput('undo')}>Undo</button>
                      <button className="hint-chip" onClick={() => setAiInput('how many objects do I have?')}>Query model</button>
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

                {aiError && <div className="aichat-error">{aiError}</div>}
                <div ref={messagesEndRef} />
              </div>

              <div className="aichat-input-row">
                <textarea
                  ref={inputRef}
                  className="aichat-input"
                  value={aiInput}
                  onChange={e => setAiInput(e.target.value)}
                  onKeyDown={handleAiKeyDown}
                  placeholder="Ask AMIR something…"
                  rows={2}
                  disabled={loading}
                />
                <button
                  className="aichat-send"
                  onClick={send}
                  disabled={loading || !aiInput.trim()}
                >
                  {loading ? '…' : 'Send'}
                </button>
              </div>

              <div className="omni-footer-actions">
                <button className="omni-btn" onClick={newConversation}>New</button>
              </div>
            </div>
          )}
        </div>
      )}

      <div className="omni-toggles">
        <button
          className={`omni-toggle ${showPanel && activeTab === 'console' ? 'active' : ''}`}
          onClick={() => togglePanel('console')}
          title="Omcom Console (Ctrl+`)"
        >
          {'>_ Console'}
        </button>
        <button
          className={`omni-toggle ${showPanel && activeTab === 'ai' ? 'active' : ''}`}
          onClick={() => togglePanel('ai')}
          title="AI Assistant"
        >
          AI
        </button>
      </div>
    </div>
  );
}