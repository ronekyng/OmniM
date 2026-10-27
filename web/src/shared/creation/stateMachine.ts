import type { CreationForm, CreationPhase, ObjectType, Point3D } from '../types/creation';
import { createInitialForm, applyPointsToForm } from '../types/creation';

export type CreationState = {
  phase: CreationPhase;
  objectType: ObjectType | null;
  points: Point3D[];
  form: CreationForm | null;
  error: string | null;
};

export const initialCreationState: CreationState = {
  phase: 'idle',
  objectType: null,
  points: [],
  form: null,
  error: null,
};

export type CreationAction =
  | { type: 'startPicking'; objectType: ObjectType }
  | { type: 'collectPoint'; point: Point3D }
  | { type: 'completePicking'; form: CreationForm }
  | { type: 'updateForm'; partial: Partial<CreationForm> }
  | { type: 'submitCreation' }
  | { type: 'finishCreation' }
  | { type: 'failCreation'; error: string }
  | { type: 'cancelCreation' }
  | { type: 'resetToPicking' };

function assertTransition(from: CreationPhase, action: string): void {
  const allowed: Record<string, CreationPhase[]> = {
    startPicking: ['idle'],
    collectPoint: ['picking'],
    completePicking: ['idle', 'picking'],
    updateForm: ['picking', 'configuring'],
    submitCreation: ['picking', 'configuring'],
    finishCreation: ['creating'],
    failCreation: ['creating'],
    cancelCreation: ['picking', 'configuring', 'error'],
    resetToPicking: ['error'],
  };
  const allowedFrom = allowed[action];
  if (allowedFrom && !allowedFrom.includes(from)) {
    throw new Error(
      `Illegal transition: action "${action}" is not allowed from phase "${from}". ` +
      `Allowed phases: ${allowedFrom.join(', ')}`
    );
  }
}

export function creationReducer(state: CreationState, action: CreationAction): CreationState {
  switch (action.type) {
    case 'startPicking': {
      assertTransition(state.phase, 'startPicking');
      return {
        phase: 'picking',
        objectType: action.objectType,
        points: [],
        form: createInitialForm(action.objectType, []),
        error: null,
      };
    }
    case 'collectPoint': {
      assertTransition(state.phase, 'collectPoint');
      const points = [...state.points, action.point];
      const form = state.form ? applyPointsToForm(state.form, points) : null;
      return { ...state, points, form };
    }
    case 'completePicking': {
      assertTransition(state.phase, 'completePicking');
      return { ...state, phase: 'configuring', form: action.form, error: null };
    }
    case 'updateForm': {
      assertTransition(state.phase, 'updateForm');
      if (!state.form) return state;
      return { ...state, form: { ...state.form, ...action.partial } };
    }
    case 'submitCreation': {
      assertTransition(state.phase, 'submitCreation');
      return { ...state, phase: 'creating' };
    }
    case 'finishCreation': {
      assertTransition(state.phase, 'finishCreation');
      return initialCreationState;
    }
    case 'failCreation': {
      assertTransition(state.phase, 'failCreation');
      return { ...state, phase: 'error', error: action.error };
    }
    case 'cancelCreation': {
      assertTransition(state.phase, 'cancelCreation');
      return initialCreationState;
    }
    case 'resetToPicking': {
      assertTransition(state.phase, 'resetToPicking');
      return {
        ...state,
        phase: 'picking',
        points: [],
        error: null,
      };
    }
    default:
      return state;
  }
}