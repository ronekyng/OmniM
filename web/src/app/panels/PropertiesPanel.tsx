import { useCallback } from 'react';
import { ObjectPropertiesPanel } from '../components/ObjectPropertiesPanel';
import { useAppStore } from '../store/appStore';
import { useThreeViewer } from '../context/ThreeViewerContext';
import { validateForm } from '../../shared/types/creation';
import type { CreationForm } from '../../shared/types/creation';
import { useToastApi } from '../../shared/toast';

export function PropertiesPanel() {
  const toast = useToastApi();
  const viewerRef = useThreeViewer();
  const selectedIds = useAppStore((s) => s.selectedIds);
  const tree = useAppStore((s) => s.tree);
  const primaryMeshInfo = useAppStore((s) => s.primaryMeshInfo);
  const creatingForm = useAppStore((s) => s.creationForm);
  const creationPhase = useAppStore((s) => s.creationPhase);
  const panelMode = useAppStore((s) => s.panelMode);
  const setPanelMode = useAppStore((s) => s.setPanelMode);
  const setSelectedIds = useAppStore((s) => s.setSelectedIds);
  const creationDispatch = useAppStore((s) => s.creationDispatch);
  const setCreatingExtra = useAppStore((s) => s.setCreatingExtra);
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);

  const handleSaved = useCallback(() => {
    refresh();
    reloadScene();
  }, [refresh, reloadScene]);

  const handleClose = useCallback(() => {
    setSelectedIds([]);
    creationDispatch({ type: 'cancelCreation' });
  }, [setSelectedIds, creationDispatch]);

  const handleExtraChange = useCallback((extra: { ifcClass: string; desc: string }) => {
    setCreatingExtra(extra);
  }, [setCreatingExtra]);

  const handleChangeForm = useCallback((partial: Partial<CreationForm>) => {
    creationDispatch({ type: 'updateForm', partial });
  }, [creationDispatch]);

  const handleSubmitCreate = useCallback(() => {
    const form = useAppStore.getState().creationForm;
    if (!form) return;
    const result = validateForm(form);
    if (!result.valid) {
      toast.error('请修正表单错误');
      return;
    }
    creationDispatch({ type: 'submitCreation' });
  }, [creationDispatch, toast]);

  const handleToggleMode = useCallback(() => {
    setPanelMode(panelMode === 'floating' ? 'docked' : 'floating');
  }, [panelMode, setPanelMode]);

  const handleGetMaterial = useCallback((id: string) => {
    return viewerRef.current?.getMeshMaterial(id);
  }, [viewerRef]);

  const handleSetMaterial = useCallback((id: string, mat: { color: string; roughness: number; metalness: number }) => {
    viewerRef.current?.setMeshMaterial(id, mat);
  }, [viewerRef]);

  return (
    <ObjectPropertiesPanel
      selectedIds={selectedIds}
      tree={tree}
      meshInfo={primaryMeshInfo}
      creatingForm={creatingForm}
      creationPhase={creationPhase}
      onSaved={handleSaved}
      onClose={handleClose}
      onExtraChange={handleExtraChange}
      onChangeForm={handleChangeForm}
      onSubmitCreate={handleSubmitCreate}
      panelMode={panelMode}
      onToggleMode={handleToggleMode}
      onGetMaterial={handleGetMaterial}
      onSetMaterial={handleSetMaterial}
    />
  );
}