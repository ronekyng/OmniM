import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from './api';
import type { StatsResponse, TreeNode, UndoStatusResponse, MeshResponse, OmniObject } from './types';

// ── Query keys ──
export const queryKeys = {
  stats: ['stats'] as const,
  tree: ['tree'] as const,
  undoStatus: ['undo-status'] as const,
  mesh: ['mesh'] as const,
  objects: ['objects'] as const,
  object: (id: string) => ['objects', id] as const,
  sketches: (componentId?: string) => ['sketches', componentId] as const,
  sketch: (id: string) => ['sketches', id] as const,
  features: (componentId: string) => ['features', componentId] as const,
  catalog: ['catalog'] as const,
  series: ['series'] as const,
  components: ['components'] as const,
  component: (id: string) => ['components', id] as const,
};

// ── Queries ──
export function useStats() {
  return useQuery<StatsResponse>({
    queryKey: queryKeys.stats,
    queryFn: () => api.stats(),
    staleTime: 5000,
  });
}

export function useTree() {
  return useQuery<TreeNode[]>({
    queryKey: queryKeys.tree,
    queryFn: () => api.tree(),
    staleTime: 5000,
  });
}

export function useUndoStatus() {
  return useQuery<UndoStatusResponse>({
    queryKey: queryKeys.undoStatus,
    queryFn: () => api.undoStatus(),
    staleTime: 1000,
  });
}

export function useMesh() {
  return useQuery<MeshResponse>({
    queryKey: queryKeys.mesh,
    queryFn: () => api.mesh(),
    staleTime: 0,
  });
}

export function useObject(id: string) {
  return useQuery<OmniObject>({
    queryKey: queryKeys.object(id),
    queryFn: () => api.object(id),
    enabled: !!id,
  });
}

// ── Mutations ──
export function useDeleteObject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteObject(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tree });
      qc.invalidateQueries({ queryKey: queryKeys.stats });
      qc.invalidateQueries({ queryKey: queryKeys.mesh });
    },
  });
}

export function useCreateObject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: unknown) => api.createObject(body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tree });
      qc.invalidateQueries({ queryKey: queryKeys.stats });
      qc.invalidateQueries({ queryKey: queryKeys.mesh });
    },
  });
}

export function useDuplicateObject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ sourceId, newName, parentId }: { sourceId: string; newName: string; parentId: string }) =>
      api.duplicateObject(sourceId, newName, parentId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tree });
      qc.invalidateQueries({ queryKey: queryKeys.stats });
      qc.invalidateQueries({ queryKey: queryKeys.mesh });
    },
  });
}

export function useSetParent() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ childId, newParentId }: { childId: string; newParentId: string | null }) =>
      api.setParent(childId, newParentId),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tree });
    },
  });
}

export function useUndo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.undo(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.undoStatus });
      qc.invalidateQueries({ queryKey: queryKeys.tree });
      qc.invalidateQueries({ queryKey: queryKeys.stats });
      qc.invalidateQueries({ queryKey: queryKeys.mesh });
    },
  });
}

export function useRedo() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.redo(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.undoStatus });
      qc.invalidateQueries({ queryKey: queryKeys.tree });
      qc.invalidateQueries({ queryKey: queryKeys.stats });
      qc.invalidateQueries({ queryKey: queryKeys.mesh });
    },
  });
}

export function useExecuteOmcom() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: unknown) => api.executeOmcom(body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tree });
      qc.invalidateQueries({ queryKey: queryKeys.stats });
      qc.invalidateQueries({ queryKey: queryKeys.mesh });
    },
  });
}