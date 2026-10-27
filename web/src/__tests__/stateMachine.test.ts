import { describe, it, expect } from 'vitest';
import { creationReducer, initialCreationState, type CreationState, type CreationAction } from '../shared/creation/stateMachine';
import { DEFAULT_LAYERS_WALL, type ObjectType, type Point3D, type CreationForm } from '../shared/types/creation';

const mockForm: CreationForm = {
  objectType: 'wall',
  name: 'test-wall',
  ifcClass: 'IfcWall',
  desc: '',
  parentId: '',
  seriesId: '',
  posX: '0', posY: '0', posZ: '0',
  startX: '0', startY: '0', startZ: '0',
  endX: '5', endY: '0', endZ: '0',
  geometryType: 'box',
  p1: '5', p2: '1', p3: '3',
  length: '5', height: '3', width: '3.5',
  core: '0.2', finish: '0.02',
  thickness: '0.3', radius: '0.3',
  targetWallId: '',
  openingWidth: '1.0', openingHeight: '2.1',
  catalogType: 'concrete-rect',
  catalogDefId: '',
  paramValues: {},
  layers: DEFAULT_LAYERS_WALL,
  layersJson: JSON.stringify(DEFAULT_LAYERS_WALL),
  boundary_points: [],
  is_closed: false,
  pickMode: { mode: 'polyline', minClicks: 2 },
  roofMode: 'footprint',
  columnMode: 'place',
  columnHeight: '4',
};

function expectState(
  state: CreationState,
  overrides: Partial<CreationState>,
) {
  expect(state.phase).toBe(overrides.phase);
  if (overrides.objectType !== undefined) expect(state.objectType).toBe(overrides.objectType);
  if (overrides.points !== undefined) expect(state.points).toEqual(overrides.points);
  if (overrides.form !== undefined) expect(state.form).toEqual(overrides.form);
  if (overrides.error !== undefined) expect(state.error).toBe(overrides.error);
}

function expectTransition(
  from: CreationState,
  action: CreationAction,
  expected: Partial<CreationState>,
) {
  const next = creationReducer(from, action);
  expectState(next, expected);
}

function expectIllegal(
  from: CreationState,
  action: CreationAction,
) {
  expect(() => creationReducer(from, action)).toThrow(/Illegal transition/);
}

