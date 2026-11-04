// zh-CN 词典 — I18N_L10N §2：真值来源（每种语言一个文件）。
// 键命名：`<area>.<component>.<element>`，扁平点分（§7）。
// 中文无复数范畴，但必须声明 `other`（§4）。
export const zhCN = {
  // —— shell.titlebar ——
  'shell.titlebar.workspace_suffix': ' 工作区',
  'shell.titlebar.back_to_projects': '返回项目列表',
  'shell.titlebar.dirty_hint': '已编辑（服务器自动保存），快照待显式保存',
  'shell.breadcrumb.workspace': '工作区',
  'shell.breadcrumb.modeling': '建模',
  'shell.breadcrumb.transform': '变换',
  'shell.breadcrumb.create': 'Create',
  'shell.theme.switch_to_light': '切换到浅色主题',
  'shell.theme.switch_to_dark': '切换到深色主题',
  'shell.action.save': '保存',
  'shell.action.save_shortcut': '保存 (Ctrl/Cmd+S)',
  'shell.action.save_as': '另存为',
  'shell.action.reset_layout': '恢复默认布局',
  'shell.language.switcher': '界面语言',
  'shell.saveas.title': '项目另存为',
  'shell.saveas.hint': '服务器上的路径，相对或绝对，必须以',
  'shell.saveas.hint_ext': ' 结尾。',
  'shell.saveas.path_label': '项目文件路径',
  'shell.saveas.cancel': '取消',
  'shell.meta.button': '项目设置',
  'shell.meta.title': '项目设置',
  'shell.meta.name': '名称',
  'shell.meta.description': '描述',
  'shell.meta.author': '作者 / 组织',
  'shell.meta.units': '单位制（仅显示）',
  'shell.statusbar.no_selection': '未选中',
  'shell.statusbar.one_selected': '已选中 1 个',
  'shell.statusbar.n_selected': '已选中 {count} 个',
  'shell.statusbar.coords_hint': '光标世界坐标（m）',
  'shell.statusbar.units_hint': '显示单位',
  'shell.save.clean': '已保存',
  'shell.save.clean_at': '已保存 {time}',
  'shell.save.dirty_hint': '服务器自动保存 · Ctrl+S 显式快照',
  'shell.save.saving': '保存中…',
  'shell.save.failed': '保存失败',
  'shell.meta.readonly': '存储几何始终为米制；单位只为显示；重命名会在项目根目录内同步改名文件。',
  'shell.leave.title': '不保存就离开吗？',
  'shell.leave.message': '工作区有本次会话中未显式保存的更改。',
  'shell.leave.confirm': '离开',
  'shell.leave.cancel': '留下',
  'shell.titlebar.doc_prefix': '— OmniM',
  // —— 复数（§4：中文只需 other） ——
  'common.plural.objects.other': '{count} 个对象',
  // —— 语言自命名 ——
  'locale.zh-CN': '中文',
  'locale.en-US': '英文',
} as const;

export type MessageKey = keyof typeof zhCN;