describe('creationReducer', () => {
  describe('idle phase', () => {
    it('allows startPicking and initializes form', () => {
      const next = creationReducer(initialCreationState, { type: 'startPicking', objectType: 'wall' });
      expect(next.phase).toBe('picking');
      expect(next.objectType).toBe('wall');
      expect(next.points).toEqual([]);
      expect(next.form).not.toBeNull();
      expect(next.form?.objectType).toBe('wall');
      expect(next.form?.name).toBe('');
      expect(next.form?.height).toBe('3');
      expect(next.form?.catalogType).toBe('concrete-rect');
      expect(next.error).toBeNull();
    });

    it('allows completePicking (non-picking types)', () => {
      expectTransition(initialCreationState, { type: 'completePicking', form: mockForm }, {
        phase: 'configuring',
        form: mockForm,
      });
    });

    it('rejects collectPoint', () => {
      expectIllegal(initialCreationState, { type: 'collectPoint', point: { x: 1, y: 2, z: 3 } });
    });

    it('rejects updateForm', () => {
      expectIllegal(initialCreationState, { type: 'updateForm', partial: { name: 'test' } });
    });

    it('rejects submitCreation', () => {
      expectIllegal(initialCreationState, { type: 'submitCreation' });
    });

    it('rejects finishCreation', () => {
      expectIllegal(initialCreationState, { type: 'finishCreation' });
    });

    it('rejects failCreation', () => {
      expectIllegal(initialCreationState, { type: 'failCreation', error: 'error' });
    });

    it('rejects cancelCreation', () => {
      expectIllegal(initialCreationState, { type: 'cancelCreation' });
    });

    it('rejects resetToPicking', () => {
      expectIllegal(initialCreationState, { type: 'resetToPicking' });
    });
  });

  describe('picking phase', () => {
    const pickingState: CreationState = {
      phase: 'picking',
      objectType: 'wall',
      points: [],
      form: { ...mockForm, posX: '0', posY: '0', posZ: '0', startX: '0', startY: '0', startZ: '0', endX: '5', endY: '0', endZ: '0', length: '5' },
      error: null,
    };

    it('allows collectPoint and updates form points', () => {
      const next = creationReducer(pickingState, { type: 'collectPoint', point: { x: 10, y: 5, z: 0 } });
      expect(next.phase).toBe('picking');
      expect(next.points).toEqual([{ x: 10, y: 5, z: 0 }]);
      expect(next.form?.posX).toBe('10.000');
      expect(next.form?.posY).toBe('5.000');
      expect(next.form?.posZ).toBe('0.000');
    });

    it('accumulates multiple points and updates form', () => {
      const state: CreationState = { ...pickingState, points: [{ x: 0, y: 0, z: 0 }] };
      const next = creationReducer(state, { type: 'collectPoint', point: { x: 10, y: 5, z: 0 } });
      expect(next.points).toEqual([{ x: 0, y: 0, z: 0 }, { x: 10, y: 5, z: 0 }]);
      expect(next.form?.startX).toBe('0.000');
      expect(next.form?.endX).toBe('10.000');
      expect(next.form?.length).toBe(Math.sqrt(10*10 + 5*5).toFixed(3));
    });

    it('allows updateForm', () => {
      const next = creationReducer(pickingState, { type: 'updateForm', partial: { name: 'updated-wall', height: '4.5' } });
      expect(next.phase).toBe('picking');
      expect(next.form?.name).toBe('updated-wall');
      expect(next.form?.height).toBe('4.5');
    });

    it('allows submitCreation', () => {
      expectTransition(pickingState, { type: 'submitCreation' }, {
        phase: 'creating',
      });
    });

    it('allows cancelCreation', () => {
      expectTransition(pickingState, { type: 'cancelCreation' }, {
        phase: 'idle',
      });
    });

    it('rejects startPicking', () => {
      expectIllegal(pickingState, { type: 'startPicking', objectType: 'beam' });
    });

    it('rejects finishCreation', () => {
      expectIllegal(pickingState, { type: 'finishCreation' });
    });

    it('rejects failCreation', () => {
      expectIllegal(pickingState, { type: 'failCreation', error: 'err' });
    });
  });

  describe('configuring phase', () => {
    const configuringState: CreationState = {
      phase: 'configuring',
      objectType: 'wall',
      points: [{ x: 0, y: 0, z: 0 }, { x: 5, y: 0, z: 0 }],
      form: mockForm,
      error: null,
    };

    it('allows updateForm', () => {
      const next = creationReducer(configuringState, { type: 'updateForm', partial: { name: 'updated-wall' } });
      expect(next.phase).toBe('configuring');
      expect(next.form?.name).toBe('updated-wall');
    });

    it('allows submitCreation', () => {
      expectTransition(configuringState, { type: 'submitCreation' }, {
        phase: 'creating',
      });
    });

    it('allows cancelCreation', () => {
      expectTransition(configuringState, { type: 'cancelCreation' }, {
        phase: 'idle',
      });
    });

    it('rejects startPicking', () => {
      expectIllegal(configuringState, { type: 'startPicking', objectType: 'wall' });
    });

    it('rejects collectPoint', () => {
      expectIllegal(configuringState, { type: 'collectPoint', point: { x: 1, y: 2, z: 3 } });
    });

    it('rejects finishCreation', () => {
      expectIllegal(configuringState, { type: 'finishCreation' });
    });

    it('rejects failCreation', () => {
      expectIllegal(configuringState, { type: 'failCreation', error: 'err' });
    });
  });

  describe('creating phase', () => {
    const creatingState: CreationState = {
      phase: 'creating',
      objectType: 'wall',
      points: [{ x: 0, y: 0, z: 0 }, { x: 5, y: 0, z: 0 }],
      form: mockForm,
      error: null,
    };

    it('allows finishCreation', () => {
      expectTransition(creatingState, { type: 'finishCreation' }, {
        phase: 'idle',
        objectType: null,
        points: [],
        form: null,
        error: null,
      });
    });

    it('allows failCreation', () => {
      expectTransition(creatingState, { type: 'failCreation', error: 'API error' }, {
        phase: 'error',
        error: 'API error',
      });
    });

    it('rejects startPicking', () => {
      expectIllegal(creatingState, { type: 'startPicking', objectType: 'wall' });
    });

    it('rejects collectPoint', () => {
      expectIllegal(creatingState, { type: 'collectPoint', point: { x: 1, y: 2, z: 3 } });
    });

    it('rejects submitCreation', () => {
      expectIllegal(creatingState, { type: 'submitCreation' });
    });

    it('rejects cancelCreation', () => {
      expectIllegal(creatingState, { type: 'cancelCreation' });
    });
  });

  describe('error phase', () => {
    const errorState: CreationState = {
      phase: 'error',
      objectType: 'wall',
      points: [{ x: 0, y: 0, z: 0 }, { x: 5, y: 0, z: 0 }],
      form: mockForm,
      error: 'API error',
    };

    it('allows cancelCreation', () => {
      expectTransition(errorState, { type: 'cancelCreation' }, {
        phase: 'idle',
      });
    });

    it('allows resetToPicking', () => {
      expectTransition(errorState, { type: 'resetToPicking' }, {
        phase: 'picking',
        points: [],
        error: null,
      });
    });

    it('rejects startPicking', () => {
      expectIllegal(errorState, { type: 'startPicking', objectType: 'wall' });
    });

    it('rejects collectPoint', () => {
      expectIllegal(errorState, { type: 'collectPoint', point: { x: 1, y: 2, z: 3 } });
    });

    it('rejects submitCreation', () => {
      expectIllegal(errorState, { type: 'submitCreation' });
    });

    it('rejects finishCreation', () => {
      expectIllegal(errorState, { type: 'finishCreation' });
    });

    it('rejects failCreation', () => {
      expectIllegal(errorState, { type: 'failCreation', error: 'another error' });
    });
  });

  describe('full workflow integration', () => {
    it('happy path: idle → picking → creating → idle (no configuring needed)', () => {
      let state = initialCreationState;

      state = creationReducer(state, { type: 'startPicking', objectType: 'wall' });
      expect(state.phase).toBe('picking');
      expect(state.form).not.toBeNull();
      expect(state.form?.height).toBe('3');

      state = creationReducer(state, { type: 'updateForm', partial: { height: '4.5' } });
      expect(state.phase).toBe('picking');
      expect(state.form?.height).toBe('4.5');

      state = creationReducer(state, { type: 'collectPoint', point: { x: 0, y: 0, z: 0 } });
      expect(state.phase).toBe('picking');
      expect(state.points).toHaveLength(1);
      expect(state.form?.posX).toBe('0.000');

      state = creationReducer(state, { type: 'collectPoint', point: { x: 5, y: 0, z: 0 } });
      expect(state.phase).toBe('picking');
      expect(state.points).toHaveLength(2);
      expect(state.form?.length).toBe('5.000');

      state = creationReducer(state, { type: 'submitCreation' });
      expect(state.phase).toBe('creating');

      state = creationReducer(state, { type: 'finishCreation' });
      expect(state.phase).toBe('idle');
    });

    it('non-picking path: idle → configuring → creating → idle', () => {
      let state = creationReducer(initialCreationState, { type: 'completePicking', form: mockForm });
      expect(state.phase).toBe('configuring');

      state = creationReducer(state, { type: 'submitCreation' });
      expect(state.phase).toBe('creating');

      state = creationReducer(state, { type: 'finishCreation' });
      expect(state.phase).toBe('idle');
    });

    it('error path: picking → creating → error → picking', () => {
      let state = creationReducer(initialCreationState, { type: 'startPicking', objectType: 'wall' });
      state = creationReducer(state, { type: 'submitCreation' });
      state = creationReducer(state, { type: 'failCreation', error: 'Network error' });
      expect(state.phase).toBe('error');
      expect(state.error).toBe('Network error');

      state = creationReducer(state, { type: 'resetToPicking' });
      expect(state.phase).toBe('picking');
      expect(state.points).toEqual([]);
      expect(state.error).toBeNull();
    });

    it('cancel during picking', () => {
      let state = creationReducer(initialCreationState, { type: 'startPicking', objectType: 'wall' });
      state = creationReducer(state, { type: 'collectPoint', point: { x: 1, y: 2, z: 3 } });
      state = creationReducer(state, { type: 'cancelCreation' });
      expect(state.phase).toBe('idle');
    });

    it('cancel during configuring', () => {
      let state = creationReducer(initialCreationState, { type: 'completePicking', form: mockForm });
      state = creationReducer(state, { type: 'cancelCreation' });
      expect(state.phase).toBe('idle');
    });
  });
});